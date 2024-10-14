use crate::data::battle::{client::Client, models::*};
use anyhow::Result;

use super::fs;

/// Client that always returns `sheet.json` data.
/// `send_sheet_blocking` does nothing.
pub struct FakeClient;

impl Client for FakeClient {
    fn clear_sheet_blocking(&mut self) -> Result<()> {
        // Simulate failure on the first call
        // static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        // if FIRST_CALL.swap(false, std::sync::atomic::Ordering::SeqCst) {
        //     return Err(anyhow!("Filesystem error (314): Very very very very very very very very very very very very very very very very very very very very very long error"));
        // }

        fs::clear_sheet_blocking()
    }

    fn save_sheet_blocking(&mut self, sheet: &Sheet) -> Result<()> {
        // Simulate failure on the first call
        // static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        // if FIRST_CALL.swap(false, std::sync::atomic::Ordering::SeqCst) {
        //     return Err(anyhow!("Filesystem error (42): Permission denied"));
        // }

        fs::save_sheet_blocking(sheet)
    }

    fn restore_sheet_blocking(&mut self) -> Result<Sheet> {
        fs::restore_sheet_blocking()
    }

    fn download_sheet_blocking(&mut self, code: String) -> Result<Sheet> {
        // simulate fetching from server
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Simulate failure on the first call
        // static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        // if FIRST_CALL.swap(false, std::sync::atomic::Ordering::SeqCst) {
        //     return Err(anyhow!("500 Internal Server Error"));
        // }

        let text = include_str!("sheet.json");
        let mut sheet = serde_json::from_str::<Sheet>(text)?;
        sheet.code = code;

        Ok(sheet)
    }

    fn send_sheet_blocking(&mut self, _sheet: Sheet) -> Result<()> {
        // simulate sending to server
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Simulate failure on the first call
        // static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        // if FIRST_CALL.swap(false, std::sync::atomic::Ordering::SeqCst) {
        //     return Err(anyhow!("No connecton"));
        // }

        Ok(())
    }
}
