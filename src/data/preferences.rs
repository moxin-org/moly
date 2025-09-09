use moly_kit::{BotId, utils::asynchronous::spawn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::data::providers::ProviderID;
use crate::shared::utils::filesystem;

use super::mcp_servers::McpServersConfig;
use super::providers::{Provider, ProviderType};

const PREFERENCES_DIR: &str = "preferences";
const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preferences {
    pub current_chat_model: Option<BotId>,
    #[serde(default)]
    pub downloaded_files_dir: PathBuf,
    #[serde(default)]
    pub providers_preferences: Vec<ProviderPreferences>,
    #[serde(default)]
    pub mcp_servers_config: McpServersConfig,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            current_chat_model: None,
            downloaded_files_dir: default_model_downloads_dir().to_path_buf(),
            providers_preferences: vec![],
            mcp_servers_config: McpServersConfig::new(),
        }
    }
}

impl Preferences {
    pub async fn load() -> Self {
        let preferences_path = preferences_path();
        let fs = filesystem::global();
        match fs.read_json::<Preferences>(&preferences_path).await {
            Ok(mut preferences) => {
                // Migrate providers without IDs
                preferences.migrate_provider_ids();
                preferences
            }
            Err(_e) => {
                log::info!("No preferences file found, a default one will be created.");
                Preferences::default()
            }
        }
    }

    pub fn save(&self) {
        let self_clone = self.clone();
        spawn(async move {
            match filesystem::global()
                .queue_write_json(preferences_path(), &self_clone)
                .await
            {
                Ok(()) => (),
                Err(e) => log::error!("Failed to write preferences file: {:?}", e),
            }
        });
    }

    pub fn set_current_chat_model(&mut self, bot_id: Option<BotId>) {
        self.current_chat_model = bot_id;
        self.save();
    }

    pub fn _set_downloaded_files_dir(&mut self, path: PathBuf) {
        self.downloaded_files_dir = path;
        self.save();
    }

    pub fn insert_or_update_provider(&mut self, provider: &Provider) {
        if let Some(existing_provider) = self
            .providers_preferences
            .iter_mut()
            .find(|p| p.id == provider.id || (p.id.is_empty() && p.url == provider.url))
        {
            existing_provider.id = provider.id.clone();
            existing_provider.url = provider.url.clone();
            existing_provider.api_key = provider.api_key.clone();
            existing_provider.enabled = provider.enabled;
            existing_provider.system_prompt = provider.system_prompt.clone();
            existing_provider.tools_enabled = provider.tools_enabled;
        } else {
            self.providers_preferences.push(ProviderPreferences {
                id: provider.id.clone(),
                name: provider.name.clone(),
                url: provider.url.clone(),
                api_key: provider.api_key.clone(),
                enabled: provider.enabled,
                provider_type: provider.provider_type.clone(),
                models: provider
                    .models
                    .iter()
                    .map(|m| (m.as_str().to_string(), true))
                    .collect(),
                was_customly_added: provider.was_customly_added,
                system_prompt: provider.system_prompt.clone(),
                tools_enabled: provider.tools_enabled,
            });
        }
        self.save();
    }

    pub fn remove_provider(&mut self, provider_id: &ProviderID) {
        self.providers_preferences.retain(|p| &p.id != provider_id);
        self.save();
    }

    /// Update the enabled/disabled status of a model for a specific server
    pub fn update_model_status(
        &mut self,
        provider_id: &ProviderID,
        model_name: &str,
        enabled: bool,
    ) {
        if let Some(provider) = self
            .providers_preferences
            .iter_mut()
            .find(|p| &p.id == provider_id)
        {
            if let Some(model) = provider.models.iter_mut().find(|m| m.0 == model_name) {
                model.1 = enabled;
            } else {
                // If not found, add it
                provider.models.push((model_name.to_string(), enabled));
            }
        }
        self.save();
    }

    /// Import preferences from a JSON string
    ///
    /// If merge is true, the provider preferences will be extended with the new ones,
    /// otherwise, the existing preferences will be replaced.
    ///
    /// If include_mcp_servers is true, the MCP servers will be included in the import, replacing the existing ones.
    pub fn import_from_json(
        &mut self,
        json: &str,
        merge: bool,
        include_mcp_servers: bool,
    ) -> Result<(), serde_json::Error> {
        let preferences = serde_json::from_str::<Preferences>(json)?;
        if merge {
            self.providers_preferences
                .extend(preferences.providers_preferences.clone());
        } else {
            self.providers_preferences = preferences.providers_preferences.clone();
        }

        if include_mcp_servers {
            self.mcp_servers_config = preferences.mcp_servers_config;
        }

        self.save();
        Ok(())
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn get_mcp_servers_config_json(&self) -> String {
        self.mcp_servers_config
            .to_json()
            .unwrap_or_else(|_| "{}".to_string())
    }

    pub fn update_mcp_servers_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let config = McpServersConfig::from_json(json)?;
        self.mcp_servers_config = config;
        self.save();
        Ok(())
    }

    pub fn set_mcp_servers_enabled(&mut self, enabled: bool) {
        self.mcp_servers_config.enabled = enabled;
        self.save();
    }

    pub fn get_mcp_servers_enabled(&self) -> bool {
        self.mcp_servers_config.enabled
    }

    /// Migrate providers without IDs by generating them from URLs
    fn migrate_provider_ids(&mut self) {
        let mut needs_save = false;
        for provider in &mut self.providers_preferences {
            if provider.id.is_empty() {
                provider.ensure_id();
                needs_save = true;
            }
        }
        if needs_save {
            self.save();
        }
    }
}

fn preferences_path() -> PathBuf {
    Path::new(PREFERENCES_DIR).join(PREFERENCES_FILENAME)
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProviderPreferences {
    /// Unique identifier for the provider
    #[serde(default)]
    pub id: ProviderID,
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub enabled: bool,
    pub provider_type: ProviderType,
    // (model_name, enabled)
    pub models: Vec<(String, bool)>,
    pub was_customly_added: bool,
    /// Custom system prompt for the provider (currently used by Realtime providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Whether tools (MCP) are enabled for this provider
    #[serde(default = "default_tools_enabled")]
    pub tools_enabled: bool,
}

fn default_tools_enabled() -> bool {
    true
}

impl ProviderPreferences {
    /// Ensure this provider has an ID, generating one if needed
    pub fn ensure_id(&mut self) {
        if self.id.is_empty() {
            self.id =
                Self::generate_id_from_url_and_name(&self.url, &self.name, &self.provider_type);
        }
    }

    /// Generate a stable ID from URL and name for migration
    pub fn generate_id_from_url_and_name(
        url: &str,
        name: &str,
        provider_type: &ProviderType,
    ) -> String {
        // For known built-in providers, use predefined IDs
        match url {
            "https://api.openai.com/v1" => match provider_type {
                ProviderType::OpenAI => "openai_chat".to_string(),
                _ => "openai_chat".to_string(),
            },
            "#https://api.openai.com/v1" => "openai_image".to_string(),
            "wss://api.openai.com/v1/realtime" => "openai_realtime".to_string(),
            "ws://127.0.0.1:8123" => "dora_realtime".to_string(),
            "https://generativelanguage.googleapis.com/v1beta/openai" => "gemini".to_string(),
            "http://127.0.0.1:8000/v3" => "deepinquire".to_string(),
            "https://api.siliconflow.cn/v1" => "siliconflow".to_string(),
            "https://openrouter.ai/api/v1" => "openrouter".to_string(),
            "http://localhost:8765/api/v1" => "molyserver".to_string(),
            "https://api.deepseek.com/v1" => "deepseek".to_string(),
            _ => {
                // For custom providers, create ID from name
                let base = name
                    .to_lowercase()
                    .replace(" ", "_")
                    .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
                if base.is_empty() {
                    "custom_provider".to_string()
                } else {
                    base
                }
            }
        }
    }
}

fn default_model_downloads_dir() -> &'static Path {
    Path::new("model_downloads")
}
