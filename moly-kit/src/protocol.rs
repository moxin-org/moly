use futures::StreamExt;
use makepad_widgets::LiveValue;
use serde::{Deserialize, Serialize};
use std::{
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

/// Content types for messages, supporting different provider formats
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum MessageContent {
    /// Simple text content with optional citations
    PlainText {
        /// The text content
        text: String,
        /// Citations/sources associated with this content
        citations: Vec<String>,
    },

    /// Multi-stage content (like DeepInquire)
    MultiStage {
        /// Text representation
        text: String,
        /// Stages in various states
        stages: Vec<MessageStage>,
        /// Citations/sources associated with this content
        citations: Vec<String>,
    },
}

/// A message that is part of a conversation.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Message {
    /// The id of who sent this message.
    pub from: EntityId,

    /// The content of the message (text, stages, etc.)
    pub content: MessageContent,

    /// Whether the message is actively being modified. False when no more changes are expected.
    pub is_writing: bool,
}

impl Default for Message {
    fn default() -> Self {
        Message {
            from: EntityId::default(),
            content: MessageContent::PlainText {
                text: String::new(),
                citations: Vec::new(),
            },
            is_writing: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MessageStage {
    /// Stage identifier
    pub id: usize,
    /// Thinking block content
    pub thinking: Option<MessageBlockContent>,
    /// Writing block content
    pub writing: Option<MessageBlockContent>,
    /// Completed block content
    pub completed: Option<MessageBlockContent>,
}

impl MessageStage {
    /// Check if this stage has completed content
    pub fn is_completed(&self) -> bool {
        self.completed.is_some()
    }

    /// Get the text content of the most advanced stage (completed > writing > thinking)
    pub fn latest_content(&self) -> Option<&str> {
        if let Some(completed) = &self.completed {
            Some(&completed.content)
        } else if let Some(writing) = &self.writing {
            Some(&writing.content)
        } else if let Some(thinking) = &self.thinking {
            Some(&thinking.content)
        } else {
            None
        }
    }
}

/// Content for a specific stage
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MessageBlockContent {
    /// Text content of the block
    pub content: String,
    /// Citations associated with this block
    pub citations: Vec<String>,
}

/// Delta for streaming responses
#[derive(Clone, Debug, PartialEq)]
pub struct MessageDelta {
    /// The content delta
    pub content: MessageContent,
}

impl Message {
    /// Update the message with a delta
    pub fn apply_delta(&mut self, delta: MessageDelta) {
        match (&mut self.content, &delta.content) {
            // PlainText + PlainText -> append text and add citations
            (
                MessageContent::PlainText { text, citations },
                MessageContent::PlainText {
                    text: delta_text,
                    citations: delta_citations,
                },
            ) => {
                text.push_str(delta_text);
                for citation in delta_citations {
                    if !citations.contains(citation) {
                        citations.push(citation.clone());
                    }
                }
            }

            // MultiStage + MultiStage -> update stages and append text
            (
                MessageContent::MultiStage {
                    text,
                    stages,
                    citations,
                },
                MessageContent::MultiStage {
                    text: delta_text,
                    stages: delta_stages,
                    citations: delta_citations,
                },
            ) => {
                // Append text if not empty
                if !delta_text.is_empty() {
                    text.push_str(delta_text);
                }

                // Merge stages from delta into existing stages
                for new_stage in delta_stages {
                    if let Some(existing_stage) = stages.iter_mut().find(|s| s.id == new_stage.id) {
                        // Update existing stage
                        if new_stage.thinking.is_some() {
                            existing_stage.thinking = new_stage.thinking.clone();
                        }
                        if new_stage.writing.is_some() {
                            existing_stage.writing = new_stage.writing.clone();
                        }
                        if new_stage.completed.is_some() {
                            existing_stage.completed = new_stage.completed.clone();
                            // Mark message as completed when we get a completed stage
                            self.is_writing = false;
                        }
                    } else {
                        // Add new stage
                        stages.push(new_stage.clone());
                    }
                }

                // Add new citations
                for citation in delta_citations {
                    if !citations.contains(citation) {
                        citations.push(citation.clone());
                    }
                }
            }

            // PlainText + MultiStage -> convert to MultiStage and update
            (
                MessageContent::PlainText {
                    text: existing_text,
                    citations: existing_citations,
                },
                MessageContent::MultiStage {
                    text: delta_text,
                    stages: delta_stages,
                    citations: delta_citations,
                },
            ) => {
                let mut combined_text = existing_text.clone();
                if !delta_text.is_empty() {
                    combined_text.push_str(delta_text);
                }

                let mut combined_citations = existing_citations.clone();
                for citation in delta_citations {
                    if !combined_citations.contains(citation) {
                        combined_citations.push(citation.clone());
                    }
                }

                // Convert to MultiStage
                self.content = MessageContent::MultiStage {
                    text: combined_text,
                    stages: delta_stages.clone(),
                    citations: combined_citations,
                };
            }

            // MultiStage + PlainText -> just append text and citations
            (
                MessageContent::MultiStage {
                    text, citations, ..
                },
                MessageContent::PlainText {
                    text: delta_text,
                    citations: delta_citations,
                },
            ) => {
                text.push_str(delta_text);

                for citation in delta_citations {
                    if !citations.contains(citation) {
                        citations.push(citation.clone());
                    }
                }
            }
        }
    }

    /// Gets the visible text content to display, regardless of the content type
    pub fn visible_text(&self) -> String {
        match &self.content {
            MessageContent::PlainText { text, .. } => text.clone(),
            MessageContent::MultiStage { text, .. } => text.clone(),
        }
    }

    /// Gets the citations/sources regardless of the content type
    pub fn sources(&self) -> Vec<String> {
        match &self.content {
            MessageContent::PlainText { citations, .. } => citations.clone(),
            MessageContent::MultiStage { citations, .. } => citations.clone(),
        }
    }

    /// Checks if this message has multi-stage content
    pub fn has_stages(&self) -> bool {
        match &self.content {
            MessageContent::PlainText { .. } => false,
            MessageContent::MultiStage { stages, .. } => !stages.is_empty(),
        }
    }

    /// Gets the stages if this message has multi-stage content
    pub fn get_stages(&self) -> Vec<MessageStage> {
        match &self.content {
            MessageContent::PlainText { .. } => Vec::new(),
            MessageContent::MultiStage { stages, .. } => stages.clone(),
        }
    }
}

/// Factory methods for creating properly formatted MessageDelta objects
pub trait MessageDeltaFactory {
    /// Create a text-only delta with optional citations
    fn text_delta(text: String, citations: Vec<String>) -> MessageDelta;

    /// Create a stage-based delta
    fn stage_delta(text: String, stage: MessageStage, citations: Vec<String>) -> MessageDelta;
}

impl MessageDeltaFactory for MessageDelta {
    fn text_delta(text: String, citations: Vec<String>) -> Self {
        MessageDelta {
            content: MessageContent::PlainText { text, citations },
        }
    }

    fn stage_delta(text: String, stage: MessageStage, citations: Vec<String>) -> Self {
        MessageDelta {
            content: MessageContent::MultiStage {
                text,
                stages: vec![stage],
                citations,
            },
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
    fn send_stream(
        &mut self,
        bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageDelta>>;

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
    ) -> MolyFuture<'static, ClientResult<MessageDelta>> {
        let stream = self.send_stream(bot, messages);

        let future = async move {
            let mut text = String::new();
            let mut citations = Vec::new();
            let mut errors = Vec::new();

            let mut stream = stream;
            while let Some(result) = stream.next().await {
                let (v, e) = result.into_value_and_errors();

                if let Some(delta) = v {
                    match &delta.content {
                        MessageContent::PlainText {
                            text: delta_text,
                            citations: delta_citations,
                        } => {
                            text.push_str(delta_text);
                            citations.extend(delta_citations.clone());
                        }
                        MessageContent::MultiStage {
                            text: delta_text,
                            citations: delta_citations,
                            ..
                        } => {
                            text.push_str(delta_text);
                            citations.extend(delta_citations.clone());
                        }
                    }
                }

                if !e.is_empty() {
                    errors.extend(e);
                    break;
                }
            }

            if errors.is_empty() {
                ClientResult::new_ok(MessageDelta {
                    content: MessageContent::PlainText { text, citations },
                })
            } else {
                ClientResult::new_ok_and_err(
                    MessageDelta {
                        content: MessageContent::PlainText { text, citations },
                    },
                    errors,
                )
            }
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
