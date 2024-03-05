use chrono::Utc;
use moxin_protocol::data::{Model, File};
use moxin_protocol::protocol::Command;
use moxin_backend::Backend;
use std::sync::mpsc::channel;

#[derive(Default)]
pub struct Store {
    // This is the backend representation, including the sender and receiver ends of the channels to
    // communicate with the backend thread.
    pub backend: Backend,

    // Local cache for the list of models
    pub models: Vec<Model>,
}

impl Store {
    pub fn new() -> Self {
        let mut store = Self {
            models: vec![],
            backend: Backend::default(),
        };

        let (tx, rx) = channel();
        store
            .backend
            .command_sender
            .send(Command::GetFeaturedModels(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(models) => store.models = models,
                Err(err) => eprintln!("Error fetching models: {:?}", err),
            }
        };

        store
    }

    pub fn get_model_by_id(&self, id: &str) -> Option<Model> {
        self.models.iter().find(|m| m.id == id).cloned()
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