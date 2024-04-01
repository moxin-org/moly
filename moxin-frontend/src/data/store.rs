use chrono::{Utc, DateTime, TimeZone};
use moxin_protocol::data::{Model, File, FileID, DownloadedFile, CompatibilityGuess};
use moxin_protocol::protocol::{Command, LoadModelOptions, LoadModelResponse, FileDownloadResponse};
use moxin_backend::Backend;
use std::sync::mpsc::channel;
use makepad_widgets::DefaultNone;
use crate::data::{
    chat::Chat,
    download::Download,
    search::Search,
};
use std::collections::HashMap;

#[derive(Clone, DefaultNone, Debug)]
pub enum StoreAction {
    Search(String),
    ResetSearch,
    Sort(SortCriteria),
    None,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SortCriteria {
    #[default] MostDownloads,
    LeastDownloads,
    MostLikes,
    LeastLikes,
}

#[derive(Default)]
pub struct Store {
    // This is the backend representation, including the sender and receiver ends of the channels to
    // communicate with the backend thread.
    pub backend: Backend,

    // Local cache for the list of models
    pub models: Vec<Model>,
    pub downloaded_files: Vec<DownloadedFile>,

    pub search: Search,

    pub keyword: Option<String>,
    pub sorted_by: SortCriteria,

    pub current_chat: Option<Chat>,
    pub current_downloads: HashMap<FileID, Download>,

    
}

impl Store {
    pub fn new() -> Self {
        let mut store = Self {
            models: vec![],
            backend: Backend::default(),
            search: Search::new(),
            keyword: None,
            sorted_by: SortCriteria::MostDownloads,
            current_chat: None,
            current_downloads: HashMap::new(),
            downloaded_files: vec![],
        };
        store.load_downloaded_files();
        store.load_featured_models();
        store.sort_models(SortCriteria::MostDownloads);
        store
    }

    // Commands to the backend

    pub fn load_featured_models(&mut self) {
        let (tx, rx) = channel();
        self
            .backend
            .command_sender
            .send(Command::GetFeaturedModels(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(models) => {
                    self.models = models;
                    self.keyword = None;
                },
                Err(err) => eprintln!("Error fetching models: {:?}", err),
            }
        };
    }

    pub fn load_search_results(&mut self, query: String) {
        self.search.run_or_enqueue(query.clone(), &self.backend);
        // TODO remove this
        self.keyword = Some(query);
    }

    pub fn load_downloaded_files(&mut self) {
        let (tx, rx) = channel();
        self
            .backend
            .command_sender
            .send(Command::GetDownloadedFiles(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(files) => {
                    self.downloaded_files = files;
                },
                Err(err) => eprintln!("Error fetching downloaded files: {:?}", err),
            }
        };
    }

    pub fn download_file(&mut self, file: &File) {
        self.current_downloads.insert(file.id.clone(), Download::new(file.clone(), &self.backend));
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
                context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            },
            tx,
        );

        self
            .backend
            .command_sender
            .send(cmd)
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(response) => {
                    let LoadModelResponse::Completed(loaded_model) = response else {
                        eprintln!("Error loading model");
                        return;
                    };
                    self.current_chat = Some(Chat::new(loaded_model.file_id.clone()));
                },
                Err(err) => eprintln!("Error loading model: {:?}", err),
            }
        };
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        if let Some(chat) = &mut self.current_chat {
            chat.send_message_to_model(prompt, &self.backend);
        }
        // TODO: Handle error case
    }

    pub fn process_event_signal(&mut self) {
        self.update_downloads();
        self.update_chat_messages();
        self.update_search_results();
    }

    fn update_search_results(&mut self) {
        if let Ok(models) = self.search.process_results(&self.backend) {
            self.models = models;
        }
    }

    fn update_chat_messages(&mut self) {
        let Some(ref mut chat) = self.current_chat else { return };
        chat.update_messages();
    }

    fn update_downloads(&mut self) {
        for download in self.current_downloads.values_mut() {
            download.update_download_progress();
        }
    }



    // Utility functions

    pub fn sort_models(&mut self, criteria: SortCriteria) {
        match criteria {
            SortCriteria::MostDownloads => {
                self.models.sort_by(|a, b| b.download_count.cmp(&a.download_count));
            }
            SortCriteria::LeastDownloads => {
                self.models.sort_by(|a, b| a.download_count.cmp(&b.download_count));
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

    pub fn formatted_model_release_date(model: &Model) -> String {
        let released_at = model.released_at.naive_local().format("%b %-d, %C%y");
        let days_ago = (Utc::now() - model.released_at).num_days();
        format!("{} ({} days ago)", released_at, days_ago)
    }

    pub fn model_featured_files(model: &Model) -> Vec<File> {
        model.files.iter().filter(|f| f.featured).cloned().collect()
    }

    pub fn model_other_files(model: &Model) -> Vec<File> {
        model.files.iter().filter(|f| !f.featured).cloned().collect()
    }
}