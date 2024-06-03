use super::filesystem::{project_dirs, setup_model_downloads_folder};
use super::preferences::Preferences;
use super::search::SortCriteria;
use super::{chat::Chat, chat::ChatID, downloads::Downloads, search::Search};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use makepad_widgets::{DefaultNone, SignalToUI};
use moxin_backend::Backend;
use moxin_protocol::data::{Author, File, FileID, Model, ModelID, PendingDownload};
use moxin_protocol::protocol::{Command, LoadModelOptions, LoadModelResponse};
use std::rc::Rc;
use std::{cell::RefCell, path::PathBuf, sync::mpsc::channel};

pub const DEFAULT_MAX_DOWNLOAD_THREADS: usize = 3;

#[derive(Clone, DefaultNone, Debug)]
pub enum StoreAction {
    Search(String),
    ResetSearch,
    Sort(SortCriteria),
    None,
}

#[derive(Clone, Debug)]
pub struct FileWithDownloadInfo {
    pub file: File,
    pub download: Option<PendingDownload>,
    pub is_current_chat: bool,
}

#[derive(Clone, Debug)]
pub struct ModelWithDownloadInfo {
    pub model_id: ModelID,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    pub author: Author,
    pub like_count: u32,
    pub download_count: u32,
    pub files: Vec<FileWithDownloadInfo>,
}

#[derive(Default)]
pub struct Store {
    /// This is the backend representation, including the sender and receiver ends of the channels to
    /// communicate with the backend thread.
    pub backend: Rc<Backend>,

    pub search: Search,
    pub downloads: Downloads,

    /// Locally saved chats
    pub saved_chats: Vec<RefCell<Chat>>,
    pub current_chat_id: Option<ChatID>,

    pub preferences: Preferences,
    pub downloaded_files_dir: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let downloaded_files_dir = setup_model_downloads_folder();
        let app_data_dir = project_dirs().data_dir();

        let backend = Rc::new(Backend::new(
            app_data_dir,
            downloaded_files_dir.clone(),
            DEFAULT_MAX_DOWNLOAD_THREADS,
        ));

        let mut store = Self {
            backend: backend.clone(),

            search: Search::new(backend.clone()),
            downloads: Downloads::new(backend),

            saved_chats: vec![],
            current_chat_id: None,

            preferences: Preferences::load(),
            downloaded_files_dir,
        };
        store.downloads.load_downloaded_files();
        store.downloads.load_pending_downloads();

        store.search.load_featured_models();
        store
    }

    /// This function combines the search results information for a given model
    /// with the download information for the files of that model.
    pub fn add_download_info_to_model(&self, model: &Model) -> ModelWithDownloadInfo {
        let files = model
            .files
            .iter()
            .map(|file| {
                let download = self
                    .downloads
                    .pending_downloads
                    .iter()
                    .find(|d| d.file.id == file.id)
                    .cloned();
                let is_current_chat = self
                    .get_current_chat()
                    .map_or(false, |c| c.borrow().file_id == file.id);

                FileWithDownloadInfo {
                    file: file.clone(),
                    download,
                    is_current_chat,
                }
            })
            .collect();

        ModelWithDownloadInfo {
            model_id: model.id.clone(),
            name: model.name.clone(),
            summary: model.summary.clone(),
            size: model.size.clone(),
            requires: model.requires.clone(),
            architecture: model.architecture.clone(),
            like_count: model.like_count,
            download_count: model.download_count,
            released_at: model.released_at,
            author: model.author.clone(),
            files: files,
        }
    }

    pub fn get_model_and_file_download(&self, file_id: &str) -> (Model, File) {
        if let Some(result) = self
            .downloads
            .get_model_and_file_for_pending_download(file_id)
        {
            result
        } else {
            self.search
                .get_model_and_file_from_search_results(file_id)
                .unwrap()
        }
    }

    pub fn delete_file(&mut self, file_id: FileID) -> Result<()> {
        self.downloads.delete_file(file_id.clone())?;
        self.search
            .update_downloaded_file_in_search_results(&file_id, false);
        SignalToUI::set_ui_signal();
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // Functions related with model's chat                                           //
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn load_model(&mut self, file: &File) {
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
                        return;
                    };
                    // TODO: Creating a new chat, maybe put in a method and save on disk or smth.
                    let new_chat = RefCell::new(Chat::new(file.name.clone(), file.id.clone()));
                    self.current_chat_id = Some(new_chat.borrow().id);
                    self.saved_chats.push(new_chat);

                    self.preferences.set_current_chat_model(file.id.clone());
                }
                Err(err) => eprintln!("Error loading model: {:?}", err),
            }
        };
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

    pub fn process_event_signal(&mut self) {
        self.update_downloads();
        self.update_chat_messages();
        self.update_search_results();
    }

    fn update_search_results(&mut self) {
        match self.search.process_results() {
            Ok(Some(models)) => {
                self.search.set_models(models);
            }
            Ok(None) => {
                // No results arrived, do nothing
            }
            Err(_) => {
                self.search.set_models(vec![]);
            }
        }
    }

    fn update_chat_messages(&mut self) {
        let Some(chat) = self.get_current_chat() else {
            return;
        };
        chat.borrow_mut().update_messages();
    }

    fn update_downloads(&mut self) {
        let completed_download_ids = self.downloads.refresh_downloads_data();

        // For search results let's trust on our local cache, but updating
        // the downloaded state of the files
        for file_id in completed_download_ids {
            self.search
                .update_downloaded_file_in_search_results(&file_id, true);
        }
    }
}
