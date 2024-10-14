use crate::data::{
    battle::{client::Client, models::*},
    filesystem,
};
use anyhow::{anyhow, Context, Result};
use reqwest::Method;
use std::path::PathBuf;

pub const SHEET_FILE_NAME: &'static str = "current_battle_sheet.json";

pub struct RemoteClient {
    base_url: String,
}

impl Client for RemoteClient {
    fn clear_sheet_blocking(&mut self) -> Result<()> {
        let path = battle_sheet_path();
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn download_sheet_blocking(&mut self, code: String) -> Result<Sheet> {
        let req = self.request(Method::GET, &format!("sheets/{}", code));
        let res = req
            .send()
            .with_context(|| format!("Failed to fetch sheet {}", code))?;

        if res.status().is_success() {
            let mut sheet = res
                .json::<Sheet>()
                .with_context(|| "Can not parse the sheet provided by the server")?;
            // Just in case...
            sheet.code = code;
            Ok(sheet)
        } else {
            let message = res
                .text()
                .with_context(|| "Can not read the error message from the server")?;
            Err(anyhow!("Failed to fetch sheet: {}", message))
        }
    }

    fn restore_sheet_blocking(&mut self) -> Result<Sheet> {
        let path = battle_sheet_path();
        let text = filesystem::read_from_file(path)?;
        let sheet = serde_json::from_str::<Sheet>(&text)?;
        Ok(sheet)
    }

    fn save_sheet_blocking(&mut self, sheet: &Sheet) -> Result<()> {
        let text = serde_json::to_string(&sheet)?;
        let path = battle_sheet_path();
        filesystem::write_to_file(path, &text)?;
        Ok(())
    }

    fn send_sheet_blocking(&mut self, sheet: Sheet) -> Result<()> {
        let res = self
            .request(Method::PUT, &format!("sheets/{}", sheet.code))
            .json(&sheet)
            .send()
            .with_context(|| "Failed to communicate the sheet back to the server")?;

        if !res.status().is_success() {
            let message = res
                .text()
                .with_context(|| "Can not read the error message from the server")?;
            return Err(anyhow!("Failed to send sheet: {}", message));
        }

        Ok(())
    }
}

impl RemoteClient {
    /// Build a new remote client with the given base URL.
    /// Base url example: `http://localhost:9800`.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    fn request(&self, method: Method, path: &str) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path);
        reqwest::blocking::Client::new().request(method, url)
    }
}

/// Get the built path to the current (in-progress) battle sheet file.
fn battle_sheet_path() -> PathBuf {
    let dirs = filesystem::project_dirs();
    dirs.cache_dir().join(SHEET_FILE_NAME)
}
