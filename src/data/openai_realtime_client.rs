use makepad_widgets::{Cx, error};
use moly_kit::{protocol::*, utils::asynchronous::spawn};

use super::providers::*;
use crate::data::providers::ProviderClientError;

#[derive(Clone, Debug)]
pub struct OpenAIRealtimeClient {
    url: String,
    client: moly_kit::clients::OpenAIRealtimeClient,
}

impl OpenAIRealtimeClient {
    pub fn new(url: String) -> Self {
        let client_url = url.trim_start_matches('#').to_string();
        let client = moly_kit::clients::OpenAIRealtimeClient::new(client_url);

        OpenAIRealtimeClient { url, client }
    }

    pub fn set_key(&mut self, api_key: &str) -> Result<(), String> {
        self.client.set_key(api_key)
    }
}

impl ProviderClient for OpenAIRealtimeClient {
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
                            description: format!("OpenAI Realtime Model {}", bot.id.id()),
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
                            error!("Network error while fetching realtime models: {}", e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                        ClientErrorKind::Format => {
                            error!("Failed to parse realtime models response: {}", e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                        ClientErrorKind::Response => {
                            error!("Received a bad response from the realtime server: {}", e);
                            Cx::post_action(ProviderFetchModelsResult::Failure(
                                url,
                                ProviderClientError::UnexpectedResponse,
                            ));
                        }
                        ClientErrorKind::Unknown => {
                            let message =
                                format!("Unknown error while fetching realtime models: {}", e);
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
