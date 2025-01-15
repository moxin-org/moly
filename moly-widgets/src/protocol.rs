// This is the stream type re-exported by tokio, reqwest and futures.
use futures_core::Stream;
use makepad_widgets::LiveValue;
use std::future::Future;

/// The picture/avatar of an entity that may be represented/encoded in different ways.
#[derive(Clone, PartialEq, Debug)]
pub enum Avatar {
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
    Bot(BotId),
}

pub trait Bot {
    /// Identifier for this bot. Read `BotId` documentation for more information.
    fn id(&self) -> BotId;

    /// The human-readable name of this bot.
    fn name(&self) -> &str;

    /// The avatar of this bot. Read `Avatar` documentation for more information.
    fn avatar(&self) -> &Avatar;
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
pub struct Message {
    /// The id of who sent this message.
    pub from: EntityId,
    /// Content of the message.
    pub body: String,
}

/// An interface to talk to bots.
///
/// Warning: Expect this to be cloned to avoid borrow checking issues with
/// makepad's widgets. Also, it may be cloned inside async contexts. So keep this
/// cheap to clone and synced.
///
/// Note: Generics do not play well with makepad's widgets, so this trait relies
/// on dynamic dispatch (with its limitations).
pub trait BotClient {
    /// Send a message to a bot expecting a full response at once.
    // TODO: messages may end up being a little bit more complex, using string while thinking.
    fn send(&mut self, bot: BotId, message: &str) -> Box<dyn Future<Output = Result<String, ()>>>;

    /// Send a message to a bot expecting a streamed response.
    fn send_stream(
        &mut self,
        bot: BotId,
        message: &str,
    ) -> Box<dyn Stream<Item = Result<String, ()>>>;

    /// Interrupt the bot's current operation.
    fn stop(&mut self, bot: BotId);

    /// Bots available under this client.
    fn bots(&self) -> Box<dyn Iterator<Item = &dyn Bot> + '_>;

    /// Get a bot by its id.
    fn get_bot(&self, id: BotId) -> Option<&dyn Bot>;

    /// Get a bot by its id mutably.
    fn get_bot_mut(&mut self, id: BotId) -> Option<&mut dyn Bot>;

    /// Make a boxed dynamic clone of this client to pass around.
    fn clone_box(&self) -> Box<dyn BotClient>;
}
