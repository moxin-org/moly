use chrono::Utc;
use moxin_protocol::data::{Model, File};
use moxin_protocol::protocol::{Command, LoadModelOptions};
use moxin_backend::Backend;
use std::sync::mpsc::channel;
use makepad_widgets::DefaultNone;
use crate::data::chat::Chat;

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

    pub keyword: Option<String>,
    pub sorted_by: SortCriteria,

    pub current_chat: Option<Chat>,
}

impl Store {
    pub fn new() -> Self {
        let store = Self {
            models: vec![],
            backend: Backend::default(),
            keyword: None,
            sorted_by: SortCriteria::MostDownloads,
            current_chat: None,
        };
        //store.load_featured_models();
        //store.sort_models(SortCriteria::MostDownloads);
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
        let (tx, rx) = channel();
        self
            .backend
            .command_sender
            .send(Command::SearchModels(query.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(models) => {
                    self.models = models;
                    self.keyword = Some(query.clone());
                },
                Err(err) => eprintln!("Error fetching models: {:?}", err),
            }
        };
    }

    pub fn load_model(&mut self) {
        let (tx, rx) = channel();
        let cmd = Command::LoadModel(
            "llama-2-7b-chat.Q4_K_M".to_string(),
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
                    makepad_widgets::log!("Model loaded");
                    dbg!(response);
                },
                Err(err) => eprintln!("Error loading model: {:?}", err),
            }
        };
    }

    // Chat specific commands

    pub fn send_chat_message(&mut self, prompt: String) {
        let chat = &mut self.current_chat.get_or_insert(Chat::new());
        chat.send_message_to_model(prompt, &self.backend);
    }

    pub fn update_chat_messages(&mut self) {
        let Some(ref mut chat) = self.current_chat else { return };
        chat.update_messages();
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
        let released_at = model.released_at.format("%b %-d, %C%y");
        let days_ago = (Utc::now().date_naive() - model.released_at).num_days();
        format!("{} ({} days ago)", released_at, days_ago)
    }

    pub fn model_featured_files(model: &Model) -> Vec<File> {
        model.files.iter().filter(|f| f.featured).cloned().collect()
    }

    pub fn model_other_files(model: &Model) -> Vec<File> {
        model.files.iter().filter(|f| !f.featured).cloned().collect()
    }
}