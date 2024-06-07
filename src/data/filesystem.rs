use std::path::PathBuf;
use std::{env, fs};

// Note that .moxin will create a hidden folder in unix-like systems.
// However in Windows the folder will be visible by default.
pub const DEFAULT_DOWNLOADS_DIR: &str = ".moxin/model_downloads";
pub const MOXIN_HOME_DIR: &str = ".moxin";

pub fn setup_model_downloads_folder() -> String {
    let home_dir = home_dir();
    let downloads_dir = PathBuf::from(home_dir).join(DEFAULT_DOWNLOADS_DIR);

    if fs::create_dir_all(&downloads_dir).is_err() {
        eprintln!(
            "Failed to create the model downloads directory at '{}'. Using current directory as fallback.",
            downloads_dir.display()
        );
        ".".to_string()
    } else {
        downloads_dir.to_string_lossy().to_string()
    }
}

fn home_dir() -> String {
    // TODO: FIXME: use directories::ProjectDirs::data_dir() instead.
    // <https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.data_dir>
    env::var("HOME") // Unix-like systems
        .or_else(|_| env::var("USERPROFILE")) // Windows
        .unwrap_or_else(|_| ".".to_string())
}

pub fn moxin_home_dir() -> PathBuf {
    let home_dir = home_dir();
    PathBuf::from(home_dir).join(MOXIN_HOME_DIR)
}
