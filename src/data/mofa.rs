use moly_kit::BotId;
use std::sync::mpsc::{self, channel, Sender};
use makepad_widgets::Cx;
use super::providers::*;

/// This is a OpenAI-compatible client for MoFa.
/// The only reason we have this is to return fake responses upon model fetching.
#[derive(Clone, Debug)]
pub struct MofaClient {
    command_sender: Sender<ProviderCommand>,
}

impl ProviderClient for MofaClient {
    fn fetch_models(&self) {
        self.command_sender
            .send(ProviderCommand::FetchModels())
            .unwrap();
    }
}

impl MofaClient {
    pub fn new(address: String) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone);
        });

        Self {
            command_sender,
        }
    }

    /// Handles the communication between the MofaClient and the MoFa server.
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String) {
        while let Ok(command) = command_receiver.recv() {
            match command {
                ProviderCommand::FetchModels() => {
                    let url = address.clone();
                    let resp = reqwest::blocking::ClientBuilder::new()
                        .timeout(std::time::Duration::from_secs(5))
                        .no_proxy()
                        .build()
                        .unwrap()
                        .get(format!("{}/v1/models", url))
                        .send();


                    match resp {
                        Ok(r) => {
                            match r.status() {
                                reqwest::StatusCode::OK => {
                                    let agents = vec![
                                        ProviderBot {
                                            id: BotId::new("Reasoner", &url),
                                            name: "Reasoner".to_string(),
                                            description: "An agent that will help you find good questions about any topic".to_string(),
                                            enabled: true,
                                            provider_url: url.clone(),
                                        },
                                    ];
                                    Cx::post_action(ProviderFetchModelsResult::Success(url, agents));
                                }
                                reqwest::StatusCode::UNAUTHORIZED => {
                                    eprintln!("Unauthorized to fetch models from: {}, your API key might be missing or invalid", url);
                                    Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::Unauthorized));
                                }
                                status => {
                                    eprintln!("Failed to fetch models from: {}, with status: {:?}", url, status);
                                    Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to fetch models from server: {e}");
                            Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
                        }
                    }
                }
            }
        }
    }
}
