use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use makepad_widgets::Cx;
use moly_kit::{utils::asynchronous::spawn, BotId};

use super::providers::*;

#[derive(Clone, Debug)]
pub struct DeepInquireClient {
    command_sender: UnboundedSender<ProviderCommand>,
}

impl ProviderClient for DeepInquireClient {
    fn fetch_models(&self) {
        self.command_sender
            .unbounded_send(ProviderCommand::FetchModels())
            .unwrap();
    }
}

impl DeepInquireClient {
    pub fn new(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = unbounded();
        let address_clone = address.clone();
        spawn(async move {
            Self::process_agent_commands(command_receiver, address_clone, api_key).await;
        });

        Self { command_sender }
    }

    /// Handles the communication between the DeepInquireClient and the MoFa DeepInquire server.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    async fn process_agent_commands(
        mut command_receiver: UnboundedReceiver<ProviderCommand>,
        address: String,
        _api_key: Option<String>,
    ) {
        while let Some(command) = command_receiver.next().await {
            match command {
                ProviderCommand::FetchModels() => {
                    let id = BotId::new("DeepInquire", &address);
                    let provider_bots = vec![ProviderBot {
                        id,
                        name: "DeepInquire".to_string(),
                        description: "A search assistant".to_string(),
                        enabled: true,
                        provider_url: address.clone(),
                    }];
                    Cx::post_action(ProviderFetchModelsResult::Success(
                        address.clone(),
                        provider_bots,
                    ));
                }
            }
        }
    }
}
