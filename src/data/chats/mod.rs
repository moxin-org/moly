pub mod chat;
pub mod model_loader;

use anyhow::{Context, Result};
use chat::{Chat, ChatID};
use model_loader::ModelLoader;
use moxin_backend::Backend;
use moxin_protocol::data::*;
use moxin_protocol::protocol::Command;
use std::fs;
use std::sync::mpsc::channel;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use super::filesystem::setup_chats_folder;

pub struct Chats {
    pub backend: Rc<Backend>,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub loaded_model: Option<File>,
    pub model_loader: Option<ModelLoader>,

    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,
}

impl Chats {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            saved_chats: Vec::new(),
            current_chat_id: None,
            loaded_model: None,
            model_loader: None,
            chats_dir: setup_chats_folder(),
        }
    }

    pub fn load_chats(&mut self) {
        let paths = fs::read_dir(&self.chats_dir).unwrap();

        for path in paths.map(|p| p.unwrap().path()) {
            let loaded_chat_result = Chat::load(path, self.chats_dir.clone());
            match loaded_chat_result {
                Err(e) => {
                    eprintln!("{}", &e.to_string());
                }
                Ok(loaded_chat) => self.saved_chats.push(RefCell::new(loaded_chat)),
            }
        }
    }

    pub fn get_last_selected_chat_id(&self) -> Option<ChatID> {
        self.saved_chats
            .iter()
            .max_by_key(|c| c.borrow().accessed_at)
            .map(|c| c.borrow().id)
    }

    pub fn load_model(&mut self, file: &File) {
        self.cancel_chat_streaming();

        if let Some(loader) = &self.model_loader {
            if !loader.complete {
                return;
            }
        }

        let loader = ModelLoader::new(file.clone());
        loader.load_model(self.backend.as_ref());
        self.model_loader = Some(loader);
    }

    pub fn get_currently_loading_model(&self) -> Option<&File> {
        self.model_loader
            .as_ref()
            .filter(|loader| !loader.complete)
            .map(|loader| &loader.file)
    }

    pub fn update_load_model(&mut self) {
        let loader = self.model_loader.as_mut();
        if let Some(loader) = loader {
            if loader.check_load_response().is_ok() {
                self.loaded_model = Some(loader.file.clone());
            }

            if loader.complete {
                self.model_loader = None;
            }
        }
    }

    pub fn get_current_chat_id(&self) -> Option<ChatID> {
        self.current_chat_id
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

    pub fn get_chat_by_id(&self, chat_id: ChatID) -> Option<&RefCell<Chat>> {
        self.saved_chats.iter().find(|c| c.borrow().id == chat_id)
    }

    pub fn set_current_chat(&mut self, chat_id: ChatID) {
        self.cancel_chat_streaming();
        self.current_chat_id = Some(chat_id);

        let mut chat = self.get_current_chat().unwrap().borrow_mut();
        chat.update_accessed_at();
        chat.save();
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        let Some(loaded_model) = self.loaded_model.as_ref() else {
            println!("Skip sending message because loaded model not found");
            return;
        };

        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut()
                .send_message_to_model(prompt, loaded_model, self.backend.as_ref());
            chat.borrow().save();
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().cancel_streaming(self.backend.as_ref());
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
            chat.borrow().save();
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
                if let Some(loaded_model) = self.loaded_model.as_ref() {
                    if chat.is_streaming {
                        chat.cancel_streaming(self.backend.as_ref());
                    }

                    chat.remove_messages_from(message_id);
                    chat.send_message_to_model(
                        updated_message,
                        loaded_model,
                        self.backend.as_ref(),
                    );
                }
            } else {
                chat.edit_message(message_id, updated_message);
            }
            chat.save();
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

        self.loaded_model = None;
        Ok(())
    }

    pub fn create_empty_chat(&mut self) {
        let new_chat = RefCell::new(Chat::new(self.chats_dir.clone()));

        new_chat.borrow().save();

        self.current_chat_id = Some(new_chat.borrow().id);
        self.saved_chats.push(new_chat);
    }

    pub fn create_empty_chat_and_load_file(&mut self, file: &File) {
        let new_chat = RefCell::new(Chat::new(self.chats_dir.clone()));
        new_chat.borrow().save();

        self.cancel_chat_streaming();

        self.current_chat_id = Some(new_chat.borrow().id);
        self.saved_chats.push(new_chat);

        if self
            .loaded_model
            .as_ref()
            .map_or(true, |m| *m.id != file.id)
        {
            let _ = self.load_model(file);
        }
    }

    pub fn remove_chat(&mut self, chat_id: ChatID) {
        if let Some(chat) = self.saved_chats.iter().find(|c| c.borrow().id == chat_id) {
            chat.borrow().remove_saved_file();
        };
        self.saved_chats.retain(|c| c.borrow().id != chat_id);

        if let Some(current_chat_id) = self.current_chat_id {
            if current_chat_id == chat_id {
                self.current_chat_id = self.get_last_selected_chat_id();
            }
        }
    }
}
