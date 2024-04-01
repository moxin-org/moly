use std::sync::mpsc::{Sender, Receiver, channel};
use moxin_backend::Backend;
use makepad_widgets::SignalToUI;
use std::thread;
use moxin_protocol::data::*;
use moxin_protocol::protocol::Command;
use anyhow::{Result, anyhow};

pub enum SearchAction {
    Results(Vec<Model>),
}

pub struct Search {
    pub keyword: Option<String>,
    pub next_keyword: Option<String>,
    pub sender: Sender<SearchAction>,
    pub receiver: Receiver<SearchAction>,
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
            next_keyword: None,
            sender: tx,
            receiver: rx,
        };
        search
    }

    pub fn run_or_enqueue(&mut self, keyword: String, backend: &Backend) {
        if self.keyword.is_some() {
            self.next_keyword = Some(keyword);
            return;
        } else {
            self.keyword = Some(keyword.clone());
            self.next_keyword = None;
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
                        dbg!("got models", &models);
                        store_search_tx.send(SearchAction::Results(models)).unwrap();
                    },
                    Err(err) => eprintln!("Error fetching models: {:?}", err),
                }
                SignalToUI::set_ui_signal();
            }
        });
    }

    pub fn process_results(&mut self, backend: &Backend) -> Result<Vec<Model>>{
        for msg in self.receiver.try_iter() {
            match msg {
                SearchAction::Results(models) => {
                    dbg!("models procesados", &models);
                    self.keyword = None;
                    if let Some(next_keyword) = self.next_keyword.take() {
                        self.run_or_enqueue(next_keyword, backend);
                    }
                    return Ok(models)
                }
            }
        }
        Err(anyhow!("No results found"))
    }
}