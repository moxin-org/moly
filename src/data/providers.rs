use makepad_widgets::*;
use moly_kit::BotId;
use serde::{Deserialize, Serialize};

use super::{deep_inquire_client::DeepInquireClient, openai_client::OpenAIClient};

/// Represents an AI provider
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    /// Refered as API host in the UI, is used as an identifier for the provider
    pub url: String,
    pub api_key: Option<String>,
    /// Determines the API format used by the provider
    pub provider_type: ProviderType,
    pub connection_status: ProviderConnectionStatus,
    pub enabled: bool,
    pub models: Vec<BotId>,
    /// Whether the provider was added by the user or not
    pub was_customly_added: bool,
}

/// Creates a client for the provider based on the provider type
pub fn create_client_for_provider(provider: &Provider) -> Box<dyn ProviderClient> {
    match &provider.provider_type {
        ProviderType::OpenAI | ProviderType::MolyServer | ProviderType::MoFa => Box::new(
            OpenAIClient::new(provider.url.clone(), provider.api_key.clone()),
        ),
        ProviderType::DeepInquire => Box::new(DeepInquireClient::new(provider.url.clone())),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderBot {
    pub id: BotId,
    pub name: String,
    pub description: String,
    pub provider_url: String,
    pub enabled: bool,
}

impl ProviderBot {
    /// Returns a dummy provider bot whenever the corresponding provider bot cannot be found
    /// (due to the server not being available, the server no longer providing the provider bot, etc.).
    pub fn unknown() -> Self {
        ProviderBot {
            id: BotId::new("unknown", "unknown"),
            name: "Inaccesible model - check your connections".to_string(),
            description: "This model is not currently reachable, its information is not available"
                .to_string(),
            provider_url: "unknown".to_string(),
            enabled: true,
        }
    }

    pub fn human_readable_name(&self) -> &str {
        // Trim the 'models/' prefix from Gemini models
        // TODO: also trim and cleanup naming for filenames
        self.name.trim_start_matches("models/")
    }
}

/// The connection status of the server
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum ProviderConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Disconnected,
    Error(ProviderClientError),
}

impl ProviderConnectionStatus {
    pub fn to_human_readable(&self) -> &str {
        match self {
            ProviderConnectionStatus::Connecting => "Connecting...",
            ProviderConnectionStatus::Connected => "Models synchronized",
            ProviderConnectionStatus::Disconnected => {
                "Haven't synchronized models since app launch"
            }
            ProviderConnectionStatus::Error(error) => error.to_human_readable(),
        }
    }
}

#[derive(Debug, DefaultNone, Clone)]
pub enum ProviderFetchModelsResult {
    Success(String, Vec<ProviderBot>),
    Failure(String, ProviderClientError),
    None,
}

/// Errors that can occur when interacting with the provider client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderClientError {
    Unauthorized,
    BadRequest,
    UnexpectedResponse,
    InternalServerError,
    Timeout,
    Other(String),
}

impl ProviderClientError {
    pub fn to_human_readable(&self) -> &str {
        match self {
            ProviderClientError::Unauthorized => "Unauthorized, check your API key",
            ProviderClientError::BadRequest => {
                "Something is wrong in our end, please file an issue if you think this is an error"
            }
            ProviderClientError::UnexpectedResponse => "Unexpected Response",
            ProviderClientError::InternalServerError => "We have trouble reaching the server",
            ProviderClientError::Timeout => {
                "The server is taking too long to respond, please try again later"
            }
            ProviderClientError::Other(message) => message,
        }
    }
}

/// The behaviour that must be implemented by the provider clients.
pub trait ProviderClient: Send + Sync {
    fn fetch_models(&self);
}

#[derive(Live, LiveHook, PartialEq, Debug, LiveRead, Serialize, Deserialize, Clone)]
pub enum ProviderType {
    #[pick]
    OpenAI,
    MoFa,
    DeepInquire,
    MolyServer,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::OpenAI
    }
}
