use std::sync::mpsc::channel;
use crate::data::store::*;

#[derive(Clone, Debug)]
pub enum Command {
    GetFeaturedModels,
    SearchModels(String),
}

#[derive(Clone, Debug)]
pub enum Response {
    FeaturedModels(Vec<Model>)
}

pub struct Backend {
    pub command_sender: std::sync::mpsc::Sender<Command>,
    pub response_receiver: std::sync::mpsc::Receiver<Response>,
}


impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Backend {
        let (command_sender, command_receiver) = channel();
        let (response_sender, response_receiver) = channel();

        // The backend thread... will be provided by the backend crate
        std::thread::spawn(move || {
            loop {
                let command = command_receiver.recv().unwrap();

                match command {
                    Command::GetFeaturedModels => {
                        let models = Store::new().models;
                        response_sender.send(Response::FeaturedModels(models)).unwrap();
                    }
                    Command::SearchModels(query) => {
                        println!("Searching for models with query: {}", query);
                    }
                }
            }
        });

        Backend { command_sender, response_receiver }
    }
}