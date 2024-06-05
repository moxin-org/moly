use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use moxin_protocol::data::FileID;
use serde::{Deserialize, Serialize};

use super::filesystem::{project_dirs, setup_model_downloads_folder};
const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Preferences {
    pub current_chat_model: Option<FileID>,
    #[serde(default)]
    pub downloaded_files_dir: PathBuf,
}

impl Preferences {
    pub fn load() -> Self {
        match read_from_file() {
            Ok(json) => {
                let mut preferences: Preferences = serde_json::from_str(&json).unwrap();
                preferences.downloaded_files_dir = setup_model_downloads_folder();
                return preferences;
            }
            Err(_) => {}
        }

        Self {
            current_chat_model: None,
            downloaded_files_dir: setup_model_downloads_folder(),
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string(&self).unwrap();
        write_to_file(&json).unwrap();
    }

    pub fn set_current_chat_model(&mut self, file: FileID) {
        self.current_chat_model = Some(file);
        self.save();
    }
}

fn read_from_file() -> Result<String, std::io::Error> {
    let path = preferences_path();

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(why) => return Err(why),
    };

    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => Ok(json),
        Err(why) => Err(why),
    }
}

fn write_to_file(json: &str) -> Result<(), std::io::Error> {
    let path = preferences_path();

    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(why) => return Err(why),
    };

    match file.write_all(json.as_bytes()) {
        Ok(_) => Ok(()),
        Err(why) => Err(why),
    }
}

fn preferences_path() -> PathBuf {
    let preference_dir = project_dirs().preference_dir();
    preference_dir.join(PREFERENCES_FILENAME)
}
