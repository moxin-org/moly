use makepad_widgets::{Cx, error};
use moly_kit::{protocol::*, utils::asynchronous::spawn};

use super::providers::*;
use crate::data::providers::ProviderClientError;

#[derive(Clone, Debug)]
pub struct OpenAIImageClient {
    url: String,
    client: moly_kit::clients::OpenAIImageClient,
}

impl OpenAIImageClient {
    pub fn new(url: String, api_key: Option<String>) -> Self {
        let client_url = url.trim_start_matches('#').to_string();
        let mut client = moly_kit::clients::OpenAIImageClient::new(client_url);
        if let Some(key) = api_key {
            let _ = client.set_key(&key);
        }

        OpenAIImageClient { url, client }
    }
}

impl ProviderClient for OpenAIImageClient {
    fn fetch_models(&self) {
        let url = self.url.clone();
        let future = self.client.bots();
        spawn(async move {
            match future.await.into_result() {
                Ok(bots) => {
                    let bots = bots
                        .into_iter()
                        .map(|bot| ProviderBot {
                            id: bot.id.clone(),
                            name: bot.name,
                            description: format!("OpenAI Image Generation Model {}", bot.id.id()),
                            provider_url: url.clone(),
                            enabled: true,
                        })
                        .collect::<Vec<_>>();
                    Cx::post_action(ProviderFetchModelsResult::Success(url, bots));
                }
                Err(e) => {
                    let e = e
                        .first()
                        .expect("error variant without error should not be possible");
                    match e.kind() {
                        ClientErrorKind::Network => {
                            error!("Network error while fetching models: {}", e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                        ClientErrorKind::Format => {
                            error!("Failed to parse models response: {}", e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                        ClientErrorKind::Response => {
                            error!("Received a bad response from the server: {}", e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                        ClientErrorKind::Unknown => {
                            let message = format!("Unknown error while fetching models: {}", e);
                            error!("{}", message);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::Other(message),
                            ));
                        }
                    }
                }
            }
        });
    }
}
