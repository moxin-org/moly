use crate::protocol::*;
pub use crate::utils::asynchronous::{moly_future, moly_stream, MolyFuture, MolyStream};
use std::sync::{Arc, Mutex};

/// A client that can be composed from multiple subclients to interact with all of them as one.
#[derive(Clone)]
pub struct MultiClient {
    clients_with_bots: Arc<Mutex<Vec<(Box<dyn BotClient>, Vec<Bot>)>>>,
}

impl MultiClient {
    pub fn new() -> Self {
        MultiClient {
            clients_with_bots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_client(&mut self, client: Box<dyn BotClient>) {
        self.clients_with_bots
            .lock()
            .unwrap()
            .push((client, Vec::new()));
    }
}

impl BotClient for MultiClient {
    fn send_stream(
        &mut self,
        bot: &BotId,
        messages: &[Message],
    ) -> MolyStream<'static, Result<String, ()>> {
        let mut client = self
            .clients_with_bots
            .lock()
            .unwrap()
            .iter()
            .find_map(|(client, bots)| {
                if bots.iter().any(|b| b.id == *bot) {
                    Some(client.clone())
                } else {
                    None
                }
            })
            .expect("no client for bot");

        client.send_stream(&bot, messages)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    fn bots(&self) -> MolyFuture<'static, Result<Vec<Bot>, ()>> {
        let clients_with_bots = self.clients_with_bots.clone();

        let future = async move {
            let clients = clients_with_bots
                .lock()
                .unwrap()
                .iter()
                .map(|(client, _)| client.clone())
                .collect::<Vec<_>>();

            let bot_futures = clients.iter().map(|client| client.bots());
            let results = futures::future::join_all(bot_futures).await;

            let mut zipped_bots = Vec::new();
            let mut flat_bots = Vec::new();

            for result in results {
                // TODO: Let's ignore any errored sub-client for now.
                let client_bots = result.unwrap_or_default();
                zipped_bots.push(client_bots.clone());
                flat_bots.extend(client_bots);
            }

            *clients_with_bots.lock().unwrap() = clients
                .into_iter()
                .zip(zipped_bots.iter().cloned())
                .collect();

            Ok(flat_bots)
        };

        moly_future(future)
    }
}
