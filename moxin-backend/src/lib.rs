mod fake_data;

use std::sync::mpsc;
use moxin_protocol::protocol::{Command, Response};

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
    pub response_receiver: mpsc::Receiver<Response>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Backend {
        let (command_sender, command_receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();

        // The backend thread
        std::thread::spawn(move || {
            loop {
                if let Ok(command) = command_receiver.recv() {
                    match command {
                        Command::GetFeaturedModels => {
                            let models = fake_data::get_models();
                            response_sender.send(Response::FeaturedModels(models)).unwrap();
                        }
                        Command::SearchModels(query) => {
                            println!("Searching for models with query: {}", query);
                        }
                    }
                }
            }
        });

        Backend { command_sender, response_receiver }
    }
}