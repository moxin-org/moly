use std::path::PathBuf;

use moly_kit::BotId;
use serde::{Deserialize, Serialize};

use super::{filesystem::{
    read_from_file, setup_model_downloads_folder, setup_preferences_folder, write_to_file
}, providers::{Provider, ProviderType}};

const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Preferences {
    pub current_chat_model: Option<BotId>,
    #[serde(default)]
    pub downloaded_files_dir: PathBuf,
    #[serde(default)]
    pub providers_preferences: Vec<ProviderPreferences>,
}

impl Preferences {
    pub fn load() -> Self {
        let preferences_path = preferences_path();

        if let Ok(json) = read_from_file(preferences_path) {
            if let Ok(mut preferences) = serde_json::from_str::<Preferences>(&json) {
                // Check if the downloaded_files_dir exists, if not, create it
                if !preferences.downloaded_files_dir.exists() {
                    preferences.downloaded_files_dir = setup_model_downloads_folder();
                }
                return preferences;
            }
        }

        // If no preferences file exists, create default preferences
        Self {
            current_chat_model: None,
            downloaded_files_dir: setup_model_downloads_folder(),
            providers_preferences: vec![],
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string(&self).unwrap();
        match write_to_file(preferences_path(), &json) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to write to the file: {:?}", e),
        }
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
        if let Some(existing_provider) = self.providers_preferences.iter_mut().find(|p| p.url == provider.url) {
            existing_provider.api_key = provider.api_key.clone();
            existing_provider.enabled = provider.enabled;
        } else {
            self.providers_preferences.push(ProviderPreferences {
                name: provider.name.clone(),
                url: provider.url.clone(),
                api_key: provider.api_key.clone(),
                enabled: provider.enabled,
                provider_type: provider.provider_type.clone(),
                models: provider.models.iter().map(|m| (m.as_str().to_string(), true)).collect(),
                was_customly_added: provider.was_customly_added,
            });
        }
        self.save();
    }

    pub fn remove_provider(&mut self, address: &str) {
        self.providers_preferences
            .retain(|p| p.url != address);
        self.save();
    }

    /// Update the enabled/disabled status of a model for a specific server
    pub fn update_model_status(&mut self, address: &str, model_name: &str, enabled: bool) {
        if let Some(provider) = self.providers_preferences.iter_mut().find(|p| p.url == address) {
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
    let preference_dir = setup_preferences_folder();
    preference_dir.join(PREFERENCES_FILENAME)
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
