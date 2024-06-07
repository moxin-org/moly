mod backend_impls;
mod store;

use moxin_protocol::protocol::Command;
use std::sync::mpsc;

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
}

impl Default for Backend {
    fn default() -> Self {
        // TODO: FIXME: use directories::ProjectDirs::data_dir() instead.
        // <https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.data_dir>
        let home_dir = std::env::var("HOME") // Unix-like systems
            .or_else(|_| std::env::var("USERPROFILE")) // Windows
            .unwrap_or_else(|_| ".".to_string());
        Backend::new(home_dir.clone(), home_dir, 3)
    }
}

impl Backend {
    /// # Argument
    ///
    /// * `home_dir` - The home directory of the application.
    /// * `models_dir` - The download path of the model.
    /// * `max_download_threads` - Maximum limit on simultaneous file downloads.
    pub fn new(home_dir: String, models_dir: String, max_download_threads: usize) -> Backend {
        let command_sender = backend_impls::BackendImpl::build_command_sender(
            home_dir,
            models_dir,
            max_download_threads,
        );
        Backend { command_sender }
    }
}
