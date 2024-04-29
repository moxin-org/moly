use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use moxin_protocol::data::FileID;
use serde::{Deserialize, Serialize};

use super::filesystem::moxin_home_dir;
const PREFERENCES_FILE: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Preferences {
    pub current_chat_model: Option<FileID>,
}

impl Preferences {
    pub fn load() -> Self {
        match read_from_file() {
            Ok(json) => {
                let preferences: Preferences = serde_json::from_str(&json).unwrap();
                return preferences;
            }
            Err(_) => {}
        }

        Self {
            current_chat_model: None,
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
    let home_dir = moxin_home_dir();
    let preferences_path = PathBuf::from(home_dir).join(PREFERENCES_FILE);
    preferences_path
}
