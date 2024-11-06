mod backend_impls;
mod store;

use moly_protocol::protocol::Command;
use std::{path::Path, sync::mpsc};

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
}

impl Backend {
    /// # Arguments
    /// * `app_data_dir` - The directory where application data should be stored.
    /// * `models_dir` - The directory where models should be downloaded.
    /// * `max_download_threads` - Maximum limit on simultaneous file downloads.
    pub fn new<A: AsRef<Path>, M: AsRef<Path>>(
        app_data_dir: A,
        models_dir: M,
        max_download_threads: usize,
    ) -> Backend {
        #[cfg(debug_assertions)]
        env_logger::try_init().unwrap_or_else(|_| {
            eprintln!("Failed to initialize the logger. Maybe another library has already initialized a global logger.");
        });
        let command_sender = backend_impls::LlamaEdgeApiServerBackend::build_command_sender(
            app_data_dir,
            models_dir,
            max_download_threads,
        );
        Backend { command_sender }
    }
}

/// Options for the Mega backend.
pub struct MegaOptions {
    pub bootstrap_nodes: &'static str,
    pub ztm_agent_port: u16,
    pub http_port: u16,
}

pub const MEGA_OPTIONS: MegaOptions = MegaOptions {
    bootstrap_nodes: "http://gitmono.org/relay",
    ztm_agent_port: 7777,
    http_port: 8000,
};
