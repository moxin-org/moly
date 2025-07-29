use makepad_widgets::{Cx, LiveDependency, LiveId, LivePtr, WidgetRef};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    sync::{Arc, Mutex},
};

pub use crate::utils::asynchronous::{MolyFuture, MolyStream, moly_future, moly_stream};

mod attachment;
pub use attachment::*;
/// Upgrade types for enhanced communication modes
#[derive(Debug, Clone, PartialEq)]
pub enum Upgrade {
    /// Realtime audio/voice communication
    Realtime(RealtimeChannel),
}

/// Channel for realtime communication events
#[derive(Debug, Clone)]
pub struct RealtimeChannel {
    /// Sender for realtime events to the UI
    pub event_sender: futures::channel::mpsc::UnboundedSender<RealtimeEvent>,
    /// Receiver for realtime events from the client
    pub event_receiver:
        Arc<Mutex<Option<futures::channel::mpsc::UnboundedReceiver<RealtimeEvent>>>>,
    /// Sender for commands to the realtime client
    pub command_sender: futures::channel::mpsc::UnboundedSender<RealtimeCommand>,
}

impl PartialEq for RealtimeChannel {
    fn eq(&self, _other: &Self) -> bool {
        // For now, we'll consider all channels equal since we can't compare the actual channels
        true
    }
}

/// Events sent from the realtime client to the UI
#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    /// Session is ready for communication
    SessionReady,
    /// Session has been configured with settings
    SessionConfigured,
    /// Audio data received (PCM16 format)
    AudioData(Vec<u8>),
    /// Text transcript of received audio
    AudioTranscript(String),
    /// User started speaking
    SpeechStarted,
    /// User stopped speaking
    SpeechStopped,
    /// AI response completed
    ResponseCompleted,
    /// Error occurred
    Error(String),
}

/// Commands sent from the UI to the realtime client
#[derive(Debug, Clone)]
pub enum RealtimeCommand {
    /// Start the realtime session
    StartSession,
    /// Stop the realtime session
    StopSession,
    /// Send audio data (PCM16 format)
    SendAudio(Vec<u8>),
    /// Send text message
    SendText(String),
    /// Interrupt current AI response
    Interrupt,
    /// Configure interruption handling
    SetInterruptionEnabled(bool),
    /// Update session configuration
    UpdateSessionConfig {
        voice: String,
        transcription_model: String,
    },
    /// Create a greeting response from AI
    CreateGreetingResponse,
}

/// The picture/avatar of an entity that may be represented/encoded in different ways.
#[derive(Clone, Debug)]
pub enum Picture {
    // TODO: could be reduced to avoid allocation
    Grapheme(String),
    Image(String),
    // TODO: could be downed to a more concrete type
    Dependency(LiveDependency),
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

/// Represents the capabilities of a bot
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum BotCapability {
    /// Bot supports realtime audio communication
    Realtime,
    /// Bot supports image/file attachments
    Attachments,
    /// Bot supports function calling
    FunctionCalling,
}

/// Set of capabilities that a bot supports
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct BotCapabilities {
    capabilities: HashSet<BotCapability>,
}

impl BotCapabilities {
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }

    pub fn with_capability(mut self, capability: BotCapability) -> Self {
        self.capabilities.insert(capability);
        self
    }

    pub fn add_capability(&mut self, capability: BotCapability) {
        self.capabilities.insert(capability);
    }

    pub fn has_capability(&self, capability: &BotCapability) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn supports_realtime(&self) -> bool {
        self.has_capability(&BotCapability::Realtime)
    }

    pub fn supports_attachments(&self) -> bool {
        self.has_capability(&BotCapability::Attachments)
    }

    pub fn supports_function_calling(&self) -> bool {
        self.has_capability(&BotCapability::FunctionCalling)
    }

    pub fn iter(&self) -> impl Iterator<Item = &BotCapability> {
        self.capabilities.iter()
    }
}

#[derive(Clone, Debug)]
pub struct Bot {
    /// Unique internal identifier for the bot across all providers
    pub id: BotId,
    pub name: String,
    pub avatar: Picture,
    pub capabilities: BotCapabilities,
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

    /// The reasoning/thinking content of this message.
    #[cfg_attr(
        feature = "json",
        serde(deserialize_with = "crate::utils::serde::deserialize_default_on_error")
    )]
    pub reasoning: String,

    /// File attachments in this content.
    #[cfg_attr(feature = "json", serde(default))]
    pub attachments: Vec<Attachment>,

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

    /// Optional upgrade to realtime communication
    #[cfg_attr(feature = "json", serde(skip))]
    pub upgrade: Option<Upgrade>,
}

impl MessageContent {
    /// Checks if the content is absolutely empty (contains no data at all).
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.citations.is_empty()
            && self.data.is_none()
            && self.reasoning.is_empty()
            && self.attachments.is_empty()
            && self.upgrade.is_none()
    }
}

/// Metadata automatically tracked by MolyKit for each message.
///
/// "Metadata" basically means "data about data". Like tracking timestamps for
/// data modification.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MessageMetadata {
    /// Runtime flag indicating that the message is still incomplete (being written).
    ///
    /// Skipped during serialization.
    #[cfg_attr(feature = "json", serde(skip))]
    pub is_writing: bool,

    /// When the message got created.
    ///
    /// Default to epoch if missing during deserialization. Otherwise, if constructed
    /// by [`MessageMetadata::default`], it defaults to "now".
    #[cfg_attr(feature = "json", serde(default))]
    pub created_at: DateTime<Utc>,

    /// Last time the reasoning/thinking content was updated.
    ///
    /// Default to epoch if missing during deserialization. Otherwise, if constructed
    /// by [`MessageMetadata::default`], it defaults to "now".
    #[cfg_attr(feature = "json", serde(default))]
    pub reasoning_updated_at: DateTime<Utc>,

    /// Last time the main text was updated.
    ///
    /// Default to epoch if missing during deserialization. Otherwise, if constructed
    /// by [`MessageMetadata::default`], it defaults to "now".
    #[cfg_attr(feature = "json", serde(default))]
    pub text_updated_at: DateTime<Utc>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        // Use the same timestamp for all fields.
        let now = Utc::now();
        MessageMetadata {
            is_writing: false,
            created_at: now,
            reasoning_updated_at: now,
            text_updated_at: now,
        }
    }
}

impl MessageMetadata {
    /// Same behavior as [`MessageMetadata::default`].
    pub fn new() -> Self {
        MessageMetadata::default()
    }

    /// Create a new metadata with all fields set to default but timestamps set to epoch.
    pub fn epoch() -> Self {
        MessageMetadata {
            is_writing: false,
            created_at: DateTime::UNIX_EPOCH,
            reasoning_updated_at: DateTime::UNIX_EPOCH,
            text_updated_at: DateTime::UNIX_EPOCH,
        }
    }
}

impl MessageMetadata {
    /// The inferred amount of time the reasoning step took, in seconds (with milliseconds).
    pub fn reasoning_time_taken_seconds(&self) -> f32 {
        let delta = self.reasoning_updated_at - self.created_at;
        delta.as_seconds_f32()
    }

    pub fn is_idle(&self) -> bool {
        !self.is_writing
    }

    pub fn is_writing(&self) -> bool {
        self.is_writing
    }
}

/// A message that is part of a conversation.
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Message {
    /// The id of who sent this message.
    pub from: EntityId,

    /// Auto-generated metadata for this message.
    ///
    /// If missing during deserialization, uses [`MessageMetadata::epoch`] instead
    /// of [`MessageMetadata::default`].
    #[cfg_attr(feature = "json", serde(default = "MessageMetadata::epoch"))]
    pub metadata: MessageMetadata,

    /// The parsed content of this message ready to present.
    pub content: MessageContent,
}

impl Message {
    /// Set the content of a message as a whole (also updates metadata).
    pub fn set_content(&mut self, content: MessageContent) {
        self.update_content(|c| {
            *c = content;
        });
    }

    /// Update specific parts of the content of a message (also updates metadata).
    pub fn update_content(&mut self, f: impl FnOnce(&mut MessageContent)) {
        let bk = self.content.clone();
        let now = Utc::now();

        f(&mut self.content);

        if self.content.text != bk.text {
            self.metadata.text_updated_at = now;
        }

        if self.content.reasoning != bk.reasoning {
            self.metadata.reasoning_updated_at = now;
        }
    }
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
    Response,

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
            ClientErrorKind::Response => "Remote error",
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
#[derive(Debug)]
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
    /// Send a message to a bot with support for streamed response.
    ///
    /// Each message yielded by the stream should be a snapshot of the full
    /// message as it is being built.
    ///
    /// You are free to add, modify or remove content on-the-go.
    fn send(
        &mut self,
        bot_id: &BotId,
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

    /// Optionally override how the content of a message is rendered by Makepad.
    ///
    /// Not expected to be implemented by most clients, however if this client
    /// interfaces with a service that gives content in non-standard formats,
    /// this can be used to extend moly-kit to support it.
    ///
    /// Prefer reusing previous widget if matches the expected type instead of
    /// creating a new one on every call to preserve state and avoid perfomance
    /// issues.
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

struct InnerBotContext {
    client: Box<dyn BotClient>,
    bots: Vec<Bot>,
}

/// A sharable wrapper around a [BotClient] that holds loadeed bots and provides
/// synchronous APIs to access them, mainly from the UI.
///
/// Passed down through widgets from this crate.
///
/// Separate chat widgets can share the same [BotContext] to avoid loading the same
/// bots multiple times.
pub struct BotContext(Arc<Mutex<InnerBotContext>>);

impl Clone for BotContext {
    fn clone(&self) -> Self {
        BotContext(self.0.clone())
    }
}

impl PartialEq for BotContext {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl BotContext {
    /// Differenciates [BotContext]s.
    ///
    /// Two [BotContext]s are equal and share the same underlying data if they have
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

impl<T: BotClient + 'static> From<T> for BotContext {
    fn from(client: T) -> Self {
        BotContext(Arc::new(Mutex::new(InnerBotContext {
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
