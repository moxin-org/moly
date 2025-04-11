use moly_kit::BotId;
use std::sync::mpsc::{self, channel, Sender};
use makepad_widgets::Cx;

use super::providers::*;

#[derive(Clone, Debug)]
pub struct DeepInquireClient {
    command_sender: Sender<ProviderCommand>,
}

impl ProviderClient for DeepInquireClient {
    fn fetch_models(&self) {
        self.command_sender
            .send(ProviderCommand::FetchModels())
            .unwrap();
    }
}

impl DeepInquireClient {
    pub fn new(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone, api_key);
        });

        Self {
            command_sender,
        }
    }

    /// Handles the communication between the DeepInquireClient and the MoFa DeepInquire server.
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String, _api_key: Option<String>) {
        while let Ok(command) = command_receiver.recv() {
            match command {
                ProviderCommand::FetchModels() => {
                    let id = BotId::new("DeepInquire", &address);
                    let provider_bots = vec![
                        ProviderBot {
                            id,
                            name: "DeepInquire".to_string(),
                            description: "A search assistant".to_string(),
                            enabled: true,
                            provider_url: address.clone(),
                        },
                    ];
                    Cx::post_action(ProviderFetchModelsResult::Success(address.clone(), provider_bots));
                }
            }
        }
    }
}
