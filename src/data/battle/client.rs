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
