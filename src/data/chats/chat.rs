use anyhow::{anyhow, Result};
use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::data::FileID;
use moxin_protocol::open_ai::*;
use moxin_protocol::protocol::Command;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer, Serialize};

use crate::data::filesystem::{read_from_file, write_to_file};

pub type ChatID = u128;

#[derive(Clone, Debug)]
pub enum ChatTokenArrivalAction {
    AppendDelta(String),
    StreamingDone,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: usize,
    pub role: Role,
    pub content: String,
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

#[derive(Serialize, Deserialize)]
struct ChatData {
    id: ChatID,
    model_filename: String,
    file_id: FileID,
    messages: Vec<ChatMessage>,
    title: String,
    #[serde(default)]
    title_state: TitleState,
}

#[derive(Debug)]
pub struct Chat {
    /// Unix timestamp in ms.
    pub id: ChatID,
    pub model_filename: String,
    pub file_id: FileID,
    pub messages: Vec<ChatMessage>,
    pub messages_update_sender: Sender<ChatTokenArrivalAction>,
    pub messages_update_receiver: Receiver<ChatTokenArrivalAction>,
    pub is_streaming: bool,

    title: String,
    title_state: TitleState,

    chats_dir: PathBuf,
}

impl Chat {
    pub fn new(filename: String, file_id: FileID, chats_dir: PathBuf) -> Self {
        let (tx, rx) = channel();

        // Get Unix timestamp in ms for id.
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Couldn't get Unix timestamp, time went backwards")
            .as_millis();

        Self {
            id,
            title: String::from("New Chat"),
            model_filename: filename,
            file_id,
            messages: vec![],
            messages_update_sender: tx,
            messages_update_receiver: rx,
            is_streaming: false,
            title_state: TitleState::default(),
            chats_dir,
        }
    }

    pub fn load(path: PathBuf, chats_dir: PathBuf) -> Result<Self> {
        let (tx, rx) = channel();

        match read_from_file(path) {
            Ok(json) => {
                let data: ChatData = serde_json::from_str(&json)?;
                let chat = Chat {
                    id: data.id,
                    model_filename: data.model_filename,
                    file_id: data.file_id,
                    messages: data.messages,
                    title: data.title,
                    title_state: data.title_state,
                    is_streaming: false,
                    messages_update_sender: tx,
                    messages_update_receiver: rx,
                    chats_dir,
                };
                Ok(chat)
            }
            Err(_) => Err(anyhow!("Couldn't read chat file from path")),
        }
    }

    pub fn save(&self) {
        let data = ChatData {
            id: self.id,
            model_filename: self.model_filename.clone(),
            file_id: self.file_id.clone(),
            messages: self.messages.clone(),
            title: self.title.clone(),
            title_state: self.title_state,
        };
        let json = serde_json::to_string(&data).unwrap();
        let path = self.chats_dir.join(self.file_name());
        write_to_file(path, &json).unwrap();
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

    fn update_title_based_on_first_message(&mut self) {
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
    pub fn send_message_to_model(&mut self, prompt: String, backend: &Backend) {
        let (tx, rx) = channel();
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

        let cmd = Command::Chat(
            ChatRequestData {
                messages,
                model: "llama-2-7b-chat.Q5_K_M".to_string(),
                frequency_penalty: None,
                logprobs: None,
                top_logprobs: None,
                max_tokens: None,
                presence_penalty: None,
                seed: None,
                stop: None,
                stream: Some(true),
                temperature: None,
                top_p: None,
                n: None,
                logit_bias: None,
            },
            tx,
        );

        let next_id = self.messages.last().map(|m| m.id).unwrap_or(0) + 1;
        self.messages.push(ChatMessage {
            id: next_id,
            role: Role::User,
            content: prompt.clone(),
        });
        self.messages.push(ChatMessage {
            id: next_id + 1,
            role: Role::Assistant,
            content: "".to_string(),
        });

        let store_chat_tx = self.messages_update_sender.clone();
        backend.command_sender.send(cmd).unwrap();
        self.is_streaming = true;
        thread::spawn(move || loop {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(ChatResponse::ChatResponseChunk(data)) => {
                        let mut is_done = false;

                        let _ = store_chat_tx.send(ChatTokenArrivalAction::AppendDelta(
                            data.choices[0].delta.content.clone(),
                        ));

                        if let Some(_reason) = &data.choices[0].finish_reason {
                            is_done = true;
                            let _ = store_chat_tx.send(ChatTokenArrivalAction::StreamingDone);
                        }

                        SignalToUI::set_ui_signal();
                        if is_done {
                            break;
                        }
                    }
                    Err(err) => eprintln!("Error receiving response chunk: {:?}", err),
                    _ => (),
                }
            } else {
                break;
            };
        });

        self.update_title_based_on_first_message();
    }

    pub fn cancel_streaming(&mut self, backend: &Backend) {
        let (tx, _rx) = channel();
        let cmd = Command::StopChatCompletion(tx);
        backend.command_sender.send(cmd).unwrap();

        makepad_widgets::log!("Cancel streaming");
    }

    pub fn update_messages(&mut self) {
        for msg in self.messages_update_receiver.try_iter() {
            match msg {
                ChatTokenArrivalAction::AppendDelta(response) => {
                    let last = self.messages.last_mut().unwrap();
                    last.content.push_str(&response);
                }
                ChatTokenArrivalAction::StreamingDone => {
                    self.is_streaming = false;
                }
            }
            self.save();
        }
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
}
