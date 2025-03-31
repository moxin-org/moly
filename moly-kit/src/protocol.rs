use futures::StreamExt;
use makepad_widgets::LiveValue;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    sync::{Arc, Mutex},
};

pub use crate::utils::asynchronous::{moly_future, moly_stream, MolyFuture, MolyStream};

use sha2::{Sha256, Digest};
use hex;

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
    /// Original model identifier from the provider
    pub model_id: String,
    /// Provider URL this bot belongs to
    pub provider_url: String,
    pub name: String,
    pub avatar: Picture,
}

/// Indentifies any kind of bot, local or remote, model or agent, whatever.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BotId(Arc<str>);

impl Serialize for BotId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BotId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(BotId::from(s.as_str()))
    }
}

impl From<&str> for BotId {
    fn from(id: &str) -> Self {
        BotId(id.into())
    }
}

impl BotId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a unique bot ID by hashing the provider URL and model ID
    pub fn new(model_id: &str, provider_url: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}-{}", model_id, provider_url));
        let result = hasher.finalize();
        // Take first 16 bytes of hash for a shorter but still unique identifier
        BotId::from(hex::encode(&result[..16]).as_str())
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A message that is part of a conversation.
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Message {
    /// The id of who sent this message.
    pub from: EntityId,

    /// Content of the message.
    pub body: String,
    /// Citations (e.g., URLs) to associate with the message
    pub citations: Vec<String>,
    /// Stages for multi-stage messages (like DeepInquire)
    pub stages: Vec<MessageStage>,
    /// Wether the message is actively being modified. False when no more changes are expected.
    pub is_writing: bool,
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

/// Content for a specific stage
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MessageBlockContent {
    /// Text content of the block
    pub content: String,
    /// Citations associated with this block
    pub citations: Vec<String>,
}

/// A stage in the message generation process
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum StageBlock {
    /// Thinking block, early reasoning steps
    Thinking {
        id: usize,
        content: String,
        citations: Vec<String>,
    },
    /// Writing block, formulating the response
    Writing {
        id: usize,
        content: String,
        citations: Vec<String>,
    },
    /// Completed stage, final response
    Completed {
        id: usize,
        content: String,
        citations: Vec<String>,
    },
}

impl StageBlock {
    /// Get the ID of this stage
    pub fn id(&self) -> usize {
        match self {
            StageBlock::Thinking { id, .. } => *id,
            StageBlock::Writing { id, .. } => *id,
            StageBlock::Completed { id, .. } => *id,
        }
    }
}

/// Delta for streaming responses
#[derive(Clone, Debug, PartialEq)]
pub struct MessageDelta {
    /// Delta for the [Message::body] field.
    pub body: String,
    /// Delta for the [Message::citations] field.
    pub citations: Vec<String>,
    /// Optional stage block information for multi-stage messages (DeepInquire)
    pub stage_block: Option<StageBlock>,
}

impl Message {
    /// Update the message with a delta
    pub fn apply_delta(&mut self, delta: MessageDelta) {
        // Always append to the body regardless of stage
        self.body.push_str(&delta.body);
        
        // Add any new citations
        for citation in delta.citations {
            if !self.citations.contains(&citation) {
                self.citations.push(citation);
            }
        }
        
        // If we have stage information, update the stages
        if let Some(stage_block) = delta.stage_block {
            self.update_stage(stage_block);
        }
    }   
    
    fn update_stage(&mut self, stage_block: StageBlock) {
        let stage_id = stage_block.id();
        
        // Check if we have this stage already
        let stage_exists = self.stages.iter().any(|s| s.id == stage_id);
        
        // If not, create a new one
        // We also always create a new last stage for completion
        // This is because currently DeepInquire returns id 1, but we want to make it the last stage.
        let is_block_completion = matches!(stage_block, StageBlock::Completed { .. });
        if !stage_exists || is_block_completion {
            self.stages.push(MessageStage {
                id: self.stages.len(),
                thinking: None,
                writing: None,
                completed: None,
            });
        }
        
        // Now get a mutable reference to the stage
        let stage_collection = self.stages.iter_mut()
            .find(|s| s.id == stage_id)
            .unwrap(); // Safe since we either found it or added it
        
        // Update the appropriate stage
        match stage_block {
            StageBlock::Thinking { id: _, content, citations } => {
                stage_collection.thinking = Some(MessageBlockContent {
                    content,
                    citations,
                });
            },
            StageBlock::Writing { id: _, content, citations } => {
                stage_collection.writing = Some(MessageBlockContent {
                    content,
                    citations,
                });
            },
            StageBlock::Completed { id: _, content, citations } => {
                stage_collection.completed = Some(MessageBlockContent {
                    content, 
                    citations,
                });
                // Mark message as completed when we get a completed stage
                self.is_writing = false;
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
            let mut body = String::new();
            let mut citations = Vec::new();
            let mut errors = Vec::new();

            let mut stream = stream;
            while let Some(result) = stream.next().await {
                let (v, e) = result.into_value_and_errors();

                if let Some(delta) = v {
                    body.push_str(&delta.body);
                    citations.extend(delta.citations);
                }

                if !e.is_empty() {
                    errors.extend(e);
                    break;
                }
            }

            if errors.is_empty() {
                ClientResult::new_ok(MessageDelta { body, citations, stage_block: None })
            } else {
                ClientResult::new_ok_and_err(MessageDelta { body, citations, stage_block: None }, errors)
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
