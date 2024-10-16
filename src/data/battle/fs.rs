//! Common filesystem logic that must be implemented by all clients.

use anyhow::Result;
use std::path::PathBuf;

use crate::data::{battle::Sheet, filesystem};

pub const SHEET_FILE_NAME: &'static str = "current_battle_sheet.json";

/// Get the built path to the current (in-progress) battle sheet file.
fn battle_sheet_path() -> PathBuf {
    let dirs = filesystem::project_dirs();
    dirs.cache_dir().join(SHEET_FILE_NAME)
}

/// Remove the in progress sheet from disk.
pub(super) fn clear_sheet_blocking() -> Result<()> {
    let path = battle_sheet_path();
    std::fs::remove_file(path)?;
    Ok(())
}

/// Read the in-progress, persisted battle sheet, if any.
pub(super) fn restore_sheet_blocking() -> Result<Sheet> {
    let path = battle_sheet_path();
    let text = filesystem::read_from_file(path)?;
    let sheet = serde_json::from_str::<Sheet>(&text)?;
    Ok(sheet)
}

/// Persist the in-progress sheet to disk.
pub(super) fn save_sheet_blocking(sheet: &Sheet) -> Result<()> {
    let text = serde_json::to_string(&sheet)?;
    let path = battle_sheet_path();
    filesystem::write_to_file(path, &text)?;
    Ok(())
}
