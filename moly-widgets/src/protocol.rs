use std::sync::{Arc, Mutex};

// This is the stream type re-exported by tokio, reqwest and futures.
use futures::{future, stream, FutureExt, StreamExt};
use makepad_widgets::{log, LiveValue};

#[cfg(not(target_arch = "wasm32"))]
pub type BoxFuture<'a, T> = future::BoxFuture<'a, T>;

#[cfg(not(target_arch = "wasm32"))]
pub type BoxStream<'a, T> = stream::BoxStream<'a, T>;

#[cfg(target_arch = "wasm32")]
pub type BoxFuture<'a, T> = future::LocalBoxFuture<'a, T>;

#[cfg(target_arch = "wasm32")]
pub type BoxStream<'a, T> = stream::LocalBoxStream<'a, T>;

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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
///
/// String ids are hashed so they have a very low but still possible chance of collision.
// TODO: Rethink if necessary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BotId(u64);

impl From<u64> for BotId {
    fn from(id: u64) -> Self {
        BotId(id)
    }
}

impl From<&str> for BotId {
    fn from(id: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        BotId(hasher.finish())
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
pub trait BotService: Send {
    /// Send a message to a bot expecting a streamed response.
    fn send_stream(&mut self, bot: BotId, messages: &[Message]) -> BoxStream<Result<String, ()>>;

    /// Interrupt the bot's current operation.
    // TODO: There may be many chats with the same bot/model/agent so maybe this
    // should be implemented by using cancellation tokens.
    // fn stop(&mut self, bot: BotId);

    /// Bots available under this client.
    // TODO: Should be a stream actually?
    fn bots(&self) -> BoxStream<Result<Bot, ()>>;

    /// Make a boxed dynamic clone of this client to pass around.
    fn clone_box(&self) -> Box<dyn BotService>;

    /// Send a message to a bot expecting a full response at once.
    // TODO: messages may end up being a little bit more complex, using string while thinking.
    // TOOD: Should support a way of passing, unknown, backend-specific, inference parameters.
    fn send(&mut self, bot: BotId, messages: &[Message]) -> BoxFuture<Result<String, ()>> {
        let stream = self.send_stream(bot, messages);

        let future = async move {
            let parts = stream.collect::<Vec<_>>().await;

            if parts.contains(&Err(())) {
                return Err(());
            }

            let message = parts.into_iter().filter_map(Result::ok).collect::<String>();
            Ok(message)
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            future.boxed()
        }

        #[cfg(target_arch = "wasm32")]
        future.boxed_local()
    }
}

struct InnerBotRepo {
    service: Box<dyn BotService>,
    bots: Vec<Bot>,
}

pub struct BotRepo(Arc<Mutex<InnerBotRepo>>);

impl Clone for BotRepo {
    fn clone(&self) -> Self {
        BotRepo(self.0.clone())
    }
}

impl BotRepo {
    pub fn load(&mut self) -> BoxFuture<Result<(), ()>> {
        let future = async move {
            let mut new_bots = Vec::new();
            let service = self.service();
            let mut bots_stream = service.bots();

            while let Some(bot) = bots_stream.next().await {
                match bot {
                    Ok(bot) => new_bots.push(bot),
                    Err(_) => {
                        log!("Error loading bots");
                        return Err(());
                    }
                }
            }

            self.0.lock().unwrap().bots = new_bots;
            Ok(())
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            future.boxed()
        }

        #[cfg(target_arch = "wasm32")]
        future.boxed_local()
    }
    pub fn service(&self) -> Box<dyn BotService> {
        self.0.lock().unwrap().service.clone_box()
    }

    pub fn bots(&self) -> Vec<Bot> {
        self.0.lock().unwrap().bots.clone()
    }

    pub fn get_bot(&self, id: BotId) -> Option<Bot> {
        self.bots().into_iter().find(|bot| bot.id == id)
    }
}

impl<T: BotService + 'static> From<T> for BotRepo {
    fn from(service: T) -> Self {
        BotRepo(Arc::new(Mutex::new(InnerBotRepo {
            service: Box::new(service),
            bots: Vec::new(),
        })))
    }
}
