use makepad_widgets::{Cx, LiveId, LivePtr, WidgetRef};
use crate::protocol::Tool;

use crate::protocol::*;
use crate::utils::asynchronous::{BoxPlatformSendFuture, BoxPlatformSendStream};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
        tools: &[Tool],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let client = self
            .clients_with_bots
            .lock()
            .unwrap()
            .iter()
            .find_map(|(client, bots)| {
                if bots.iter().any(|b| b.id == *bot_id) {
                    Some(client.clone())
                } else {
                    None
                }
            });

        match client {
            Some(mut client) => client.send(bot_id, messages, tools),
            None => {
                let bot_id_clone = bot_id.clone();
                Box::pin(futures::stream::once(async move {
                    ClientError::new(
                        ClientErrorKind::Unknown,
                        format!(
                            "Can't find a client to communicate with the bot {:?}",
                            bot_id_clone
                        ),
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

    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
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

        Box::pin(future)
    }

    fn content_widget(
        &mut self,
        cx: &mut Cx,
        previous_widget: WidgetRef,
        templates: &HashMap<LiveId, LivePtr>,
        content: &MessageContent,
    ) -> Option<WidgetRef> {
        self.clients_with_bots
            .lock()
            .unwrap()
            .iter_mut()
            .find_map(|(client, _)| {
                client.content_widget(cx, previous_widget.clone(), templates, content)
            })
    }
}
