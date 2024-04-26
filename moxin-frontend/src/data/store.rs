use super::preferences::Preferences;
use super::{chat::Chat, download::Download, search::Search};
use chrono::Utc;
use makepad_widgets::DefaultNone;
use moxin_backend::Backend;
use moxin_protocol::data::{
    DownloadedFile, File, FileID, Model, PendingDownload, PendingDownloadsStatus,
};
use moxin_protocol::protocol::{Command, LoadModelOptions, LoadModelResponse};
use std::collections::HashMap;
use std::sync::mpsc::channel;

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
pub enum DownloadInfoStatus {
    Downloading,
    Paused,
}

#[derive(Clone, Debug)]
pub struct DownloadInfo {
    pub file: File,
    pub model: Model,
    pub progress: f64,
    pub status: DownloadInfoStatus,
}

#[derive(Clone, Debug)]
pub struct ModelWithPendingDownloads {
    pub model: Model,
    pub pending_downloads: Vec<PendingDownload>,
}

#[derive(Default)]
pub struct Store {
    // This is the backend representation, including the sender and receiver ends of the channels to
    // communicate with the backend thread.
    pub backend: Backend,

    // Local cache of backend information
    pub models: Vec<Model>,
    pub downloaded_files: Vec<DownloadedFile>,
    pub pending_downloads: Vec<PendingDownload>,

    pub search: Search,
    pub sorted_by: SortCriteria,

    pub current_chat: Option<Chat>,
    pub current_downloads: HashMap<FileID, Download>,

    pub preferences: Preferences,
}

impl Store {
    pub fn new() -> Self {
        let mut store = Self {
            // Initialize the backend with the default values
            backend: Backend::default(),

            // Initialize the local cache with empty values
            models: vec![],

            // TODO we should unify those two into a single struct
            downloaded_files: vec![],
            pending_downloads: vec![],

            search: Search::new(),
            sorted_by: SortCriteria::MostDownloads,
            current_chat: None,
            current_downloads: HashMap::new(),

            preferences: Preferences::load(),
        };
        store.load_downloaded_files();
        store.load_pending_downloads();

        store.load_featured_models();

        store.sort_models(SortCriteria::MostDownloads);
        store
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
                }
                Err(err) => eprintln!("Error fetching pending downloads: {:?}", err),
            }
        };
    }

    pub fn download_file(&mut self, file: File, model: Model) {
        let mut current_progress = 0.0;
        if let Some(pending) = self.pending_downloads.iter().find(|d| d.file.id == file.id) {
            current_progress = pending.progress;
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
            file.id.clone(),
            Download::new(file.clone(), model.clone(), current_progress, &self.backend),
        );
    }

    pub fn pause_download_file(&mut self, file: File) {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::PauseDownload(file.id.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.current_downloads.remove(&file.id);
                    self.load_pending_downloads();
                }
                Err(err) => eprintln!("Error pausing download: {:?}", err),
            }
        };
    }

    pub fn cancel_download_file(&mut self, file: File) {
        let (tx, rx) = channel();
        self.backend
            .command_sender
            .send(Command::CancelDownload(file.id.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.current_downloads.remove(&file.id);
                    self.pending_downloads.retain(|d| d.file.id != file.id);
                    self.load_pending_downloads();
                }
                Err(err) => eprintln!("Error cancelling download: {:?}", err),
            }
        };
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
                    self.current_chat = Some(Chat::new(file.name.clone()));
                    self.preferences.set_current_chat_model(file.id.clone());
                }
                Err(err) => eprintln!("Error loading model: {:?}", err),
            }
        };
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        if let Some(chat) = &mut self.current_chat {
            chat.send_message_to_model(prompt, &self.backend);
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = &mut self.current_chat {
            chat.cancel_streaming(&self.backend);
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = &mut self.current_chat {
            chat.delete_message(message_id);
        }
    }

    pub fn edit_chat_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(chat) = &mut self.current_chat {
            chat.edit_message(message_id, updated_message);
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
                        new_model.size = rng.gen_range(100000000..999999999).to_string();
                    };

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
        if let Ok(models) = self.search.process_results(&self.backend) {
            self.set_models(models);
            self.sort_models_by_current_criteria();
        }
    }

    fn update_chat_messages(&mut self) {
        let Some(ref mut chat) = self.current_chat else {
            return;
        };
        chat.update_messages();
    }

    fn update_downloads(&mut self) {
        let mut completed_downloads = Vec::new();

        for (id, download) in &mut self.current_downloads {
            download.process_download_progress();
            if download.done {
                completed_downloads.push(id.clone());
            }
        }

        if !completed_downloads.is_empty() {
            // Reload downloaded files
            self.load_downloaded_files();
            self.load_pending_downloads();
        }

        for id in completed_downloads {
            self.current_downloads.remove(&id);
            self.mark_file_as_downloaded(&id);
        }
    }

    fn mark_file_as_downloaded(&mut self, file_id: &FileID) {
        let model = self
            .models
            .iter_mut()
            .find(|m| m.files.iter().any(|f| f.id == *file_id));
        if let Some(model) = model {
            let file = model.files.iter_mut().find(|f| f.id == *file_id).unwrap();
            file.downloaded = true;
        }
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

    fn sort_models_by_current_criteria(&mut self) {
        self.sort_models(self.sorted_by);
    }

    pub fn formatted_model_release_date(model: &Model) -> String {
        let released_at = model.released_at.naive_local().format("%b %-d, %C%y");
        let days_ago = (Utc::now() - model.released_at).num_days();
        format!("{} ({} days ago)", released_at, days_ago)
    }

    pub fn model_featured_files(model: &Model) -> Vec<File> {
        model.files.iter().filter(|f| f.featured).cloned().collect()
    }

    pub fn model_other_files(model: &Model) -> Vec<File> {
        model
            .files
            .iter()
            .filter(|f| !f.featured)
            .cloned()
            .collect()
    }

    pub fn current_downloads_info(&self) -> Vec<DownloadInfo> {
        // Collect information about current downloads
        let mut results: Vec<DownloadInfo> = self
            .current_downloads
            .iter()
            .map(|(_id, download)| DownloadInfo {
                file: download.file.clone(),
                model: download.model.clone(),
                progress: download.progress,
                status: DownloadInfoStatus::Downloading,
            })
            .collect();

        // Add files that are still partially downloaded (from previous sessions with the app)
        let mut partial_downloads: Vec<DownloadInfo> = self
            .pending_downloads
            .iter()
            .filter(|f| !self.current_downloads.contains_key(&f.file.id))
            .map(|d| DownloadInfo {
                file: d.file.clone(),
                model: d.model.clone(),
                progress: d.progress,

                // TODO: Handle errors and other statuses
                status: DownloadInfoStatus::Paused,
            })
            .collect();

        results.append(&mut partial_downloads);
        results
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

        Some(ModelWithPendingDownloads {
            model: model.clone(),
            pending_downloads,
        })
    }
}
