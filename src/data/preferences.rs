use std::path::PathBuf;

use moly_protocol::data::FileID;
use serde::{Deserialize, Serialize};

use super::{filesystem::{
    read_from_file, setup_model_downloads_folder, setup_preferences_folder, write_to_file
}, store::ProviderType};

const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerModel {
    pub name: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnection {
    pub address: String,
    pub provider: ProviderType,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub models: Vec<ServerModel>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Preferences {
    pub current_chat_model: Option<FileID>,
    #[serde(default)]
    pub downloaded_files_dir: PathBuf,
    #[serde(default)]
    pub server_connections: Vec<ServerConnection>,
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
            server_connections: vec![],
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string(&self).unwrap();
        match write_to_file(preferences_path(), &json) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to write to the file: {:?}", e),
        }
    }

    pub fn set_current_chat_model(&mut self, file: FileID) {
        self.current_chat_model = Some(file);
        self.save();
    }

    pub fn _set_downloaded_files_dir(&mut self, path: PathBuf) {
        self.downloaded_files_dir = path;
        self.save();
    }

    pub fn add_or_update_server_connection(
        &mut self,
        provider: ProviderType,
        address: String,
        api_key: Option<String>,
    ) {
        // Remove existing entry if it exists:
        if let Some(pos) = self
            .server_connections
            .iter()
            .position(|sc| sc.address == address)
        {
            self.server_connections.remove(pos);
        }
        // Add the new connection
        self.server_connections.push(ServerConnection {
            address,
            provider,
            api_key,
            models: vec![], // start empty; populate later if needed
        });
        self.save();
    }

    pub fn remove_server_connection(&mut self, address: &str) {
        self.server_connections
            .retain(|sc| sc.address != address);
        self.save();
    }

    /// Refresh or insert a model in the server's model list.
    pub fn _ensure_server_model_exists(&mut self, address: &str, model_name: &str) {
        if let Some(conn) = self.server_connections.iter_mut().find(|sc| sc.address == address) {
            let already_exists = conn.models.iter().any(|m| m.name == model_name);
            if !already_exists {
                conn.models.push(ServerModel {
                    name: model_name.to_string(),
                    enabled: true,
                });
            }
        }
    }

    /// Update the enabled/disabled status of a model for a specific server
    pub fn update_model_status(&mut self, address: &str, model_name: &str, enabled: bool) {
        if let Some(conn) = self.server_connections.iter_mut().find(|sc| sc.address == address) {
            if let Some(model) = conn.models.iter_mut().find(|m| m.name == model_name) {
                model.enabled = enabled;
            } else {
                // If not found, add it
                conn.models.push(ServerModel {
                    name: model_name.to_string(),
                    enabled,
                });
            }
        }
        self.save();
    }
}

fn preferences_path() -> PathBuf {
    let preference_dir = setup_preferences_folder();
    preference_dir.join(PREFERENCES_FILENAME)
}
