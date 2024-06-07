mod backend_impls;
mod store;

use moxin_protocol::protocol::Command;
use std::{path::Path, sync::{mpsc, OnceLock}};
use directories::ProjectDirs;

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
}

impl Default for Backend {
    fn default() -> Self {
        // TODO: FIXME: this has been copied from <src/data/filesystem.rs>,
        //              but should be deduplicated into a separate file shared between
        //              the frontend and backend (perhaps in moxin-protocol?)..
        pub const APP_QUALIFIER: &str = "com";
        pub const APP_ORGANIZATION: &str = "moxin-org";
        pub const APP_NAME: &str = "moxin";

        pub fn project_dirs() -> &'static ProjectDirs {
            // This can be redesigned once std::sync::LazyLock is stabilized.
            static MOXIN_PROJECT_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

            MOXIN_PROJECT_DIRS.get_or_init(|| {
                ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
                    .expect("Failed to obtain Moxin project directories")
            })
        }
        
        pub const MODEL_DOWNLOADS_DIR_NAME: &str = "model_downloads";
        // end of copied code from <src/data/filesystem.rs>

        let app_data_dir = project_dirs().data_dir();
        let models_dir = app_data_dir.join(MODEL_DOWNLOADS_DIR_NAME);
        Backend::new(app_data_dir, models_dir, 3)
    }
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
        let command_sender = backend_impls::BackendImpl::build_command_sender(
            app_data_dir,
            models_dir,
            max_download_threads,
        );
        Backend { command_sender }
    }
}
