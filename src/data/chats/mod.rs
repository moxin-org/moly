pub mod chat;

use chat::{Chat, ChatID};
use moxin_backend::Backend;
use moxin_protocol::{data::*, protocol::LoadModelOptions};
use moxin_protocol::protocol::{Command, LoadModelResponse};
use std::{cell::RefCell, rc::Rc, sync::mpsc::channel};
use anyhow::{Context, Result};

#[derive(Default)]
pub struct Chats {
    pub backend: Rc<Backend>,
    pub saved_chats: Vec<RefCell<Chat>>,
    pub current_chat_id: Option<ChatID>,   
}

impl Chats {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            saved_chats: Vec::new(),
            current_chat_id: None,
        }
    }

    pub fn load_model(&mut self, file: &File) -> Result<()> {
        let (tx, rx) = channel();
        let cmd = Command::LoadModel(
            file.id.clone(),
            LoadModelOptions {
                prompt_template: None,
                gpu_layers: moxin_protocol::protocol::GPULayers::Max,
                use_mlock: false,
                n_batch: 512,
                n_ctx: 512,
                rope_freq_scale: 0.0,
                rope_freq_base: 0.0,
                context_overflow_policy:
                    moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            },
            tx,
        );

        self.backend.as_ref().command_sender.send(cmd).unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(response) => {
                    let LoadModelResponse::Completed(_) = response else {
                        eprintln!("Error loading model");
                        return Ok(());
                    };
                    // TODO: Creating a new chat, maybe put in a method and save on disk or smth.
                    let new_chat = RefCell::new(Chat::new(file.name.clone(), file.id.clone()));
                    self.current_chat_id = Some(new_chat.borrow().id);
                    self.saved_chats.push(new_chat);

                }
                Err(err) => {
                    eprintln!("Error loading model: {:?}", err);
                    return Err(err);
                }
            }
        };

        Err(anyhow::anyhow!("Error loading model"))
    }

    pub fn get_current_chat(&self) -> Option<&RefCell<Chat>> {
        if let Some(current_chat_id) = self.current_chat_id {
            self.saved_chats
                .iter()
                .find(|c| c.borrow().id == current_chat_id)
        } else {
            None
        }
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut()
                .send_message_to_model(prompt, &self.backend.as_ref());
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().cancel_streaming(&self.backend.as_ref());
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
        }
    }

    pub fn edit_chat_message(
        &mut self,
        message_id: usize,
        updated_message: String,
        regenerate: bool,
    ) {
        if let Some(chat) = &mut self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            if regenerate {
                if chat.is_streaming {
                    chat.cancel_streaming(&self.backend.as_ref());
                }

                chat.remove_messages_from(message_id);
                chat.send_message_to_model(updated_message, &self.backend.as_ref());
            } else {
                chat.edit_message(message_id, updated_message);
            }
        }
    }

    pub fn eject_model(&mut self) -> Result<()> {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::EjectModel(tx))
            .context("Failed to send eject model command")?;

        let _ = rx
            .recv()
            .context("Failed to receive eject model response")?
            .context("Eject model operation failed");

        self.current_chat_id = None;
        Ok(())
    }
}
