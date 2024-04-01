use std::sync::mpsc::{Sender, Receiver, channel};
use moxin_backend::Backend;
use makepad_widgets::SignalToUI;
use std::thread;
use moxin_protocol::open_ai::*;
use moxin_protocol::protocol::Command;

pub enum ChatTokenArrivalAction {
    AppendDelta(String),
    StreamingDone,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn is_assistant(&self) -> bool {
        // self.role == Role::Assistant
        matches!(self.role, Role::Assistant)
    }
}

pub struct Chat {
    pub model_filename: String,
    pub messages: Vec<ChatMessage>,
    pub messages_update_sender: Sender<ChatTokenArrivalAction>,
    pub messages_update_receiver: Receiver<ChatTokenArrivalAction>,
    pub is_streaming: bool,
}

impl Chat {
    pub fn new(filename: String) -> Self {
        let (tx, rx) = channel();
        let chat = Self {
            model_filename: filename,
            messages: vec![],
            messages_update_sender: tx,
            messages_update_receiver: rx,
            is_streaming: false,
        };
        chat
    }

    pub fn send_message_to_model(&mut self, prompt: String, backend: &Backend) {
        let (tx, rx) = channel();
        let mut messages: Vec<_> = self.messages.iter().map(|message| {
            Message {
                content: message.content.clone(),
                role: message.role.clone(),
                name: None,
            }
        }).collect();

        messages.push(Message {
            content: prompt.clone(),
            role: Role::User,
            name: None,
        });

        let cmd = Command::Chat(
            ChatRequestData {
                messages: messages,
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

        self.messages.push(ChatMessage{role: Role::User, content: prompt.clone()});
        self.messages.push(ChatMessage{role: Role::Assistant, content: "".to_string()});

        let store_chat_tx = self.messages_update_sender.clone();
        backend.command_sender.send(cmd).unwrap();
        self.is_streaming = true;
        thread::spawn(move || {
            loop {
                if let Ok(response) = rx.recv() {
                    match response {
                        Ok(ChatResponse::ChatResponseChunk(data)) => {
                            let mut is_done = false;

                            store_chat_tx.send(ChatTokenArrivalAction::AppendDelta(
                                data.choices[0].delta.content.clone()
                            )).unwrap();

                            if let Some(_reason) = &data.choices[0].finish_reason {
                                is_done = true;
                                store_chat_tx.send(ChatTokenArrivalAction::StreamingDone).unwrap();
                            }

                            SignalToUI::set_ui_signal();
                            if is_done { break; }
                        },
                        Err(err) => eprintln!("Error receiving response chunk: {:?}", err),
                        _ => (),
                    }
                };
            }
        });
    }

    pub fn update_messages(&mut self) {
        for msg in self.messages_update_receiver.try_iter() {
            match msg {
                ChatTokenArrivalAction::AppendDelta(response) => {
                    let last = self.messages.last_mut().unwrap();
                    last.content.push_str(&response);
                },
                ChatTokenArrivalAction::StreamingDone => {
                    self.is_streaming = false;
                },
            }
        }
    }
}