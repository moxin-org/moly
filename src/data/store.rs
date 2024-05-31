use super::chat::ChatID;
use super::download::DownloadState;
use super::filesystem::{project_dirs, setup_model_downloads_folder};
use super::preferences::Preferences;
use super::{chat::Chat, download::Download, search::Search};
use anyhow::{Context, Result};
use chrono::Utc;
use makepad_widgets::{DefaultNone, SignalToUI};
use moxin_backend::Backend;
use moxin_protocol::data::{
    DownloadedFile, File, FileID, Model, PendingDownload, PendingDownloadsStatus,
};
use moxin_protocol::protocol::{Command, LoadModelOptions, LoadModelResponse};
use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    sync::mpsc::channel,
};

pub const DEFAULT_MAX_DOWNLOAD_THREADS: usize = 3;

#[derive(Clone, DefaultNone, Debug)]
pub enum StoreAction {
    Search(String),
    ResetSearch,
    Sort(SortCriteria),
    None,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SortCriteria {
    #[default]
    MostDownloads,
    LeastDownloads,
    MostLikes,
    LeastLikes,
}

#[derive(Clone, Debug)]
pub struct ModelWithPendingDownloads {
    pub model: Model,
    pub pending_downloads: Vec<PendingDownload>,
    pub current_file_id: Option<FileID>,
}

pub enum DownloadPendingNotification {
    DownloadedFile(File),
    DownloadErrored(File),
}

#[derive(Default)]
pub struct Store {
    /// This is the backend representation, including the sender and receiver ends of the channels to
    /// communicate with the backend thread.
    pub backend: Backend,

    /// Local cache of search results and downloaded files
    pub models: Vec<Model>,
    pub downloaded_files: Vec<DownloadedFile>,
    pub pending_downloads: Vec<PendingDownload>,

    pub search: Search,
    pub sorted_by: SortCriteria,

    /// Locally saved chats
    pub saved_chats: Vec<RefCell<Chat>>,
    pub current_chat_id: Option<ChatID>,
    pub current_downloads: HashMap<FileID, Download>,

    pub preferences: Preferences,
    pub downloaded_files_dir: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let downloaded_files_dir = setup_model_downloads_folder();
        let app_data_dir = project_dirs().data_dir();

        let backend = Backend::new(
            app_data_dir,
            downloaded_files_dir.clone(),
            DEFAULT_MAX_DOWNLOAD_THREADS,
        );
        let mut store = Self {
            backend,
            // Initialize the local cache with empty values
            models: vec![],

            // TODO we should unify those two into a single struct
            downloaded_files: vec![],
            pending_downloads: vec![],

            search: Search::new(),
            sorted_by: SortCriteria::MostDownloads,
            saved_chats: vec![],
            current_chat_id: None,
            current_downloads: HashMap::new(),

            preferences: Preferences::load(),
            downloaded_files_dir,
        };
        store.load_downloaded_files();
        store.load_pending_downloads();

        store.load_featured_models();

        store.sort_models(SortCriteria::MostDownloads);
        store
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

    // Commands to the backend

    pub fn load_featured_models(&mut self) {
        self.search.load_featured_models(&self.backend);
    }

    pub fn load_search_results(&mut self, query: String) {
        self.search.run_or_enqueue(query.clone(), &self.backend);
    }

    pub fn load_downloaded_files(&mut self) {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::GetDownloadedFiles(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(files) => {
                    self.downloaded_files = files;
                }
                Err(err) => eprintln!("Error fetching downloaded files: {:?}", err),
            }
        };
    }

    pub fn load_pending_downloads(&mut self) {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::GetCurrentDownloads(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(files) => {
                    self.pending_downloads = files;

                    // There is a issue with the backend response where all pending
                    // downloads come with status `Paused` even if they are downloading.
                    self.pending_downloads.iter_mut().for_each(|d| {
                        if self.current_downloads.contains_key(&d.file.id) {
                            d.status = PendingDownloadsStatus::Downloading;
                        }
                    });
                }
                Err(err) => eprintln!("Error fetching pending downloads: {:?}", err),
            }
        };
    }

    fn get_model_and_file_download(&self, file_id: &str) -> (Model, File) {
        if let Some(result) = self.get_model_and_file_for_pending_download(file_id) {
            result
        } else {
            self.get_model_and_file_from_search_results(file_id)
                .unwrap()
        }
    }

    fn get_model_and_file_from_search_results(&self, file_id: &str) -> Option<(Model, File)> {
        self.models.iter().find_map(|m| {
            m.files
                .iter()
                .find(|f| f.id == file_id)
                .map(|f| (m.clone(), f.clone()))
        })
    }

    fn get_model_and_file_for_pending_download(&self, file_id: &str) -> Option<(Model, File)> {
        self.pending_downloads.iter().find_map(|d| {
            if d.file.id == file_id {
                Some((d.model.clone(), d.file.clone()))
            } else {
                None
            }
        })
    }

    pub fn download_file(&mut self, file_id: FileID) {
        let (model, file) = self.get_model_and_file_download(&file_id);
        let mut current_progress = 0.0;

        if let Some(pending) = self
            .pending_downloads
            .iter_mut()
            .find(|d| d.file.id == file_id)
        {
            current_progress = pending.progress;
            pending.status = PendingDownloadsStatus::Downloading;
        } else {
            let pending_download = PendingDownload {
                file: file.clone(),
                model: model.clone(),
                progress: 0.0,
                status: PendingDownloadsStatus::Downloading,
            };
            self.pending_downloads.push(pending_download);
        }

        self.current_downloads.insert(
            file_id.clone(),
            Download::new(file, model, current_progress, &self.backend),
        );
    }

    pub fn pause_download_file(&mut self, file_id: FileID) {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::PauseDownload(file_id.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.current_downloads.remove(&file_id);
                    self.load_pending_downloads();
                }
                Err(err) => eprintln!("Error pausing download: {:?}", err),
            }
        };
    }

    pub fn cancel_download_file(&mut self, file_id: FileID) {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::CancelDownload(file_id.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.current_downloads.remove(&file_id);
                    self.load_pending_downloads();
                }
                Err(err) => eprintln!("Error cancelling download: {:?}", err),
            }
        };
    }

    pub fn eject_model(&mut self) -> Result<()> {
        let (tx, rx) = channel();
        self.backend
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

    pub fn delete_file(&mut self, file_id: FileID) -> Result<()> {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::DeleteFile(file_id.clone(), tx))
            .context("Failed to send delete file command")?;

        rx.recv()
            .context("Failed to receive delete file response")?
            .context("Delete file operation failed")?;

        self.update_downloaded_file_in_search_results(&file_id, false);
        self.load_downloaded_files();
        self.load_pending_downloads();
        SignalToUI::set_ui_signal();
        Ok(())
    }

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

        self.backend.command_sender.send(cmd).unwrap();

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

    pub fn send_chat_message(&mut self, prompt: String) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut()
                .send_message_to_model(prompt, &self.backend);
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().cancel_streaming(&self.backend);
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
                    chat.cancel_streaming(&self.backend);
                }

                chat.remove_messages_from(message_id);
                chat.send_message_to_model(updated_message, &self.backend);
            } else {
                chat.edit_message(message_id, updated_message);
            }
        }
    }

    pub fn process_event_signal(&mut self) {
        self.update_downloads();
        self.update_chat_messages();
        self.update_search_results();
    }

    fn set_models(&mut self, models: Vec<Model>) {
        #[cfg(not(debug_assertions))]
        {
            self.models = models;
        }
        #[cfg(debug_assertions)]
        'debug_block: {
            use lipsum::lipsum;
            use rand::distributions::{Alphanumeric, DistString};
            use rand::Rng;
            let mut rng = rand::thread_rng();
            fn random_string(size: usize) -> String {
                Alphanumeric.sample_string(&mut rand::thread_rng(), size)
            }

            let fill_fake_data = std::env::var("FILL_FAKE_DATA").is_ok_and(|fill_fake_data| {
                ["true", "t", "1"].iter().any(|&s| s == fill_fake_data)
            });

            if !fill_fake_data {
                self.models = models;
                break 'debug_block;
            }

            let faked_models: Vec<Model> = models
                .iter()
                .map(|model| {
                    // filling model attributes
                    let mut new_model = model.clone();
                    if model.summary.is_empty() {
                        new_model.summary = lipsum(30);
                    }

                    if model.name.is_empty() {
                        // we might need a fancier word generator
                        new_model.name = format!(
                            "{}-{}-{}{}-{}-{}",
                            lipsum(1),
                            rng.gen_range(0..10),
                            random_string(1).to_uppercase(),
                            rng.gen_range(0..10),
                            lipsum(1),
                            lipsum(1),
                        );
                    }

                    if model.size.is_empty() {
                        new_model.size = format!("{}B", rng.gen_range(1..10));
                    };

                    if model.requires.is_empty() {
                        new_model.requires = match rng.gen_range(0..3) {
                            0 => "4GB+ RAM".to_string(),
                            1 => "8GB+ RAM".to_string(),
                            2 => "16GB+ RAM".to_string(),
                            _ => "32GB+ RAM".to_string(),
                        };
                    }

                    if model.architecture.is_empty() {
                        new_model.architecture = match rng.gen_range(0..3) {
                            0 => "Mistral".to_string(),
                            1 => "StableLM".to_string(),
                            2 => "LlaMa".to_string(),
                            _ => "qwen2".to_string(),
                        };
                    }

                    if model.like_count == 0 {
                        new_model.like_count = rng.gen_range(1..1000);
                    };

                    if model.download_count == 0 {
                        new_model.download_count = rng.gen_range(0..10000);
                    };

                    // filling files
                    let new_files: Vec<File> = model
                        .files
                        .iter()
                        .map(|file| {
                            let mut new_file = file.clone();

                            if new_file.quantization.is_empty() {
                                new_file.quantization = format!(
                                    "Q{}_{}_{}",
                                    rng.gen_range(0..10),
                                    random_string(1).to_uppercase(),
                                    random_string(1).to_uppercase()
                                );
                            }

                            if file.name.is_empty() {
                                // we might need a fancier word generator
                                new_file.name = format!(
                                    "{}-{}-{}-{}-{}.{}.gguf",
                                    lipsum(1),
                                    rng.gen_range(0..10),
                                    random_string(5),
                                    lipsum(1),
                                    new_file.quantization,
                                    rng.gen_range(0..10),
                                );
                            }

                            if file.size.is_empty() {
                                new_file.size = rng.gen_range(100000000..999999999).to_string();
                            };

                            new_file
                        })
                        .collect();

                    new_model.files = new_files;
                    new_model
                })
                .collect();
            self.models = faked_models;
        }
    }

    fn update_search_results(&mut self) {
        match self.search.process_results(&self.backend) {
            Ok(Some(models)) => {
                self.set_models(models);
                self.sort_models(self.sorted_by);
            }
            Ok(None) => {
                // No results arrived, do nothing
            }
            Err(_) => {
                self.set_models(vec![]);
            }
        }
    }

    fn update_chat_messages(&mut self) {
        let Some(chat) = self.get_current_chat() else {
            return;
        };
        chat.borrow_mut().update_messages();
    }

    fn update_downloaded_file_in_search_results(&mut self, file_id: &FileID, downloaded: bool) {
        let model = self
            .models
            .iter_mut()
            .find(|m| m.files.iter().any(|f| f.id == *file_id));
        if let Some(model) = model {
            let file = model.files.iter_mut().find(|f| f.id == *file_id).unwrap();
            file.downloaded = downloaded;
        }
    }

    fn update_downloads(&mut self) {
        let mut completed_download_ids = Vec::new();

        for (id, download) in &mut self.current_downloads {
            if let Some(pending) = self
                .pending_downloads
                .iter_mut()
                .find(|d| d.file.id == id.to_string())
            {
                match download.state {
                    DownloadState::Downloading(_) => {
                        pending.status = PendingDownloadsStatus::Downloading
                    }
                    DownloadState::Errored(_) => pending.status = PendingDownloadsStatus::Error,
                    DownloadState::Completed => (),
                };
                pending.progress = download.get_progress();
            }

            download.process_download_progress();
            if download.is_complete() {
                completed_download_ids.push(id.clone());
            }
        }

        // Reload downloaded files and pending downloads from the backend
        if !completed_download_ids.is_empty() {
            self.load_downloaded_files();
            self.load_pending_downloads();
        }

        // For search results let's trust on our local cache, but updating
        // the downloaded state of the files
        for file_id in completed_download_ids {
            self.update_downloaded_file_in_search_results(&file_id, true);
        }
    }

    pub fn next_download_notification(&mut self) -> Option<DownloadPendingNotification> {
        self.current_downloads
            .iter_mut()
            .filter_map(|(_, download)| {
                if download.must_show_notification() {
                    if download.is_errored() {
                        return Some(DownloadPendingNotification::DownloadErrored(
                            download.file.clone(),
                        ));
                    } else if download.is_complete() {
                        return Some(DownloadPendingNotification::DownloadedFile(
                            download.file.clone(),
                        ));
                    } else {
                        return None;
                    }
                }
                None
            })
            .next()
    }

    // Utility functions

    pub fn sort_models(&mut self, criteria: SortCriteria) {
        match criteria {
            SortCriteria::MostDownloads => {
                self.models
                    .sort_by(|a, b| b.download_count.cmp(&a.download_count));
            }
            SortCriteria::LeastDownloads => {
                self.models
                    .sort_by(|a, b| a.download_count.cmp(&b.download_count));
            }
            SortCriteria::MostLikes => {
                self.models.sort_by(|a, b| b.like_count.cmp(&a.like_count));
            }
            SortCriteria::LeastLikes => {
                self.models.sort_by(|a, b| a.like_count.cmp(&b.like_count));
            }
        }
        self.sorted_by = criteria;
    }

    pub fn get_model_with_pending_downloads(
        &self,
        model_id: &str,
    ) -> Option<ModelWithPendingDownloads> {
        let model = self.models.iter().find(|m| m.id == model_id)?;
        let pending_downloads = self
            .pending_downloads
            .iter()
            .filter(|d| d.model.id == model_id)
            .cloned()
            .collect();
        let current_file_id = model
            .files
            .iter()
            .find(|f| {
                self.get_current_chat()
                    .as_ref()
                    .map_or(false, |c| c.borrow().file_id == f.id)
            })
            .map(|f| f.id.clone());

        Some(ModelWithPendingDownloads {
            model: model.clone(),
            pending_downloads,
            current_file_id,
        })
    }

    pub fn search_is_loading(&self) -> bool {
        self.search.is_pending()
    }

    pub fn search_is_errored(&self) -> bool {
        self.search.was_error()
    }
}
