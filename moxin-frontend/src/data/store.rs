use chrono::Utc;
use moxin_protocol::data::{Model, File};
use moxin_protocol::protocol::{Command, Response};
use moxin_backend::Backend;

#[derive(Default)]
pub struct Store {
    pub backend: Backend,
    pub models: Vec<Model>,
}

impl Store {
    pub fn new() -> Self {
        let mut store = Self {
            models: vec![],
            backend: Backend::default(),
        };

        store.backend.command_sender.send(Command::GetFeaturedModels).unwrap();
        if let Ok(response) = store.backend.response_receiver.recv() {
            let Response::FeaturedModels(models) = response;
            store.models = models;
        };

        store
    }

    pub fn formatted_model_release_date(model: &Model) -> String {
        let released_at = model.released_at.format("%b %-d, %C%y");
        let days_ago = (Utc::now().date_naive() - model.released_at).num_days();
        format!("{} ({} days ago)", released_at, days_ago)
    }

    pub fn model_featured_files(model: &Model) -> Vec<File> {
        model.files.iter().filter(|f| f.featured).cloned().collect()
    }
}