use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::data::FileID;
use moxin_protocol::open_ai::*;
use moxin_protocol::protocol::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum ChatTokenArrivalAction {
    AppendDelta(String),
    StreamingDone,
}

#[derive(Clone, Debug)]
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

#[derive(Debug)]
pub struct Chat {
    /// Unix timestamp in ms.
    pub id: u128,
    pub model_filename: String,
    pub file_id: FileID,
    pub messages: Vec<ChatMessage>,
    pub messages_update_sender: Sender<ChatTokenArrivalAction>,
    pub messages_update_receiver: Receiver<ChatTokenArrivalAction>,
    pub is_streaming: bool,
}

impl Chat {
    pub fn new(filename: String, file_id: FileID) -> Self {
        let (tx, rx) = channel();

        // Get Unix timestamp in ms for id.
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Couldn't get Unix timestamp, time went backwards")
            .as_millis();

        Self {
            id,
            model_filename: filename,
            file_id,
            messages: vec![],
            messages_update_sender: tx,
            messages_update_receiver: rx,
            is_streaming: false,
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
