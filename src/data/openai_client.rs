use moly_kit::BotId;
use serde::Deserialize;
use std::sync::mpsc::{self, channel, Sender};
use log::error;

use makepad_widgets::Cx;
use crate::data::providers::ProviderClientError;

use super::providers::*;

fn should_include_model(model_id: &str) -> bool {
    // First, filter out non-chat models
    if model_id.contains("dall-e") || 
       model_id.contains("whisper") || 
       model_id.contains("tts") ||
       model_id.contains("davinci") ||
       model_id.contains("audio") ||
       model_id.contains("babbage") ||
       model_id.contains("moderation") ||
       model_id.contains("embedding") {
        return false;
    }

    true
}

#[derive(Clone, Debug)]
pub struct OpenAIClient {
    command_sender: Sender<ProviderCommand>,
}

impl ProviderClient for OpenAIClient {
    fn fetch_models(&self) {
        self.command_sender
            .send(ProviderCommand::FetchModels())
            .unwrap();
    }
}

impl OpenAIClient {
    pub fn new(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        let api_key_clone = api_key.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone, api_key_clone);
        });

        Self {
            command_sender,
        }
    }

    /// Handles the communication between the OpenAIClient and the remote server
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String, api_key: Option<String>) {
        while let Ok(command) = command_receiver.recv() {
            match command {
                ProviderCommand::FetchModels() => {
                    let url = address.clone();
                    let client = reqwest::blocking::ClientBuilder::new()
                        .timeout(std::time::Duration::from_secs(5))
                        .no_proxy()
                        .build()
                        .unwrap();
                    
                    let mut req = client.get(format!("{}/models", url));

                    // Add Authorization header if API key is available
                    if let Some(key) = &api_key {
                        req = req.header("Authorization", format!("Bearer {}", key));
                    }

                    let resp = req.send();

                    #[allow(dead_code)]
                    #[derive(Deserialize, Debug)]
                    struct ModelInfo {
                        id: String,
                        // may not be present
                        object: Option<String>,
                        created: Option<i64>,
                        owned_by: Option<String>,
                    }

                    #[allow(dead_code)]
                    #[derive(Deserialize, Debug)]
                    struct ModelsResponse {
                        object: Option<String>,
                        data: Vec<ModelInfo>,
                    }

                    match resp {
                        Ok(r) => {
                            match r.status() {
                                reqwest::StatusCode::OK => {
                                    match r.json::<ModelsResponse>() {
                                        Ok(models) => {
                                            let models: Vec<ProviderBot> = models.data.into_iter()
                                                .filter(|model| should_include_model(&model.id))
                                                .map(|model| ProviderBot {
                                                    id: BotId::new(&model.id, &url),
                                                    name: model.id.clone(),
                                                    description: format!("OpenAI {} model", model.object.unwrap_or(model.id)),
                                                    provider_url: url.clone(),
                                                    enabled: true,
                                                })
                                                .collect();
                                            Cx::post_action(ProviderFetchModelsResult::Success(url, models));
                                        }
                                        Err(e) => {
                                            error!("Failed to parse models response from {}: {:?}", url, e);
                                            Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
                                        }
                                    }
                                }
                                reqwest::StatusCode::UNAUTHORIZED => {
                                    error!("Unauthorized (401) fetching models from {}: API key missing/invalid?", url);
                                    Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::Unauthorized));
                                }
                                status => {
                                    error!("Failed to fetch models from {} - Status: {:?}", url, status);
                                    Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
                                }
                            }
                        },
                        Err(e) => {
                            error!("Network/Request error fetching models from {}: {}", url, e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
                        }
                    }
                }
            }
        }
    }
}
