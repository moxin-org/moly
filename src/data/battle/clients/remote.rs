use crate::data::battle::{client::Client, models::*};
use anyhow::{anyhow, Context, Result};
use reqwest::Method;

use super::fs;

/// Client that interacts with a remote HTTP server.
/// Can be used with the real server, or with a fake one.
pub struct RemoteClient {
    base_url: String,
}

impl Client for RemoteClient {
    fn clear_sheet_blocking(&mut self) -> Result<()> {
        fs::clear_sheet_blocking()
    }

    fn restore_sheet_blocking(&mut self) -> Result<Sheet> {
        fs::restore_sheet_blocking()
    }

    fn save_sheet_blocking(&mut self, sheet: &Sheet) -> Result<()> {
        fs::save_sheet_blocking(sheet)
    }

    fn download_sheet_blocking(&mut self, code: String) -> Result<Sheet> {
        let code = code.trim();

        if code.is_empty() {
            return Err(anyhow!("Sheet code can not be empty"));
        }

        let req = self.request(Method::GET, &format!("sheets/{}", code));
        let res = req
            .send()
            .with_context(|| format!("Failed to fetch sheet {}", code))?;

        if res.status().is_success() {
            let mut sheet = res
                .json::<Sheet>()
                .with_context(|| "Can not parse the sheet provided by the server")?;
            // Just in case...
            sheet.code = code.to_string();
            Ok(sheet)
        } else {
            let message = res
                .text()
                .with_context(|| "Can not read the error message from the server")?;
            Err(anyhow!("Failed to fetch sheet: {}", message))
        }
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
