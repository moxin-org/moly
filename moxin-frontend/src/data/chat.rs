use std::sync::mpsc::{Sender, Receiver, channel};
use moxin_backend::Backend;
use makepad_widgets::SignalToUI;
use std::thread;
use moxin_protocol::open_ai::*;
use moxin_protocol::protocol::Command;

pub enum ChatHistoryUpdate {
    Append(String),
}

pub struct Chat {
    pub messages: Vec<String>,
    pub messages_update_sender: Sender<ChatHistoryUpdate>,
    pub messages_update_receiver: Receiver<ChatHistoryUpdate>,
}

impl Chat {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let chat = Self {
            messages: vec![],
            messages_update_sender: tx,
            messages_update_receiver: rx,
        };
        chat
    }

    pub fn send_message_to_model(&mut self, prompt: String, backend: &Backend) {
        let (tx, rx) = channel();
        let mut messages: Vec<_> = self.messages.iter().enumerate().map(|(i, message)| {
            let role = if i % 2 == 0 { Role::User } else { Role::Assistant };
            Message {
                content: message.clone(),
                role: role,
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

        self.messages.push(prompt.clone());
        self.messages.push("".to_string());

        let store_chat_tx = self.messages_update_sender.clone();
        backend.command_sender.send(cmd).unwrap();

        thread::spawn(move || {
            loop {
                if let Ok(response) = rx.recv() {
                    match response {
                        Ok(ChatResponse::ChatResponseChunk(data)) => {
                            store_chat_tx.send(ChatHistoryUpdate::Append(
                                data.choices[0].delta.content.clone()
                            )).unwrap();

                            SignalToUI::set_ui_signal();

                            if let Some(_reason) = &data.choices[0].finish_reason {
                                break;
                            }
                        },
                        Err(err) => eprintln!("Error receiving response chunk: {:?}", err),
                        _ => (),
                    }
                };
            }
        });
    }

    pub fn update_messages(&mut self) {
        if let Ok(ChatHistoryUpdate::Append(response)) =
            self.messages_update_receiver.recv() {
                let last = self.messages.last_mut().unwrap();
                last.push_str(&response);
        }
    }
}