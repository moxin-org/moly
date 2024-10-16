use anyhow::Result;

use super::{fake::FakeClient, remote::RemoteClient};
use crate::data::battle::{client::Client, models::Sheet};

/// Automatically chooses between the fake and remote clients.
/// The fake client is used when the env var `MOLY_ARENA_FAKE` is set.
pub struct AutoClient {
    inner: Box<dyn Client + Send + 'static>,
}

impl AutoClient {
    /// Creates a new auto client that inside may contain a fake or remote client.
    /// `base_url` is only used if the remote client is chosen.
    pub fn new(base_url: String) -> Self {
        let inner: Box<dyn Client + Send + 'static> = match option_env!("MOLY_ARENA_FAKE") {
            Some(_) => Box::new(FakeClient::new()),
            None => Box::new(RemoteClient::new(base_url)),
        };

        Self { inner }
    }
}

impl Client for AutoClient {
    fn clear_sheet_blocking(&mut self) -> Result<()> {
        self.inner.clear_sheet_blocking()
    }

    fn save_sheet_blocking(&mut self, sheet: &Sheet) -> Result<()> {
        self.inner.save_sheet_blocking(sheet)
    }

    fn restore_sheet_blocking(&mut self) -> Result<Sheet> {
        self.inner.restore_sheet_blocking()
    }

    fn download_sheet_blocking(&mut self, code: String) -> Result<Sheet> {
        self.inner.download_sheet_blocking(code)
    }

    fn send_sheet_blocking(&mut self, sheet: Sheet) -> Result<()> {
        self.inner.send_sheet_blocking(sheet)
    }
}
