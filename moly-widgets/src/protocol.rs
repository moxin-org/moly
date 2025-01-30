// This is the stream type re-exported by tokio, reqwest and futures.
use futures::{future, stream};
use makepad_widgets::LiveValue;

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
pub trait BotRepo: Send {
    /// Get ready and pull the available bots list.
    fn load(&mut self) -> BoxFuture<Result<(), ()>>;

    /// Send a message to a bot expecting a full response at once.
    // TODO: messages may end up being a little bit more complex, using string while thinking.
    // TOOD: Should support a way of passing, unknown, backend-specific, inference parameters.
    fn send(&mut self, bot: BotId, messages: &[Message]) -> BoxFuture<Result<String, ()>>;

    /// Send a message to a bot expecting a streamed response.
    fn send_stream(&mut self, bot: BotId, messages: &[Message]) -> BoxStream<Result<String, ()>>;

    /// Interrupt the bot's current operation.
    // TODO: There may be many chats with the same bot/model/agent so maybe this
    // should be implemented by using cancellation tokens.
    // fn stop(&mut self, bot: BotId);

    /// Bots available under this client.
    // TODO: Should be a stream actually?
    fn bots(&self) -> Box<dyn Iterator<Item = Bot>>;

    /// Get a bot by its id.
    // TODO: What if you want to pull remote to get this? What if you don't have
    // it inside the struct? Would make sense to return something owned and async?
    // Would make sense for `Bot` to be a trait instead of just a data struct?
    fn get_bot(&self, id: BotId) -> Option<Bot>;

    /// Make a boxed dynamic clone of this client to pass around.
    fn clone_box(&self) -> Box<dyn BotRepo>;
}
