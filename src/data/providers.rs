use crate::data::bot_fetcher;
use makepad_widgets::*;
use moly_kit::{BotId, protocol::ClientError};
use serde::{Deserialize, Serialize};

pub type ProviderID = String;

/// Represents an AI provider
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provider {
    /// Unique identifier for the provider
    #[serde(default)]
    pub id: ProviderID,
    pub name: String,
    /// Refered as API host in the UI
    pub url: String,
    pub api_key: Option<String>,
    /// Determines the API format used by the provider
    pub provider_type: ProviderType,
    pub connection_status: ProviderConnectionStatus,
    pub enabled: bool,
    pub models: Vec<BotId>,
    /// Whether the provider was added by the user or not
    pub was_customly_added: bool,
    /// Custom system prompt for the provider (currently used by Realtime providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

/// Fetch models for a provider using MolyKit clients
pub fn fetch_models_for_provider(provider: &Provider) {
    bot_fetcher::fetch_models_for_provider(provider);
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderBot {
    pub id: BotId,
    pub name: String,
    pub description: String,
    pub provider_id: String,
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
            provider_id: "unknown".to_string(),
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
    Error(String), // Store error message as string for serialization
}

impl ProviderConnectionStatus {
    pub fn to_human_readable(&self) -> &str {
        match self {
            ProviderConnectionStatus::Connecting => "Connecting...",
            ProviderConnectionStatus::Connected => "Models synchronized",
            ProviderConnectionStatus::Disconnected => {
                "Haven't synchronized models since app launch"
            }
            ProviderConnectionStatus::Error(error_msg) => error_msg,
        }
    }

    /// Create an error status from a ClientError with a user-friendly message
    pub fn from_client_error(error: &ClientError) -> Self {
        let error_msg = error.message().to_lowercase();
        let error_string = error.to_string().to_lowercase();

        let user_message = match error.kind() {
            moly_kit::protocol::ClientErrorKind::Network => {
                if error_msg.contains("invalid url")
                    || error_msg.contains("invalid host")
                    || error_msg.contains("name resolution")
                {
                    "Invalid URL or hostname - please check your provider configuration".to_string()
                } else if error_msg.contains("connection refused") || error_msg.contains("refused")
                {
                    "Connection refused - check if the service is running and the port is correct"
                        .to_string()
                } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    "The server is taking too long to respond, please try again later".to_string()
                } else if error_msg.contains("ssl")
                    || error_msg.contains("tls")
                    || error_msg.contains("certificate")
                {
                    "SSL/TLS connection error - check if HTTPS is required or certificate is valid"
                        .to_string()
                } else {
                    "Network error - check your connection and URL".to_string()
                }
            }
            moly_kit::protocol::ClientErrorKind::Format => {
                "Something is wrong in our end, please file an issue if you think this is an error"
                    .to_string()
            }
            moly_kit::protocol::ClientErrorKind::Response => {
                if error_string.contains("401") || error_string.contains("unauthorized") {
                    "Unauthorized, check your API key".to_string()
                } else if error_string.contains("400") || error_string.contains("bad request") {
                    "Something is wrong in our end, please file an issue if you think this is an error".to_string()
                } else if error_string.contains("404") || error_string.contains("not found") {
                    "API endpoint not found - check your URL path".to_string()
                } else if error_string.contains("500")
                    || error_string.contains("502")
                    || error_string.contains("503")
                {
                    "We have trouble reaching the server".to_string()
                } else if error_string.contains("403") || error_string.contains("forbidden") {
                    "Access forbidden - check your API key permissions".to_string()
                } else if error_string.contains("429") || error_string.contains("rate limit") {
                    "Rate limit exceeded - please wait and try again".to_string()
                } else {
                    format!("Server error: {}", error.message())
                }
            }
            moly_kit::protocol::ClientErrorKind::Unknown => error.message().to_string(),
        };

        ProviderConnectionStatus::Error(user_message)
    }
}

#[derive(Debug, DefaultNone, Clone)]
pub enum ProviderFetchModelsResult {
    Success(ProviderID, Vec<ProviderBot>),
    Failure(ProviderID, ClientError),
    None,
}

#[derive(Live, LiveHook, PartialEq, Debug, LiveRead, Serialize, Deserialize, Clone)]
pub enum ProviderType {
    #[pick]
    OpenAI,
    OpenAIImage,
    OpenAIRealtime,
    MoFa,
    DeepInquire,
    MolyServer,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::OpenAI
    }
}
