use super::models::*;
use crate::data::filesystem;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub const SHEET_FILE_NAME: &'static str = "current_battle_sheet.json";

/// Get the built path to the current battle sheet file.
pub fn battle_sheet_path() -> PathBuf {
    let dirs = filesystem::project_dirs();
    dirs.cache_dir().join(SHEET_FILE_NAME)
}

/// Try reading the in-progress, persisted battle sheet.
pub fn restore_sheet_blocking() -> Result<Sheet> {
    let path = battle_sheet_path();
    let text = filesystem::read_from_file(path)?;
    let sheet = serde_json::from_str::<Sheet>(&text)?;
    Ok(sheet)
}

// Try saving the in-progress sheet to disk.
pub fn save_sheet_blocking(sheet: &Sheet) -> Result<()> {
    let text = serde_json::to_string(&sheet)?;
    let path = battle_sheet_path();
    filesystem::write_to_file(path, &text)?;
    Ok(())
}

/// Remove the in progress sheet from disk.
pub fn clear_sheet_blocking() -> Result<()> {
    let path = battle_sheet_path();
    std::fs::remove_file(path)?;
    Ok(())
}

/// Try to download the battle sheet corresponding to the given code from the remote.
pub fn download_sheet_blocking(code: String) -> Result<Sheet> {
    // simulate fetching from server
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Simulate failure on the first call
    static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    if FIRST_CALL.swap(false, std::sync::atomic::Ordering::SeqCst) {
        return Err(anyhow!("Failed to download battle sheet"));
    }

    let text = include_str!("sheet.json");
    let mut sheet = serde_json::from_str::<Sheet>(text)?;
    sheet.code = code;

    Ok(sheet)
}

/// Try to send the completed sheet to the server.
pub fn send_sheet_blocking(_sheet: Sheet) -> Result<()> {
    // simulate sending to server
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Simulate failure on the first call
    // static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    // if FIRST_CALL.swap(false, std::sync::atomic::Ordering::SeqCst) {
    //     return Err(anyhow!("Failed to send battle sheet"));
    // }

    Ok(())
}
