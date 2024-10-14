use anyhow::Result;

use super::{
    clients::{fake::FakeClient, remote::RemoteClient},
    Sheet,
};

pub trait Client {
    /// Try reading the in-progress, persisted battle sheet.
    fn restore_sheet_blocking(&mut self) -> Result<Sheet>;

    // Try saving the in-progress sheet to disk.
    fn save_sheet_blocking(&mut self, sheet: &Sheet) -> Result<()>;

    /// Remove the in progress sheet from disk.
    fn clear_sheet_blocking(&mut self) -> Result<()>;

    /// Try to download the battle sheet corresponding to the given code from the remote.
    fn download_sheet_blocking(&mut self, code: String) -> Result<Sheet>;

    /// Try to send the completed sheet to the server.
    fn send_sheet_blocking(&mut self, _sheet: Sheet) -> Result<()>;
}

/// Pick a client for battle, based on the environment variable.
/// If an API URL is specified, the remote client will be used.
// TODO: Can be probably done without boxing, by using a FF.
pub fn client() -> Box<dyn Client + Send + 'static> {
    match option_env!("MOLY_BATTLE_API") {
        Some(base_url) => Box::new(RemoteClient::new(base_url.to_string())),
        None => Box::new(FakeClient),
    }
}
