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

/// Indentify the entities that are recognized by this crate, mainly in a chat.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EntityId {
    /// Represents the user operating this app.
    User,

    /// Represents the `system`/`developer` expected by many LLMs in the chat
    /// context to customize the chat experience and behavior.
    System,

    /// Represents a bot, which is an automated assistant of any kind (model, agent, etc).
    Bot(BotId),

    /// This app itself. Normally appears when app specific information must be displayed
    /// (like inline errors).
    ///
    /// It's not supposed to be sent as part of a conversation to bots.
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
    /// Citations for the message.
    pub citations: Vec<String>,
}

/// A new structure to hold both text delta and optional metadata like citations.
#[derive(Clone, Debug)]
pub struct ChatDelta {
    pub content_delta: String,
    pub citations: Option<Vec<String>>,
}

/// A standard interface to fetch bots information and send messages to them.
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
    ) -> MolyStream<'static, Result<ChatDelta, ()>>;

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
    fn send(
        &mut self,
        bot: &BotId,
        messages: &[Message],
    ) -> MolyFuture<'static, Result<Message, ()>> {
        let stream = self.send_stream(bot, messages);
        let bot = bot.clone();

        let future = async move {
            let mut content = String::new();
            let mut citations = Vec::new();

            let mut stream = stream;
            while let Some(delta) = stream.next().await {
                match delta {
                    Ok(chat_delta) => {
                        content.push_str(&chat_delta.content_delta);
                        if let Some(cits) = chat_delta.citations {
                            citations = cits;
                        }
                    }
                    Err(()) => return Err(()),
                }
            }

            Ok(Message {
                from: EntityId::Bot(bot),
                body: content,
                is_writing: false,
                citations,
            })
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

/// A sharable wrapper around a [BotClient] that holds loadeed bots and provides
/// synchronous APIs to access them, mainly from the UI.
///
/// Passed down through widgets from this crate.
///
/// Separate chat widgets can share the same [BotRepo] to avoid loading the same
/// bots multiple times.
pub struct BotRepo(Arc<Mutex<InnerBotRepo>>);

impl Clone for BotRepo {
    fn clone(&self) -> Self {
        BotRepo(self.0.clone())
    }
}

impl PartialEq for BotRepo {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl BotRepo {
    /// Differenciates [BotRepo]s.
    ///
    /// Two [BotRepo]s are equal and share the same underlying data if they have
    /// the same id.
    pub fn id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

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
