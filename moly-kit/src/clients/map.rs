use crate::protocol::*;
use crate::utils::asynchronous::{BoxPlatformSendFuture, BoxPlatformSendStream};
use std::sync::{Arc, Mutex};

struct Inner<C: BotClient> {
    client: C,
    map_bots: Option<Box<dyn FnMut(Vec<Bot>) -> Vec<Bot> + Send + 'static>>,
    map_send: Option<Box<dyn FnMut(MessageContent) -> MessageContent + Send + 'static>>,
}

/// Utility wrapper client that transforms the output of the underlying client.
///
/// This is just limited to synchronous transformations over successful results.
///
/// In general, it's recommended to implement the [`BotClient`] trait directly for
/// maximum control instead of using this.
pub struct MapClient<C: BotClient> {
    inner: Arc<Mutex<Inner<C>>>,
}

impl<C: BotClient> Clone for MapClient<C> {
    fn clone(&self) -> Self {
        MapClient {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<C: BotClient> MapClient<C> {
    pub fn new(client: C) -> Self {
        MapClient {
            inner: Arc::new(Mutex::new(Inner {
                client,
                map_bots: None,
                map_send: None,
            })),
        }
    }

    /// Sets a transformation function for the list of bots returned by the `bots` method.
    pub fn set_map_bots(&mut self, map: impl FnMut(Vec<Bot>) -> Vec<Bot> + Send + 'static) {
        let mut inner = self.inner.lock().unwrap();
        inner.map_bots = Some(Box::new(map));
    }

    /// Sets a sync transformation function for the successful result of the `send` method.
    pub fn set_map_send(
        &mut self,
        map: impl FnMut(MessageContent) -> MessageContent + Send + 'static,
    ) {
        let mut inner = self.inner.lock().unwrap();
        inner.map_send = Some(Box::new(map));
    }
}

impl<C: BotClient> From<C> for MapClient<C> {
    fn from(client: C) -> Self {
        MapClient::new(client)
    }
}

impl<C: BotClient + 'static> BotClient for MapClient<C> {
    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        let inner = self.inner.clone();
        let future = self.inner.lock().unwrap().client.bots();

        Box::pin(async move {
            let result = future.await;

            if result.has_errors() {
                return result;
            }

            let mut bots = result.into_value().unwrap();

            if let Some(map_bots) = &mut inner.lock().unwrap().map_bots {
                bots = map_bots(bots);
            }

            ClientResult::new_ok(bots)
        })
    }

    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let inner = self.inner.clone();
        let stream = self.inner.lock().unwrap().client.send(bot_id, messages);

        let stream = async_stream::stream! {
            for await result in stream {
                if result.has_errors() {
                    yield result;
                    continue;
                }

                let mut content = result.into_value().unwrap();

                if let Some(map_send) = &mut inner.lock().unwrap().map_send {
                    content = map_send(content);
                }

                yield ClientResult::new_ok(content);
            }
        };

        Box::pin(stream)
    }
}
