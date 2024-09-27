mod faked_models;

use anyhow::{anyhow, Result};
use makepad_widgets::SignalToUI;
use moly_backend::Backend;
use moly_protocol::data::*;
use moly_protocol::protocol::Command;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Clone, Copy, Debug, Default)]
pub enum SortCriteria {
    #[default]
    MostDownloads,
    LeastDownloads,
    MostLikes,
    LeastLikes,
}
pub enum SearchAction {
    Results(Vec<Model>),
    Error,
}

#[derive(Clone)]
pub enum SearchCommand {
    Search(String),
    LoadFeaturedModels,
}

#[derive(Default, Clone)]
pub enum SearchState {
    #[default]
    Idle,
    Pending(SearchCommand, Option<SearchCommand>),
    Errored,
}
pub struct Search {
    pub backend: Rc<Backend>,
    pub models: Vec<Model>,
    pub sorted_by: SortCriteria,
    pub keyword: Option<String>,
    pub sender: Sender<SearchAction>,
    pub receiver: Receiver<SearchAction>,
    pub state: SearchState,
}

impl Search {
    pub fn new(backend: Rc<Backend>) -> Self {
        let (tx, rx) = channel();
        let search = Self {
            backend,
            models: Vec::new(),
            sorted_by: SortCriteria::MostDownloads,
            keyword: None,
            sender: tx,
            receiver: rx,
            state: SearchState::Idle,
        };
        search
    }

    pub fn load_featured_models(&mut self) {
        match self.state {
            SearchState::Pending(_, ref mut next_command) => {
                *next_command = Some(SearchCommand::LoadFeaturedModels);
                return;
            }
            SearchState::Idle | SearchState::Errored => {
                self.state = SearchState::Pending(SearchCommand::LoadFeaturedModels, None);
                self.keyword = None;
            }
        }

        let (tx, rx) = channel();

        let store_search_tx = self.sender.clone();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::GetFeaturedModels(tx))
            .unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(models) => {
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error fetching models: {:?}", err);
                        store_search_tx.send(SearchAction::Error).unwrap();
                    }
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

    pub fn load_search_results(&mut self, query: String) {
        self.run_or_enqueue(query);
    }

    fn run_or_enqueue(&mut self, keyword: String) {
        match self.state {
            SearchState::Pending(_, ref mut next_command) => {
                *next_command = Some(SearchCommand::Search(keyword));
                return;
            }
            SearchState::Idle | SearchState::Errored => {
                self.state = SearchState::Pending(SearchCommand::Search(keyword.clone()), None);
            }
        }

        let (tx, rx) = channel();

        let store_search_tx = self.sender.clone();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::SearchModels(keyword.clone(), tx))
            .unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(models) => {
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error fetching models: {:?}", err);
                        store_search_tx.send(SearchAction::Error).unwrap();
                    }
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

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

    pub fn set_models(&mut self, models: Vec<Model>) {
        #[cfg(not(debug_assertions))]
        {
            self.models = models;
        }
        #[cfg(debug_assertions)]
        'debug_block: {
            use faked_models::get_faked_models;

            let fill_fake_data = std::env::var("FILL_FAKE_DATA").is_ok_and(|fill_fake_data| {
                ["true", "t", "1"].iter().any(|&s| s == fill_fake_data)
            });

            if !fill_fake_data {
                self.models = models;
                break 'debug_block;
            }

            let faked_models: Vec<Model> = get_faked_models(&models);
            self.models = faked_models;
        }

        self.sort_models(self.sorted_by);
    }

    pub fn process_results(&mut self) -> Result<Option<Vec<Model>>> {
        for msg in self.receiver.try_iter() {
            match msg {
                SearchAction::Results(models) => {
                    let previous_state = self.state.to_owned();
                    self.state = SearchState::Idle;

                    if let SearchState::Pending(current_command, next_command) = previous_state {
                        if let SearchCommand::Search(keyword) = current_command {
                            self.keyword = Some(keyword.clone());
                        }

                        match next_command {
                            Some(SearchCommand::Search(next_keyword)) => {
                                self.run_or_enqueue(next_keyword.clone());
                            }
                            Some(SearchCommand::LoadFeaturedModels) => {
                                self.load_featured_models();
                            }
                            None => {}
                        }
                        return Ok(Some(models));
                    } else {
                        return Err(anyhow!("Client was not expecting to receive results"));
                    }
                }
                SearchAction::Error => {
                    self.state = SearchState::Errored;
                    return Err(anyhow!("Error fetching models from the server"));
                }
            }
        }
        Ok(None)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.state, SearchState::Pending(_, _))
    }

    pub fn was_error(&self) -> bool {
        matches!(self.state, SearchState::Errored)
    }

    pub fn update_downloaded_file_in_search_results(&mut self, file_id: &FileID, downloaded: bool) {
        let model = self
            .models
            .iter_mut()
            .find(|m| m.files.iter().any(|f| f.id == *file_id));
        if let Some(model) = model {
            let file = model.files.iter_mut().find(|f| f.id == *file_id).unwrap();
            file.downloaded = downloaded;
        }
    }

    pub fn get_model_and_file_from_search_results(&self, file_id: &str) -> Option<(Model, File)> {
        self.models.iter().find_map(|m| {
            m.files
                .iter()
                .find(|f| f.id == file_id)
                .map(|f| (m.clone(), f.clone()))
        })
    }
}
