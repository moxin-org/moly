mod backend_impls;
mod store;

use moxin_protocol::protocol::Command;
use std::sync::mpsc;

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new(".".to_string())
    }
}

impl Backend {
    /// # Argument
    ///
    /// * `models_dir` - The download path of the model.
    pub fn new(models_dir: String) -> Backend {
        let command_sender = backend_impls::BackendImpl::build_command_sender(models_dir);
        Backend { command_sender }
    }
}
