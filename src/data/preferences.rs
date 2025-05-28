use moly_kit::{utils::asynchronous::spawn, BotId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::shared::utils::filesystem;

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
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            current_chat_model: None,
            downloaded_files_dir: default_model_downloads_dir().to_path_buf(),
            providers_preferences: vec![],
        }
    }
}

impl Preferences {
    pub async fn load() -> Self {
        let preferences_path = preferences_path();
        let fs = filesystem::global();
        match fs.read_json::<Preferences>(&preferences_path).await {
            Ok(preferences) => preferences,
            Err(e) => {
                eprintln!("Failed to read preferences file: {:?}", e);
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
                Err(e) => eprintln!("Failed to write to the file: {:?}", e),
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
            .find(|p| p.url == provider.url)
        {
            existing_provider.api_key = provider.api_key.clone();
            existing_provider.enabled = provider.enabled;
        } else {
            self.providers_preferences.push(ProviderPreferences {
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
            });
        }
        self.save();
    }

    pub fn remove_provider(&mut self, address: &str) {
        self.providers_preferences.retain(|p| p.url != address);
        self.save();
    }

    /// Update the enabled/disabled status of a model for a specific server
    pub fn update_model_status(&mut self, address: &str, model_name: &str, enabled: bool) {
        if let Some(provider) = self
            .providers_preferences
            .iter_mut()
            .find(|p| p.url == address)
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
}

fn preferences_path() -> PathBuf {
    Path::new(PREFERENCES_DIR).join(PREFERENCES_FILENAME)
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProviderPreferences {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub enabled: bool,
    pub provider_type: ProviderType,
    // (model_name, enabled)
    pub models: Vec<(String, bool)>,
    pub was_customly_added: bool,
}

fn default_model_downloads_dir() -> &'static Path {
    Path::new("model_downloads")
}
