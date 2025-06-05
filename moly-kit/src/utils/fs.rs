/// Represents a file picked by the user through a file picker dialog.
/// Note: This is mainly to hide the underlying dependency that may be switched out in the future.
#[derive(Debug, Clone)]
pub(crate) struct PickedFile(rfd::FileHandle);

impl PickedFile {
    pub(crate) fn file_name(&self) -> String {
        self.0.file_name()
    }

    /// Read the contents of the picked file all at once.
    /// Warning: rfd does not return a `Result`. Also, on native it is just spawning
    /// a thread, so it will run without possible cancelation.
    pub(crate) async fn read(&self) -> std::io::Result<Vec<u8>> {
        Ok(self.0.read().await)
    }
}

/// Opens a file picker to select multiple files. Fails if not possible.
pub(crate) async fn pick_files() -> Result<Vec<PickedFile>, ()> {
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "native-fs", feature = "web-fs"))] {
            let files = rfd::AsyncFileDialog::new()
                .pick_files()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(PickedFile)
                .collect();

            Ok(files)
        } else {
            Err(())
        }
    }
}
