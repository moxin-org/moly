use anyhow::{anyhow, Result};
use moly_protocol::data::FileID;
use moly_protocol::open_ai::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::data::filesystem::{read_from_file, write_to_file};
use crate::data::providers::{DeepInquireMessage, DeepInquireStage};

use super::chat_entity::ChatEntityId;
use super::ProviderClient;

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
    DeepnInquireResponse(DeepInquireMessage),
    MofaAgentResult(String, bool),
    MofaAgentCancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub id: usize,
    pub role: Role,
    pub username: Option<String>,
    pub entity: Option<ChatEntityId>,
    pub content: String,
    pub stages: Vec<DeepInquireStage>,
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
            ChatEntityActionKind::MofaAgentResult(response, is_final) => {
                let last = self.messages.last_mut().unwrap();
                last.content = response.clone();
                if *is_final {
                    *self.state.write().unwrap() = ChatState::Idle;
                }
            }
            ChatEntityActionKind::MofaAgentCancelled => {
                *self.state.write().unwrap() = ChatState::Idle;
                // Remove the last message sent by the user
                self.messages.pop();
            }
            ChatEntityActionKind::DeepnInquireResponse(message) => {
                let last_message = self.messages.last_mut().unwrap();

                // Completed messages arrive with stage id 1, but they should be the last stage
                // I'm actually unsure of this behavior, perhaps it's a bug, perhaps we should add the completed block
                // to the last active stage instead of creating a new one.
                match message {
                    DeepInquireMessage::Completed(_, content) => {
                        last_message.stages.push(DeepInquireStage { 
                            id: last_message.stages.len(),
                            thinking: None,
                            writing: None,
                            completed: Some(content.clone()),
                        });
                        *self.state.write().unwrap() = ChatState::Idle;
                        return;
                    }
                    _ => {}
                }

                // If there is an existing stage with the same id, we update it
                if let Some(existing_stage) = last_message.stages.iter_mut().find(|stage| stage.id == message.id()) {
                    match message {
                        DeepInquireMessage::Thinking(_, content) => {
                            existing_stage.thinking = Some(content.clone());
                        }
                        DeepInquireMessage::Writing(_, content) => {
                            existing_stage.writing = Some(content.clone());
                        }
                        DeepInquireMessage::Completed(_, content) => {
                            existing_stage.completed = Some(content.clone());
                            *self.state.write().unwrap() = ChatState::Idle;
                        }
                    }
                } else {
                    // Otherwise we add a new stage
                    let mut new_stage = DeepInquireStage {
                        id: message.id(),
                        thinking: None,
                        writing: None,
                        completed: None,
                    };

                    match message {
                        DeepInquireMessage::Thinking(_, content) => {
                            new_stage.thinking = Some(content.clone());
                        }
                        DeepInquireMessage::Writing(_, content) => {
                            new_stage.writing = Some(content.clone());
                        }
                        DeepInquireMessage::Completed(_, content) => {
                            new_stage.completed = Some(content.clone());
                            *self.state.write().unwrap() = ChatState::Idle;
                        }
                    }

                    last_message.stages.push(new_stage);
                }
            }
        }
        self.save();
    }
}
