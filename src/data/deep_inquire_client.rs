use makepad_widgets::Cx;
use moly_kit::BotId;

use super::providers::*;

#[derive(Clone, Debug)]
pub struct DeepInquireClient {
    address: String,
}

impl ProviderClient for DeepInquireClient {
    fn fetch_models(&self) {
        let id = BotId::new("DeepInquire", &self.address);
        let provider_bots = vec![ProviderBot {
            id,
            name: "DeepInquire".to_string(),
            description: "A search assistant".to_string(),
            enabled: true,
            provider_url: self.address.clone(),
        }];
        Cx::post_action(ProviderFetchModelsResult::Success(
            self.address.clone(),
            provider_bots,
        ));
    }
}

impl DeepInquireClient {
    pub fn new(address: String) -> Self {
        Self { address }
    }
}
