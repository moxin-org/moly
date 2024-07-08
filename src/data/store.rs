use super::chats::chat::ChatID;
use super::filesystem::project_dirs;
use super::preferences::Preferences;
use super::search::SortCriteria;
use super::{chats::Chats, downloads::Downloads, search::Search};
use anyhow::Result;
use chrono::{DateTime, Utc};
use makepad_widgets::{DefaultNone, SignalToUI};
use moxin_backend::Backend;
use moxin_protocol::data::{Author, DownloadedFile, File, FileID, Model, ModelID, PendingDownload};
use std::rc::Rc;

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
pub struct Store {
    /// This is the backend representation, including the sender and receiver ends of the channels to
    /// communicate with the backend thread.
    pub backend: Rc<Backend>,

    pub search: Search,
    pub downloads: Downloads,
    pub chats: Chats,
    pub preferences: Preferences,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let preferences = Preferences::load();
        let app_data_dir = project_dirs().data_dir();

        let backend = Rc::new(Backend::new(
            app_data_dir,
            preferences.downloaded_files_dir.clone(),
            DEFAULT_MAX_DOWNLOAD_THREADS,
        ));

        let mut store = Self {
            backend: backend.clone(),
            search: Search::new(backend.clone()),
            downloads: Downloads::new(backend.clone()),
            chats: Chats::new(backend),
            preferences,
        };

        store.downloads.load_downloaded_files();
        store.downloads.load_pending_downloads();

        store.chats.load_chats();
        store.init_current_chat();

        store.search.load_featured_models();
        store
    }

    pub fn load_model(&mut self, file: &File) {
        if self.chats.load_model(file).is_ok() {
            self.preferences.set_current_chat_model(file.id.clone());

            // If there is no chat, create an empty one
            if self.chats.get_current_chat().is_none() {
                self.chats.create_empty_chat_with_model_file(file);
            }
        }
    }

    pub fn select_chat(&mut self, chat_id: ChatID) {
        self.chats.set_current_chat(chat_id);
    }

    pub fn get_loaded_model_file_id(&self) -> Option<FileID> {
        self.chats
            .get_current_chat()
            .map(|chat| chat.borrow().file_id.clone())
    }

    pub fn get_loaded_downloaded_file(&self) -> Option<DownloadedFile> {
        if let Some(file_id) = self.get_loaded_model_file_id() {
            self.downloads
                .downloaded_files
                .iter()
                .find(|d| d.file.id == file_id)
                .cloned()
        } else {
            None
        }
    }

    /// This function ensures that the model with the provided file ID is loaded
    /// in the current chat.
    /// It is necesary to cover cases where the model file is deleted by the user,
    /// but previous chat sessions refer to that file.
    pub fn ensure_model_loaded_in_current_chat(&mut self, file_id: FileID) {
        // Check if the current chat has loaded the provided file
        // Do nothing if this is the case
        if self
            .get_loaded_model_file_id()
            .map(|id| id == file_id)
            .unwrap_or(false)
        {
            return;
        }

        let Some(current_chat_id) = self.chats.get_current_chat_id() else {
            return;
        };

        // Attempt to load the model in the current chat since it is not loaded
        let available_files: Vec<File> = self
            .downloads
            .downloaded_files
            .iter()
            .map(|d| d.file.clone())
            .collect();

        if let Some(file) = available_files.iter().find(|file| file.id == file_id) {
            self.chats
                .set_current_chat_and_load_model(current_chat_id, file);
        }
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
                    .chats
                    .loaded_model
                    .clone()
                    .map_or(false, |loaded| loaded.id == file.id);

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
            files,
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
        let Some(chat) = self.chats.get_current_chat() else {
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

    fn init_current_chat(&mut self) {
        if let Some(ref file_id) = self.preferences.current_chat_model {
            let available_files: Vec<File> = self
                .downloads
                .downloaded_files
                .iter()
                .map(|d| d.file.clone())
                .collect();

            if let Some(file) = available_files.iter().find(|file| file.id == *file_id) {
                if let Some(chat_id) = self.chats.get_latest_chat_id() {
                    self.chats.set_current_chat_and_load_model(chat_id, file);
                } else {
                    self.chats.create_empty_chat_with_model_file(file);
                }

                return;
            }
        }

        if let Some(chat_id) = self.chats.get_latest_chat_id() {
            self.chats.set_current_chat(chat_id);
        }

        // If there is no chat, create an empty one
        if self.chats.get_current_chat().is_none() {
            self.chats.create_empty_chat();
        }
    }

    pub fn delete_chat(&mut self, chat_id: ChatID) {
        self.chats.remove_chat(chat_id);

        // TODO Decide proper behavior when deleting the current chat
        // For now, we just create a new empty chat because we don't fully
        // support having no chat selected
        self.init_current_chat();
    }
}
