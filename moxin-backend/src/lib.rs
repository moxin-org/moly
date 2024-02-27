mod fake_data;

use moxin_protocol::protocol::Command;
use std::sync::mpsc;

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Backend {
        let (command_sender, command_receiver) = mpsc::channel();

        // The backend thread
        std::thread::spawn(move || loop {
            if let Ok(command) = command_receiver.recv() {
                match command {
                    Command::GetFeaturedModels(tx) => {
                        let models = fake_data::get_models();
                        tx.send(models).unwrap();
                    }
                    Command::SearchModels(query, _tx) => {
                        println!("Searching for models with query: {}", query);
                    }
                    _ => {}
                }
            }
        });

        Backend { command_sender }
    }
}
