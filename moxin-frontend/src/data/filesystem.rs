use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use anyhow::{Context, Result};

// Note that .moxin will create a hidden folder in unix-like systems.
// However in Windows the folder will be visible by default.
pub const DEFAULT_DOWNLOADS_DIR: &str = ".moxin/model_downloads";
pub const MOXIN_HOME_DIR: &str = ".moxin";

pub fn setup_model_downloads_folder() -> String {
    let home_dir = get_home_dir();
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

pub fn get_home_dir() -> String {
    env::var("HOME"). // Unix-like systems
        or_else(|_| env::var("USERPROFILE")) // Windows
        .unwrap_or_else(|_| ".".to_string())
}

pub fn moxin_home_dir() -> PathBuf {
    let home_dir = get_home_dir();
    PathBuf::from(home_dir).join(MOXIN_HOME_DIR)
}

pub fn open_folder(path: &str) -> Result<()> {
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else {
        // Assuming xdg-open is available for Linux and Unix-like systems
        Command::new("xdg-open").arg(path).spawn()
    };

    result.context(format!("Failed to open folder: {}", path))?;

    Ok(())
}
