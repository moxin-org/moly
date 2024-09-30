use super::models::*;
use crate::data::filesystem;
use anyhow::{anyhow, Error, Result};
use makepad_widgets::{Actions, Cx};
use std::path::PathBuf;

pub const SHEET_FILE_NAME: &'static str = "current_battle_sheet.json";

/// Get the built path to the current battle sheet file.
pub fn battle_sheet_path() -> PathBuf {
    let dirs = filesystem::project_dirs();
    dirs.cache_dir().join(SHEET_FILE_NAME)
}

/// Try reading the in-progress, persisted battle sheet.
fn restore_sheet_blocking() -> Result<Sheet> {
    let path = battle_sheet_path();
    let text = filesystem::read_from_file(path)?;
    let sheet = serde_json::from_str::<Sheet>(&text)?;
    Ok(sheet)
}

fn save_sheet_blocking(sheet: &Sheet) -> Result<()> {
    let text = serde_json::to_string(&sheet)?;
    let path = battle_sheet_path();
    filesystem::write_to_file(path, &text)?;
    Ok(())
}

fn clear_sheet_blocking() -> Result<()> {
    let path = battle_sheet_path();
    std::fs::remove_file(path)?;
    Ok(())
}

/// Try to download the battle sheet corresponding to the given code from the remote.
fn download_sheet_blocking(code: String) -> Result<Sheet> {
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

/// Isolated interface to connect and work with the remote battle server.
pub struct Service {
    /// Identify this instance to handle responses in isolation.
    /// `Cx::post_action` by itself is global.
    id: usize,
}

impl Service {
    /// Create a new identified instance.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Self { id }
    }

    pub fn restore_sheet(&self) {
        let id = self.id;
        std::thread::spawn(move || match restore_sheet_blocking() {
            Ok(sheet) => Cx::post_action((id, Response::SheetLoaded(sheet))),
            Err(err) => Cx::post_action((id, Response::RestoreSheetError(err))),
        });
    }

    pub fn download_sheet(&self, code: String) {
        let id = self.id;
        std::thread::spawn(move || match download_sheet_blocking(code) {
            Ok(sheet) => Cx::post_action((id, Response::SheetLoaded(sheet))),
            Err(err) => Cx::post_action((id, Response::DownloadSheetError(err))),
        });
    }

    pub fn send_battle_sheet(&self, _sheet: Sheet) {
        let id = self.id;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            Cx::post_action((id, Response::SheetSent));
        });
    }

    pub fn clear_sheet(&self) {
        let id = self.id;
        std::thread::spawn(move || match clear_sheet_blocking() {
            Ok(()) => Cx::post_action((id, Response::SheetCleared)),
            Err(err) => Cx::post_action((id, Response::ClearSheetError(err))),
        });
    }

    pub fn sheet_cleared(&self, actions: &Actions) -> bool {
        self.responses(actions)
            .any(|response| matches!(response, Response::SheetCleared))
    }

    pub fn battle_sheet_downloaded<'a>(&'a self, actions: &'a Actions) -> Option<&'a Sheet> {
        self.responses(actions)
            .filter_map(|response| match response {
                Response::SheetLoaded(sheet) => Some(sheet),
                _ => None,
            })
            .next()
    }

    pub fn battle_sheet_sent(&self, actions: &Actions) -> bool {
        self.responses(actions)
            .any(|response| matches!(response, Response::SheetSent))
    }

    pub fn download_sheet_failed<'a>(&'a self, actions: &'a Actions) -> Option<&'a Error> {
        self.responses(actions)
            .filter_map(|response| match response {
                Response::DownloadSheetError(err) => Some(err),
                _ => None,
            })
            .next()
    }

    pub fn restore_sheet_failed<'a>(&'a self, actions: &'a Actions) -> Option<&'a Error> {
        self.responses(actions)
            .filter_map(|response| match response {
                Response::RestoreSheetError(err) => Some(err),
                _ => None,
            })
            .next()
    }

    pub fn save_sheet(&self, sheet: Sheet) {
        let id = self.id;
        std::thread::spawn(move || match save_sheet_blocking(&sheet) {
            Ok(()) => Cx::post_action((id, Response::SheetSaved)),
            Err(err) => Cx::post_action((id, Response::SaveSheetError(err))),
        });
    }

    pub fn save_sheet_failed<'a>(&'a self, actions: &'a Actions) -> Option<&'a Error> {
        self.responses(actions)
            .filter_map(|response| match response {
                Response::SaveSheetError(err) => Some(err),
                _ => None,
            })
            .next()
    }

    pub fn sheet_saved(&self, actions: &Actions) -> bool {
        self.responses(actions)
            .any(|response| matches!(response, Response::SheetSaved))
    }

    /// Handle responses sent from this specific instance.
    fn responses<'a>(&'a self, actions: &'a Actions) -> impl Iterator<Item = &'a Response> {
        actions
            .iter()
            .filter_map(move |action| action.downcast_ref::<(usize, Response)>())
            .filter(|(id, _)| *id == self.id)
            .map(|(_, response)| response)
    }
}

/// Actions sent from other threads thru `Cx::post_action` representing async responses.
///
/// Doesn't actually need private, nor the `responses` function, but just exposing
/// event handling thru methods like button's `clicked` is less error prone and more
/// elegant, so would be ideal to not use this from outside.
#[derive(Debug)]
enum Response {
    SheetLoaded(Sheet),
    SheetSent,
    SheetSaved,
    SheetCleared,
    DownloadSheetError(Error),
    RestoreSheetError(Error),
    SaveSheetError(Error),
    ClearSheetError(Error),
}
