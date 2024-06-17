use std::path::PathBuf;

use moxin_protocol::data::FileID;
use serde::{Deserialize, Serialize};

use super::filesystem::{
    project_dirs, read_from_file, setup_model_downloads_folder, write_to_file,
};
const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Preferences {
    pub current_chat_model: Option<FileID>,
    #[serde(default)]
    pub downloaded_files_dir: PathBuf,
}

impl Preferences {
    pub fn load() -> Self {
        match read_from_file(preferences_path()) {
            Ok(json) => {
                let mut preferences: Preferences = serde_json::from_str(&json).unwrap();
                preferences.downloaded_files_dir = setup_model_downloads_folder();
                dbg!(&json, preferences_path());
                return preferences;
            }
            Err(_) => {}
        }

        dbg!("Creating new preferences file");
        Self {
            current_chat_model: None,
            downloaded_files_dir: setup_model_downloads_folder(),
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string(&self).unwrap();
        write_to_file(preferences_path(), &json).unwrap();
    }

    pub fn set_current_chat_model(&mut self, file: FileID) {
        self.current_chat_model = Some(file);
        self.save();
    }
}

fn preferences_path() -> PathBuf {
    let preference_dir = project_dirs().preference_dir();
    preference_dir.join(PREFERENCES_FILENAME)
}
