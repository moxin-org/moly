use makepad_widgets::{Cx, WidgetRef};

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
        bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>> {
        let client = self
            .clients_with_bots
            .lock()
            .unwrap()
            .iter()
            .find_map(|(client, bots)| {
                if bots.iter().any(|b| b.id == bot.id) {
                    Some(client.clone())
                } else {
                    None
                }
            });

        match client {
            Some(mut client) => client.send_stream(bot, messages),
            None => {
                let bot = bot.clone();
                moly_stream(futures::stream::once(async move {
                    ClientError::new(
                        ClientErrorKind::Unknown,
                        format!("Can't find a client to communicate with the bot {:?}", bot),
                    )
                    .into()
                }))
            }
        }
    }

    // TODO: Add `send` implementation to take adventage of `send` implementation in sub-clients.

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
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
            let mut errors = Vec::new();

            for result in results {
                let (v, e) = result.into_value_and_errors();
                let v = v.unwrap_or_default();
                zipped_bots.push(v.clone());
                flat_bots.extend(v);
                errors.extend(e);
            }

            *clients_with_bots.lock().unwrap() = clients
                .into_iter()
                .zip(zipped_bots.iter().cloned())
                .collect();

            if errors.is_empty() {
                ClientResult::new_ok(flat_bots)
            } else {
                if flat_bots.is_empty() {
                    ClientResult::new_err(errors)
                } else {
                    ClientResult::new_ok_and_err(flat_bots, errors)
                }
            }
        };

        moly_future(future)
    }

    fn content_widget(&mut self, cx: &mut Cx, content: &MessageContent) -> Option<WidgetRef> {
        self.clients_with_bots
            .lock()
            .unwrap()
            .iter_mut()
            .find_map(|(client, _)| client.content_widget(cx, content))
    }
}
