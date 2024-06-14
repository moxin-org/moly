pub mod chat;

use anyhow::{Context, Result};
use chat::{Chat, ChatID};
use moxin_backend::Backend;
use moxin_protocol::protocol::{Command, LoadModelResponse};
use moxin_protocol::{data::*, protocol::LoadModelOptions};
use std::fs;
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::mpsc::channel};

use super::filesystem::setup_chats_folder;

pub struct Chats {
    pub backend: Rc<Backend>,
    pub saved_chats: Vec<RefCell<Chat>>,

    current_chat_id: Option<ChatID>,
    loaded_model_id: Option<FileID>,
    chats_dir: PathBuf,
}

impl Chats {
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

    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            saved_chats: Vec::new(),
            current_chat_id: None,
            loaded_model_id: None,
            chats_dir: setup_chats_folder(),
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
                    if let LoadModelResponse::Completed(_) = response {
                        self.loaded_model_id = Some(file.id.clone());
                        Ok(())
                    } else {
                        eprintln!("Error loading model");
                        Err(anyhow::anyhow!("Error loading model"))
                    }
                }
                Err(err) => {
                    eprintln!("Error loading model: {:?}", err);
                    Err(err)
                }
            }
        } else {
            Err(anyhow::anyhow!("Error loading model"))
        }
    }

    pub fn create_new_chat(&mut self, file: &File, set_as_current: bool) -> Result<()> {
        self.load_model(file)?;

        let new_chat = RefCell::new(Chat::new(
            file.name.clone(),
            file.id.clone(),
            self.chats_dir.clone(),
        ));

        new_chat.borrow().save();

        if set_as_current {
            self.current_chat_id = Some(new_chat.borrow().id);
        }

        self.saved_chats.push(new_chat);

        Ok(())
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

    pub fn get_current_chat_model_file<'a>(&'a self, available_model_files: &'a [File]) -> &File {
        let file_id = self.get_current_chat().unwrap().borrow().file_id.clone();
        let file = available_model_files
            .iter()
            .find(|file| file.id == file_id)
            .expect("Attempted to start chat with a no longer existing file");

        file
    }

    pub fn set_current_chat(
        &mut self,
        chat_id: ChatID,
        available_model_files: &[File],
    ) -> Result<()> {
        self.current_chat_id = Some(chat_id);

        let file = self
            .get_current_chat_model_file(available_model_files)
            .clone();

        if self.loaded_model_id.as_ref().is_some_and(|m| *m != file.id) {
            self.load_model(&file)?;
        }

        Ok(())
    }

    pub fn remove_current_chat(&mut self) {
        self.current_chat_id = None;
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut()
                .send_message_to_model(prompt, self.backend.as_ref());
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
                if chat.is_streaming {
                    chat.cancel_streaming(self.backend.as_ref());
                }

                chat.remove_messages_from(message_id);
                chat.send_message_to_model(updated_message, self.backend.as_ref());
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

        self.remove_current_chat();
        Ok(())
    }
}
