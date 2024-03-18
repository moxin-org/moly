use chrono::Utc;
use moxin_protocol::data::{Model, File};
use moxin_protocol::protocol::Command;
use moxin_backend::Backend;
use std::sync::mpsc::channel;
use makepad_widgets::DefaultNone;

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
}

impl Store {
    pub fn new() -> Self {
        let mut store = Self {
            models: vec![],
            backend: Backend::default(),
            keyword: None,
            sorted_by: SortCriteria::MostDownloads,
        };
        store.load_featured_models();
        store.sort_models(SortCriteria::MostDownloads);
        store
    }

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