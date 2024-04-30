use anyhow::{anyhow, Result};
use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::data::*;
use moxin_protocol::protocol::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub enum SearchAction {
    Results(Vec<Model>),
}

pub enum SearchCommand {
    Search(String),
    LoadFeaturedModels,
}

pub struct Search {
    pub keyword: Option<String>,
    pub current_command: Option<SearchCommand>,
    pub next_command: Option<SearchCommand>,
    pub sender: Sender<SearchAction>,
    pub receiver: Receiver<SearchAction>,
    pub pending: bool,
}

impl Default for Search {
    fn default() -> Self {
        Search::new()
    }
}

impl Search {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let search = Self {
            keyword: None,
            current_command: None,
            next_command: None,
            sender: tx,
            receiver: rx,
            pending: false,
        };
        search
    }

    pub fn load_featured_models(&mut self, backend: &Backend) {
        if self.pending {
            self.next_command = Some(SearchCommand::LoadFeaturedModels);
            return;
        } else {
            self.pending = true;
            self.keyword = None;
            self.next_command = None;
        }

        let (tx, rx) = channel();

        let store_search_tx = self.sender.clone();
        backend
            .command_sender
            .send(Command::GetFeaturedModels(tx))
            .unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(models) => {
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    }
                    Err(err) => eprintln!("Error fetching models: {:?}", err),
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

    pub fn run_or_enqueue(&mut self, keyword: String, backend: &Backend) {
        if self.pending {
            self.next_command = Some(SearchCommand::Search(keyword));
            return;
        } else {
            self.pending = true;
            self.current_command = Some(SearchCommand::Search(keyword.clone()));
            self.next_command = None;
        }

        let (tx, rx) = channel();

        let store_search_tx = self.sender.clone();
        backend
            .command_sender
            .send(Command::SearchModels(keyword.clone(), tx))
            .unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(models) => {
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    }
                    Err(err) => eprintln!("Error fetching models: {:?}", err),
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

    pub fn process_results(&mut self, backend: &Backend) -> Result<Vec<Model>> {
        for msg in self.receiver.try_iter() {
            match msg {
                SearchAction::Results(models) => {
                    self.pending = false;
                    if let Some(SearchCommand::Search(keyword)) = self.current_command.take() {
                        self.keyword = Some(keyword);
                    }
                    match self.next_command.take() {
                        Some(SearchCommand::Search(next_keyword)) => {
                            self.run_or_enqueue(next_keyword, backend);
                        }
                        Some(SearchCommand::LoadFeaturedModels) => {
                            self.load_featured_models(backend);
                        }
                        None => {}
                    }
                    return Ok(models);
                }
            }
        }
        Err(anyhow!("No results found"))
    }
}
