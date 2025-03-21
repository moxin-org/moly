use anyhow::{anyhow, Result};
use makepad_widgets::Cx;
use moly_protocol::data::{File, FileID};
use moly_protocol::open_ai::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc::{self, channel};
use std::thread;
use std::sync::{Arc, RwLock};

use crate::data::filesystem::{read_from_file, write_to_file};
use crate::data::providers::{Article, RemoteModel, Stage};
use crate::data::providers::ChatResponse as RemoteModelChatResponse;

use super::chat_entity::ChatEntityId;
use super::model_loader::ModelLoader;
use super::{MolyClient, ProviderClient};

pub type ChatID = u128;

#[derive(Debug)]
pub struct ChatEntityAction {
    pub chat_id: ChatID,
    kind: ChatEntityActionKind,
}

#[derive(Debug)]
enum ChatEntityActionKind {
    ModelAppendDelta(String),
    ModelStreamingDone,
    ModelStage(Stage),
    MofaAgentResult(String, Vec<Article>, bool),
    MofaAgentCancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub id: usize,
    pub role: Role,
    pub username: Option<String>,
    pub entity: Option<ChatEntityId>,
    pub content: String,
    pub articles: Vec<Article>,
    pub stages: Vec<Stage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stages {
    pub thinking: Option<String>, 
    pub writing: Option<String>,
    pub completed: Option<String>,
}

impl ChatMessage {
    pub fn is_assistant(&self) -> bool {
        matches!(self.role, Role::Assistant)
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
enum TitleState {
    #[default]
    Default,
    Updated,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ChatState {
    #[default]
    Idle,
    Receiving,
    // The boolean indicates if the last message should be removed.
    #[allow(dead_code)]
    Cancelled(bool),
}

#[derive(Serialize, Deserialize)]
struct ChatData {
    id: ChatID,
    associated_entity: Option<ChatEntityId>,
    system_prompt: Option<String>,
    messages: Vec<ChatMessage>,
    title: String,
    #[serde(default)]
    title_state: TitleState,
    #[serde(default)]
    accessed_at: chrono::DateTime<chrono::Utc>,

    // Legacy field, it can be removed in the future.
    last_used_file_id: Option<FileID>,
}

#[derive(Debug)]
pub struct ChatInferenceParams {
    pub frequency_penalty: f32,
    pub max_tokens: u32,
    pub presence_penalty: f32,
    pub temperature: f32,
    pub top_p: f32,
    pub stream: bool,
    pub stop: String,
}

impl Default for ChatInferenceParams {
    fn default() -> Self {
        Self {
            frequency_penalty: 0.0,
            max_tokens: 2048,
            presence_penalty: 0.0,
            temperature: 1.0,
            top_p: 1.0,
            stream: true,
            stop: "".into(),
        }
    }
}

#[derive(Debug)]
pub struct Chat {
    /// Unix timestamp in ms.
    pub id: ChatID,

    /// This is the model or agent that is currently "active" on the chat
    /// For models it is the most recent model used or loaded in the context of this chat session.
    /// For agents it is the agent that originated the chat.
    pub associated_entity: Option<ChatEntityId>,

    pub messages: Vec<ChatMessage>,
    pub is_streaming: bool,
    pub inferences_params: ChatInferenceParams,
    pub system_prompt: Option<String>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,

    title: String,
    title_state: TitleState,
    state: Arc<RwLock<ChatState>>,
    chats_dir: PathBuf,
}

impl Chat {
    pub fn new(chats_dir: PathBuf) -> Self {
        // Get Unix timestamp in ms for id.
        let id = chrono::Utc::now().timestamp_millis() as u128;

        Self {
            id,
            title: String::from("New Chat"),
            messages: vec![],
            associated_entity: None,
            state: Arc::new(RwLock::new(ChatState::Idle)),
            is_streaming: false,
            title_state: TitleState::default(),
            chats_dir,
            inferences_params: ChatInferenceParams::default(),
            system_prompt: None,
            accessed_at: chrono::Utc::now(),
        }
    }

    pub fn load(path: PathBuf, chats_dir: PathBuf) -> Result<Self> {
        match read_from_file(path) {
            Ok(json) => {
                let data: ChatData = serde_json::from_str(&json)?;

                // Fallback to last_used_file_id if last_used_entity is None.
                // Until this field is removed, we need to keep this logic.
                let chat_entity = data.associated_entity.or_else(|| {
                    data.last_used_file_id
                        .map(|file_id| ChatEntityId::ModelFile(file_id))
                });

                let chat = Chat {
                    id: data.id,
                    associated_entity: chat_entity,
                    messages: data.messages,
                    title: data.title,
                    title_state: data.title_state,
                    state: Arc::new(RwLock::new(ChatState::Idle)),
                    is_streaming: false,
                    chats_dir,
                    inferences_params: ChatInferenceParams::default(),
                    system_prompt: data.system_prompt,
                    accessed_at: data.accessed_at,
                };
                Ok(chat)
            }
            Err(_) => Err(anyhow!("Couldn't read chat file from path")),
        }
    }

    pub fn save(&self) {
        let data = ChatData {
            id: self.id,
            associated_entity: self.associated_entity.clone(),
            system_prompt: self.system_prompt.clone(),
            messages: self.messages.clone(),
            title: self.title.clone(),
            title_state: self.title_state,
            accessed_at: self.accessed_at,

            // Legacy field, it can be removed in the future.
            last_used_file_id: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        let path = self.chats_dir.join(self.file_name());
        write_to_file(path, &json).unwrap();
    }

    pub fn remove_saved_file(&self) {
        let path = self.chats_dir.join(self.file_name());
        std::fs::remove_file(path).unwrap();
    }

    fn file_name(&self) -> String {
        format!("{}.chat.json", self.id)
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.title_state = TitleState::Updated;
    }

    pub fn update_title_based_on_first_message(&mut self) {
        // If it hasnt been updated, and theres at least one message, use the first
        // one as title. Else we just return the default one.
        if matches!(self.title_state, TitleState::Default) {
            if let Some(message) = self.messages.first() {
                let max_char_length = 25;
                let ellipsis = "...";

                let title = if message.content.len() > max_char_length {
                    let mut truncated = message
                        .content
                        .chars()
                        .take(max_char_length)
                        .collect::<String>()
                        .replace('\n', " ");
                    truncated.push_str(ellipsis);
                    truncated
                } else {
                    message.content.clone()
                };

                self.set_title(title);
            }
        }
    }

    pub fn send_message_to_model(
        &mut self,
        prompt: String,
        wanted_file: &File,
        mut model_loader: ModelLoader,
        moly_client: &MolyClient,
    ) {
        let mut messages: Vec<_> = self
            .messages
            .iter()
            .map(|message| Message {
                content: message.content.clone(),
                role: message.role.clone(),
                name: None,
            })
            .collect();

        messages.push(Message {
            content: prompt.clone(),
            role: Role::User,
            name: None,
        });

        if let Some(system_prompt) = &self.system_prompt {
            messages.insert(
                0 as usize,
                Message {
                    content: system_prompt.clone(),
                    role: Role::System,
                    name: None,
                },
            );
        } else {
            messages.insert(
                0 as usize,
                Message {
                    content: "You are a helpful, respectful, and honest assistant.".to_string(),
                    role: Role::System,
                    name: None,
                },
            );
        }

        let (tx, rx) = channel();

        let ip = &self.inferences_params;
        let request = ChatRequestData {
            messages,
            model: wanted_file.name.clone(),
            frequency_penalty: Some(ip.frequency_penalty),
            logprobs: None,
            top_logprobs: None,
            max_tokens: Some(ip.max_tokens),
            presence_penalty: Some(ip.presence_penalty),
            seed: None,
            stop: Some(
                ip.stop
                    .split(",")
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
            ),
            stream: Some(ip.stream),
            temperature: Some(ip.temperature),
            top_p: Some(ip.top_p),
            n: None,
            logit_bias: None,
        };

        let next_id = self.messages.last().map(|m| m.id).unwrap_or(0) + 1;
        self.messages.push(ChatMessage {
            id: next_id,
            role: Role::User,
            username: None,
            entity: None,
            content: prompt.clone(),
            articles: vec![],
            stages: vec![],
        });

        self.messages.push(ChatMessage {
            id: next_id + 1,
            role: Role::Assistant,
            username: Some(wanted_file.name.clone()),
            entity: Some(ChatEntityId::ModelFile(wanted_file.id.clone())),
            content: "".to_string(),
            articles: vec![],
            stages: vec![],
        });

        *self.state.write().unwrap() = ChatState::Receiving;

        let wanted_file = wanted_file.clone();
        let chat_id = self.id;
        let moly_client = moly_client.clone();
        let state_clone = self.state.clone();

        thread::spawn(move || {
            if let Err(err) = model_loader.load(wanted_file.id.clone(), moly_client.clone(), None) {
                eprintln!("Error loading model: {}", err);
                return;
            }

            moly_client.send_chat_message(request, tx);

            loop {
                // Check if we've been cancelled
                if matches!(*state_clone.read().unwrap(), ChatState::Cancelled(_)) {
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::ModelStreamingDone,
                    });
                    break;
                }

                if let Ok(response) = rx.recv() {
                    match response {
                        // Streaming response
                        Ok(ChatResponse::ChatResponseChunk(data)) => {
                            if let Some(_reason) = &data.choices[0].finish_reason {
                                Cx::post_action(ChatEntityAction {
                                    chat_id,
                                    kind: ChatEntityActionKind::ModelStreamingDone,
                                });
                                break;
                            } else if !data.choices[0].delta.content.is_empty() {
                                Cx::post_action(ChatEntityAction {
                                    chat_id,
                                    kind: ChatEntityActionKind::ModelAppendDelta(
                                        data.choices[0].delta.content.clone(),
                                    ),
                                });
                            }
                        }
                        // Non-streaming response
                        Ok(ChatResponse::ChatFinalResponseData(data)) => {
                            Cx::post_action(ChatEntityAction {
                                chat_id,
                                kind: ChatEntityActionKind::ModelAppendDelta(
                                    data.choices[0].message.content.clone(),
                                ),
                            });

                            Cx::post_action(ChatEntityAction {
                                chat_id,
                                kind: ChatEntityActionKind::ModelStreamingDone,
                            });

                            break;
                        }
                        Err(err) => eprintln!("Error receiving response chunk: {:?}", err),
                    }
                } else {
                    break;
                };
            }
        });

        self.update_title_based_on_first_message();
        self.save();
    }

    // TODO: this and send_message_to_remote_model are very similar, we should refactor them
    pub fn send_message_to_agent(
        &mut self,
        agent: &RemoteModel,
        prompt: String,
        client: &dyn ProviderClient,
    ) {
        let (tx, rx) = mpsc::channel();
        client.send_message(&agent, &prompt, tx);

        let next_id = self.messages.last().map(|m| m.id).unwrap_or(0) + 1;

        self.messages.push(ChatMessage {
            id: next_id,
            role: Role::User,
            username: None,
            entity: None,
            content: prompt,
            articles: vec![],
            stages: vec![],
        });

        self.messages.push(ChatMessage {
            id: next_id + 1,
            role: Role::Assistant,
            username: Some(agent.name.clone()),
            entity: Some(ChatEntityId::Agent(agent.id.clone())),
            content: "".to_string(),
            articles: vec![],
            stages: vec![],
        });

        *self.state.write().unwrap() = ChatState::Receiving;

        let chat_id = self.id;
        std::thread::spawn(move || '_loop: loop {
            match rx.recv() {
                Ok(RemoteModelChatResponse::ChatFinalResponseData(data, is_final)) => {
                    let content = data.content.clone();
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::MofaAgentResult(content, data.articles, is_final),
                    });

                    break '_loop;
                }
                Ok(RemoteModelChatResponse::ChatStage(stage)) => {
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::ModelStage(stage),
                    });
                }
                Err(e) => {
                    println!("Error receiving response from agent: {:?}", e);
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::MofaAgentCancelled,
                    });

                    break '_loop;
                }
            }
        });

        self.update_title_based_on_first_message();
        self.save();
    }

    pub fn send_message_to_remote_model(
        &mut self,
        prompt: String,
        remote_model: &RemoteModel,
        client: &dyn ProviderClient,
    ) {
        let (tx, rx) = mpsc::channel();
        client.send_message(&remote_model, &prompt, tx);

        let next_id = self.messages.last().map(|m| m.id).unwrap_or(0) + 1;

        self.messages.push(ChatMessage {
            id: next_id,
            role: Role::User,
            username: None,
            entity: None,
            content: prompt,
            articles: vec![],
            stages: vec![],
        });

        self.messages.push(ChatMessage {
            id: next_id + 1,
            role: Role::Assistant,
            username: Some(remote_model.name.clone()),
            entity: Some(ChatEntityId::RemoteModel(remote_model.id.clone())),
            content: "".to_string(),
            articles: vec![],
            stages: vec![],
        });

        *self.state.write().unwrap() = ChatState::Receiving;

        let chat_id = self.id;
        std::thread::spawn(move || '_loop: loop {
            match rx.recv() {
                Ok(RemoteModelChatResponse::ChatFinalResponseData(data, is_final)) => {
                    let content = data.content.clone();
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::MofaAgentResult(content, data.articles, is_final),
                    });

                    if is_final {
                        break '_loop;
                    }
                }
                Ok(RemoteModelChatResponse::ChatStage(stage)) => {
                    let should_break = matches!(stage, Stage::Completed(_, _));
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::ModelStage(stage),
                    });

                    if should_break {
                        break '_loop;
                    }
                }
                Err(e) => {
                    println!("Error receiving response from agent: {:?}", e);
                    Cx::post_action(ChatEntityAction {
                        chat_id,
                        kind: ChatEntityActionKind::MofaAgentCancelled,
                    });

                    break '_loop;
                }
            }
        });

        self.update_title_based_on_first_message();
        self.save();
    }

    pub fn cancel_streaming(&mut self) {
        if matches!(*self.state.read().unwrap(), ChatState::Idle | ChatState::Cancelled(_)) {
            return;
        }

        let message = self.messages.last_mut().unwrap();
        let new_state = if message.content.trim().is_empty() {
            ChatState::Cancelled(true)
        } else {
            ChatState::Cancelled(false)
        };
        
        *self.state.write().unwrap() = new_state;
    }


    pub fn cancel_interaction(&mut self, client: &dyn ProviderClient) {
        if matches!(*self.state.read().unwrap(), ChatState::Idle | ChatState::Cancelled(_)) {
            return;
        }

        client.cancel_task();

        *self.state.write().unwrap() = ChatState::Cancelled(true);
        let message = self.messages.last_mut().unwrap();
        message.content = "Cancelling, please wait...".to_string();
    }

    pub fn delete_message(&mut self, message_id: usize) {
        self.messages.retain(|message| message.id != message_id);
    }

    pub fn edit_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(message) = self.messages.iter_mut().find(|m| m.id == message_id) {
            message.content = updated_message;
        }
    }

    pub fn remove_messages_from(&mut self, message_id: usize) {
        let message_index = self
            .messages
            .iter()
            .position(|m| m.id == message_id)
            .unwrap();
        self.messages.truncate(message_index);
    }

    pub fn update_accessed_at(&mut self) {
        self.accessed_at = chrono::Utc::now();
    }

    pub fn is_receiving(&self) -> bool {
        matches!(*self.state.read().unwrap(), ChatState::Receiving)
    }

    pub fn was_cancelled(&self) -> bool {
        matches!(*self.state.read().unwrap(), ChatState::Cancelled(_))
    }

    pub fn handle_action(&mut self, action: &ChatEntityAction) {
        match &action.kind {
            ChatEntityActionKind::ModelAppendDelta(response) => {
                let last = self.messages.last_mut().unwrap();
                last.content.push_str(&response);
            }
            ChatEntityActionKind::ModelStreamingDone => {
                self.is_streaming = false;
                *self.state.write().unwrap() = ChatState::Idle;
            }
            ChatEntityActionKind::MofaAgentResult(response, articles, is_final) => {
                let last = self.messages.last_mut().unwrap();
                last.content = response.clone();
                last.articles = articles.clone();
                if *is_final {
                    *self.state.write().unwrap() = ChatState::Idle;
                }
            }
            ChatEntityActionKind::MofaAgentCancelled => {
                *self.state.write().unwrap() = ChatState::Idle;
                // Remove the last message sent by the user
                self.messages.pop();
            }
            ChatEntityActionKind::ModelStage(stage) => {
                let last = self.messages.last_mut().unwrap();

                // Often DeepInquire returns the same stage multiple times with the content extended (instead of a delta).
                // However sometimes it returns the same stage with completely different content.
                // To avoid very abrupt changes in the text (and because I don't think it's useful otherwise), we check: 
                // if the new stage content is an extension of the previous stage content, we simply add a new stage.
                // However if the new stage content is completely different, we add a new stage with a combination of the previous stage and the new stage.
                // This way the user can see the evolution of the thought process rather than just a few sentences at a time that don't 
                // make sense in isolation.
                if let Some(last_stage) = last.stages.last() {
                    match stage {
                        Stage::Thinking(content) => {        
                            match last_stage {
                                Stage::Thinking(last_content) => {
                                    if last_content.contains(content) {
                                        last.stages.push(Stage::Thinking(content.clone()));
                                    } else {
                                        last.stages.push(Stage::Thinking(format!("{} {}", last_content, content)));
                                    }
                                },
                                _ => {}
                            }
                        }
                        Stage::Writing(_content, _articles) => {
                            last.stages.push(stage.clone())
                        }
                        Stage::Completed(_content, _articles) => {
                            last.stages.push(stage.clone());
                            *self.state.write().unwrap() = ChatState::Idle;
                        }
                    }
                } else {
                    last.stages.push(stage.clone());
                }
            }
        }
        self.save();
    }
}

#[derive(Serialize, Deserialize)]
struct MofaAgentResponse {
    step_name: String,
    node_results: String,
    dataflow_status: bool,
}
