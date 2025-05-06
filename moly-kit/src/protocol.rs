use futures::StreamExt;
use makepad_widgets::{Cx, LiveId, LivePtr, LiveValue, WidgetRef};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    sync::{Arc, Mutex},
};

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
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
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
    #[default]
    App,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Bot {
    /// Unique internal identifier for the bot across all providers
    pub id: BotId,
    pub name: String,
    pub avatar: Picture,
}

/// Identifies any kind of bot, local or remote, model or agent, whatever.
///
/// It MUST be globally unique and stable. It should be generated from a provider
/// local id and the domain or url of that provider.
///
/// For serialization, this is encoded as a single string.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct BotId(Arc<str>);

impl BotId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a new bot id from a provider local id and a provider domain or url.
    pub fn new(id: &str, provider: &str) -> Self {
        // The id is encoded as: <id_len>;<id>@<provider>.
        // `@` is simply a semantic separator, meaning (literally) "at".
        // The length is what is actually used for separating components allowing
        // these to include `@` characters.
        let id = format!("{};{}@{}", id.len(), id, provider);
        BotId(id.into())
    }

    fn deconstruct(&self) -> (usize, &str) {
        let (id_length, raw) = self.0.split_once(';').expect("malformed bot id");
        let id_length = id_length.parse::<usize>().expect("malformed bot id");
        (id_length, raw)
    }

    /// The id of the bot as it is known by its provider.
    ///
    /// This may not be globally unique.
    pub fn id(&self) -> &str {
        let (id_length, raw) = self.deconstruct();
        &raw[..id_length]
    }

    /// The provider component of this bot id.
    pub fn provider(&self) -> &str {
        let (id_length, raw) = self.deconstruct();
        // + 1 skips the semantic `@` separator
        &raw[id_length + 1..]
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Standard message content format.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MessageContent {
    /// The main body/document of this message.
    ///
    /// This would normally be written in somekind of document format like
    /// markdown, html, plain text, etc. Only markdown is expected by default.
    pub text: String,

    /// List of citations/sources (urls) associated with this message.
    pub citations: Vec<String>,

    /// Non-standard data contained by this message.
    ///
    /// May be used by clients for tracking purposes or to represent unsupported
    /// content.
    ///
    /// This is not expected to be used by most clients.
    // TODO: Using `String` for now because:
    //
    // - `Box<dyn Trait>` can't be `Deserialize`.
    // - `serde_json::Value` would force `serde_json` usage.
    // - `Vec<u8>` has unefficient serialization format and doesn't have many
    //   advantages over `String`.
    //
    // A wrapper type over Value and Box exposing a unified interface could be
    // a solution for later.
    pub data: Option<String>,
}

impl MessageContent {
    /// Checks if the content is absolutely empty (contains no data at all).
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.citations.is_empty() && self.data.is_none()
    }
}

/// A message that is part of a conversation.
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Message {
    /// The id of who sent this message.
    pub from: EntityId,

    /// The parsed content of this message ready to present.
    pub content: MessageContent,

    /// Whether the message is actively being modified.
    ///
    /// False when no more changes are expected.
    pub is_writing: bool,
}

/// The standard error kinds a client implementatiin should facilitate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientErrorKind {
    /// The network connection could not be established properly or was lost.
    Network,

    /// The connection could be established, but the remote server/peer gave us
    /// an error.
    ///
    /// Example: On a centralized HTTP server, this would happen when it returns
    /// an HTTP error code.
    Remote,

    /// The remote server/peer returned a successful response, but we can't parse
    /// its content.
    ///
    /// Example: When working with JSON APIs, this can happen when the schema of
    /// the JSON response is not what we expected or is not JSON at all.
    Format,

    /// A kind of error that is not contemplated by MolyKit at the client layer.
    Unknown,
}

impl ClientErrorKind {
    pub fn to_human_readable(&self) -> &str {
        match self {
            ClientErrorKind::Network => "Network error",
            ClientErrorKind::Remote => "Remote error",
            ClientErrorKind::Format => "Format error",
            ClientErrorKind::Unknown => "Unknown error",
        }
    }
}

/// Standard error returned from client operations.
#[derive(Debug, Clone)]
pub struct ClientError {
    kind: ClientErrorKind,
    message: String,
    source: Option<Arc<dyn Error + Send + Sync + 'static>>,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.to_human_readable(), self.message)
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|s| &**s as _)
    }
}

impl From<ClientError> for Vec<ClientError> {
    fn from(error: ClientError) -> Self {
        vec![error]
    }
}

impl<T> From<ClientError> for ClientResult<T> {
    fn from(error: ClientError) -> Self {
        ClientResult::new_err(vec![error])
    }
}

impl ClientError {
    /// Construct a simple client error without source.
    ///
    /// If you have an underlying error you want to include as the source, use
    /// [ClientError::new_with_source] instead.
    pub fn new(kind: ClientErrorKind, message: String) -> Self {
        ClientError {
            kind,
            message,
            source: None,
        }
    }

    /// Construct a client error using an underlying error as the source.
    pub fn new_with_source<S>(kind: ClientErrorKind, message: String, source: Option<S>) -> Self
    where
        S: Error + Send + Sync + 'static,
    {
        ClientError {
            kind,
            message,
            source: source.map(|s| Arc::new(s) as _),
        }
    }

    /// Error kind accessor.
    pub fn kind(&self) -> ClientErrorKind {
        self.kind
    }

    /// Error message accessor.
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

/// The outcome of a client operation.
///
/// Different from the standard Result, this one may contain more than one error.
/// And at the same time, even if an error ocurrs, there may be a value to rescue.
///
/// It would be mistake if this contains no value and no errors at the same time.
/// This is taken care on creation time, and it can't be modified afterwards.
pub struct ClientResult<T> {
    errors: Vec<ClientError>,
    value: Option<T>,
}

impl<T> ClientResult<T> {
    /// Creates a result containing a successful value and no errors.
    pub fn new_ok(value: T) -> Self {
        ClientResult {
            errors: Vec::new(),
            value: Some(value),
        }
    }

    /// Creates a result containing errors and no value to rescue.
    ///
    /// The errors list should be non empty. If it's empty a default error will
    /// be added to avoid the invariant of having no value and no errors at the
    /// same time.
    pub fn new_err(errors: Vec<ClientError>) -> Self {
        let errors = if errors.is_empty() {
            vec![ClientError::new(
                ClientErrorKind::Unknown,
                "An error ocurred, but no details were provided.".into(),
            )]
        } else {
            errors
        };

        ClientResult {
            errors,
            value: None,
        }
    }

    /// Creates a result containing errors and a value to rescue.
    ///
    /// This method should only be used when there are both errors and a value.
    /// - If there are no errors, use [ClientResult::new_ok] instead.
    /// - Similar to [ClientResult::new_err], if the errors list is empty, a default
    /// error will be added.
    pub fn new_ok_and_err(value: T, errors: Vec<ClientError>) -> Self {
        let errors = if errors.is_empty() {
            vec![ClientError::new(
                ClientErrorKind::Unknown,
                "An error ocurred, but no details were provided.".into(),
            )]
        } else {
            errors
        };

        ClientResult {
            errors,
            value: Some(value),
        }
    }

    /// Returns the successful value if there is one.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Returns the errors list.
    pub fn errors(&self) -> &[ClientError] {
        &self.errors
    }

    /// Returns true if there is a successful value.
    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    /// Returns true if there are errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Consume the result and return the successful value if there is one.
    pub fn into_value(self) -> Option<T> {
        self.value
    }

    /// Consume the result and return the errors list.
    pub fn into_errors(self) -> Vec<ClientError> {
        self.errors
    }

    /// Consume the result and return the successful value and the errors list.
    pub fn into_value_and_errors(self) -> (Option<T>, Vec<ClientError>) {
        (self.value, self.errors)
    }

    /// Consume the result to convert it into a standard Result.
    pub fn into_result(self) -> Result<T, Vec<ClientError>> {
        if self.errors.is_empty() {
            Ok(self.value.expect("ClientResult has no value nor errors"))
        } else {
            Err(self.errors)
        }
    }
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
    ///
    /// Each message yielded by the stream should be a snapshot of the full
    /// message as it is being built.
    ///
    /// You are free to add, modify or remove content on-the-go.
    fn send_stream(
        &mut self,
        bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>>;

    /// Interrupt the bot's current operation.
    // TODO: There may be many chats with the same bot/model/agent so maybe this
    // should be implemented by using cancellation tokens.
    // fn stop(&mut self, bot: BotId);

    /// Bots available under this client.
    // NOTE: Could be a stream, but may add complexity rarely needed.
    // TODO: Support partial results with errors for an union multi client/service
    // later.
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>>;

    /// Make a boxed dynamic clone of this client to pass around.
    fn clone_box(&self) -> Box<dyn BotClient>;

    /// Send a message to a bot expecting a full response at once.
    fn send(
        &mut self,
        bot: &Bot,
        messages: &[Message],
    ) -> MolyFuture<'static, ClientResult<MessageContent>> {
        let mut stream = self.send_stream(bot, messages);

        let future = async move {
            let mut content = MessageContent::default();
            let mut errors = Vec::new();

            while let Some(result) = stream.next().await {
                let (c, e) = result.into_value_and_errors();

                if let Some(c) = c {
                    content = c;
                }

                if !e.is_empty() {
                    errors = e;
                    break;
                }
            }

            if errors.is_empty() {
                ClientResult::new_ok(content)
            } else {
                ClientResult::new_ok_and_err(content, errors)
            }
        };

        moly_future(future)
    }

    /// Optionally override how the content of a message is rendered by Makepad.
    ///
    /// Not expected to be implemented by most clients, however if this client
    /// interfaces with a service that gives content in non-standard formats,
    /// this can be used to extend moly-kit to support it.
    fn content_widget(
        &mut self,
        _cx: &mut Cx,
        _previous_widget: WidgetRef,
        _templates: &HashMap<LiveId, LivePtr>,
        _content: &MessageContent,
    ) -> Option<WidgetRef> {
        None
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

    /// Fetches the bots and keeps them to be read synchronously later.
    ///
    /// It errors with whatever the underlying client errors with.
    pub fn load(&mut self) -> MolyFuture<ClientResult<()>> {
        let future = async move {
            let result = self.client().bots().await;
            let (new_bots, errors) = result.into_value_and_errors();

            if let Some(new_bots) = new_bots {
                self.0.lock().unwrap().bots = new_bots;
            }

            if errors.is_empty() {
                ClientResult::new_ok(())
            } else {
                ClientResult::new_err(errors)
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_id() {
        // Simple
        let id = BotId::new("123", "example.com");
        assert_eq!(id.as_str(), "3;123@example.com");
        assert_eq!(id.id(), "123");
        assert_eq!(id.provider(), "example.com");

        // Dirty
        let id = BotId::new("a;b@c", "https://ex@a@m;ple.co@m");
        assert_eq!(id.as_str(), "5;a;b@c@https://ex@a@m;ple.co@m");
        assert_eq!(id.id(), "a;b@c");
        assert_eq!(id.provider(), "https://ex@a@m;ple.co@m");

        // Similar yet different
        let id1 = BotId::new("a@", "b");
        let id2 = BotId::new("a", "@b");
        assert_ne!(id1.as_str(), id2.as_str());
        assert_ne!(id1, id2);
    }
}
