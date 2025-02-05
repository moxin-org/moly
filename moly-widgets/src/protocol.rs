use futures::StreamExt;
use makepad_widgets::LiveValue;
use std::sync::{Arc, Mutex};

pub use crate::utils::asynchronous::{moly_future, moly_stream, MolyFuture, MolyStream};

/// The picture/avatar of an entity that may be represented/encoded in different ways.
#[derive(Clone, PartialEq, Debug)]
pub enum Picture {
    // TODO: could be reduced to avoid allocation
    Grapheme(String),
    Image(String),
    // TODO: could be downed to a more concrete type
    Dependency(LiveValue),
}

/// Indentify the entities that are recognized by this crate.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EntityId {
    User,
    System,
    Bot(BotId),
    App,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Bot {
    pub id: BotId,
    pub name: String,
    pub avatar: Picture,
}

/// Indentifies any kind of bot, local or remote, model or agent, whatever.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BotId(Arc<str>);

impl From<&str> for BotId {
    fn from(id: &str) -> Self {
        BotId(id.into())
    }
}

impl ToString for BotId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl BotId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A message that is part of a conversation.
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    /// The id of who sent this message.
    pub from: EntityId,
    /// Content of the message.
    pub body: String,
    /// If this message is still being written.
    ///
    /// This means the message is still going to be modified.
    ///
    /// If `false`, it means the message will not change anymore.
    pub is_writing: bool,
}

/// An interface to talk to bots.
///
/// Warning: Expect this to be cloned to avoid borrow checking issues with
/// makepad's widgets. Also, it may be cloned inside async contexts. So keep this
/// cheap to clone and synced.
///
/// Note: Generics do not play well with makepad's widgets, so this trait relies
/// on dynamic dispatch (with its limitations).
pub trait BotClient: Send {
    /// Send a message to a bot expecting a streamed response.
    fn send_stream(
        &mut self,
        bot: &BotId,
        messages: &[Message],
    ) -> MolyStream<'static, Result<String, ()>>;

    /// Interrupt the bot's current operation.
    // TODO: There may be many chats with the same bot/model/agent so maybe this
    // should be implemented by using cancellation tokens.
    // fn stop(&mut self, bot: BotId);

    /// Bots available under this client.
    // NOTE: Could be a stream, but may add complexity rarely needed.
    // TODO: Support partial results with errors for an union multi client/service
    // later.
    fn bots(&self) -> MolyFuture<'static, Result<Vec<Bot>, ()>>;

    /// Make a boxed dynamic clone of this client to pass around.
    fn clone_box(&self) -> Box<dyn BotClient>;

    /// Send a message to a bot expecting a full response at once.
    // TODO: messages may end up being a little bit more complex, using string while thinking.
    // TODO: Should support a way of passing, unknown, backend-specific, inference parameters.
    fn send(
        &mut self,
        bot: &BotId,
        messages: &[Message],
    ) -> MolyFuture<'static, Result<String, ()>> {
        let stream = self.send_stream(bot, messages);

        let future = async move {
            let parts = stream.collect::<Vec<_>>().await;

            if parts.contains(&Err(())) {
                return Err(());
            }

            let message = parts.into_iter().filter_map(Result::ok).collect::<String>();
            Ok(message)
        };

        moly_future(future)
    }
}

impl Clone for Box<dyn BotClient> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

struct InnerBotRepo {
    client: Box<dyn BotClient>,
    bots: Vec<Bot>,
}

pub struct BotRepo(Arc<Mutex<InnerBotRepo>>);

impl Clone for BotRepo {
    fn clone(&self) -> Self {
        BotRepo(self.0.clone())
    }
}

impl BotRepo {
    pub fn load(&mut self) -> MolyFuture<Result<(), ()>> {
        let future = async move {
            let new_bots = self.client().bots().await?;
            self.0.lock().unwrap().bots = new_bots;
            Ok(())
        };

        moly_future(future)
    }
    pub fn client(&self) -> Box<dyn BotClient> {
        self.0.lock().unwrap().client.clone_box()
    }

    pub fn bots(&self) -> Vec<Bot> {
        self.0.lock().unwrap().bots.clone()
    }

    pub fn get_bot(&self, id: &BotId) -> Option<Bot> {
        self.bots().into_iter().find(|bot| bot.id == *id)
    }
}

impl<T: BotClient + 'static> From<T> for BotRepo {
    fn from(client: T) -> Self {
        BotRepo(Arc::new(Mutex::new(InnerBotRepo {
            client: Box::new(client),
            bots: Vec::new(),
        })))
    }
}

#[derive(Clone)]
pub struct MultiBotClient {
    clients_with_bots: Arc<Mutex<Vec<(Box<dyn BotClient>, Vec<Bot>)>>>,
}

impl MultiBotClient {
    pub fn new() -> Self {
        MultiBotClient {
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

impl BotClient for MultiBotClient {
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
