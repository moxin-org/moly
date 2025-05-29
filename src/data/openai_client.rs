use log::error;
use makepad_widgets::Cx;
use moly_kit::{BotId, utils::asynchronous::spawn};
use serde::Deserialize;

use super::providers::*;
use crate::data::providers::ProviderClientError;

fn should_include_model(model_id: &str) -> bool {
    // First, filter out non-chat models
    if model_id.contains("dall-e")
        || model_id.contains("whisper")
        || model_id.contains("tts")
        || model_id.contains("davinci")
        || model_id.contains("audio")
        || model_id.contains("babbage")
        || model_id.contains("moderation")
        || model_id.contains("embedding")
    {
        return false;
    }

    true
}

#[derive(Clone, Debug)]
pub struct OpenAIClient {
    address: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl ProviderClient for OpenAIClient {
    fn fetch_models(&self) {
        let url = self.address.clone();
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        spawn(async move {
            let mut req = client.get(format!("{}/models", url));

            // Add Authorization header if API key is available
            if let Some(key) = &api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req.send().await;

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
                Ok(r) => match r.status() {
                    reqwest::StatusCode::OK => match r.json::<ModelsResponse>().await {
                        Ok(models) => {
                            let models: Vec<ProviderBot> = models
                                .data
                                .into_iter()
                                .filter(|model| should_include_model(&model.id))
                                .map(|model| ProviderBot {
                                    id: BotId::new(&model.id, &url),
                                    name: model.id.clone(),
                                    description: format!(
                                        "OpenAI {} model",
                                        model.object.unwrap_or(model.id)
                                    ),
                                    provider_url: url.clone(),
                                    enabled: true,
                                })
                                .collect();
                            Cx::post_action(ProviderFetchModelsResult::Success(url, models));
                        }
                        Err(e) => {
                            error!("Failed to parse models response from {}: {:?}", url, e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                    },
                    reqwest::StatusCode::UNAUTHORIZED => {
                        error!(
                            "Unauthorized (401) fetching models from {}: API key missing/invalid?",
                            url
                        );
                        Cx::post_action(ProviderFetchModelsResult::Failure(
                            url,
                            ProviderClientError::Unauthorized,
                        ));
                    }
                    status => {
                        error!("Failed to fetch models from {} - Status: {:?}", url, status);
                        Cx::post_action(ProviderFetchModelsResult::Failure(
                            url,
                            ProviderClientError::UnexpectedResponse,
                        ));
                    }
                },
                Err(e) => {
                    error!("Network/Request error fetching models from {}: {}", url, e);
                    Cx::post_action(ProviderFetchModelsResult::Failure(
                        url,
                        ProviderClientError::UnexpectedResponse,
                    ));
                }
            }
        })
    }
}

impl OpenAIClient {
    pub fn new(address: String, api_key: Option<String>) -> Self {
        let client = reqwest::ClientBuilder::new();

        // web doesn't support these
        #[cfg(not(target_arch = "wasm32"))]
        let client = client
            .timeout(std::time::Duration::from_secs(60))
            .no_proxy();

        let client = client.build().unwrap();

        Self {
            address,
            api_key,
            client,
        }
    }
}
