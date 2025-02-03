use directories::ProjectDirs;
use std::{
    path::PathBuf,
    sync::OnceLock,
};

// Heads-up, this is temporarily (to make collaboration easier) replicating the filesystem code in the Moly app.
// We need to make some decisions around filesystem management as we move forward. We'll revisit responsabilities,
// and also we'll very soon need abstractions around storage access for mobile and web compatibility.

pub const APP_QUALIFIER: &str = "com";
pub const APP_ORGANIZATION: &str = "moxin-org";
pub const APP_NAME: &str = "moly";

pub fn project_dirs() -> &'static ProjectDirs {
    // This can be redesigned once std::sync::LazyLock is stabilized.
    static MOLY_PROJECT_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

    MOLY_PROJECT_DIRS.get_or_init(|| {
        ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .expect("Failed to obtain Moly project directories")
    })
}

pub const MODEL_DOWNLOADS_DIR_NAME: &str = "model_downloads";
pub fn setup_model_downloads_folder() -> PathBuf {
    let downloads_dir = project_dirs().data_dir().join(MODEL_DOWNLOADS_DIR_NAME);

    std::fs::create_dir_all(&downloads_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create the model downloads directory at {:?}",
            downloads_dir
        )
    });
    downloads_dir
}

pub const CHATS_DIR_NAME: &str = "chats";
pub fn setup_chats_folder() -> PathBuf {
    let chats_dir = project_dirs().data_dir().join(CHATS_DIR_NAME);

    std::fs::create_dir_all(&chats_dir)
        .unwrap_or_else(|_| panic!("Failed to create the chats directory at {:?}", chats_dir));
    chats_dir
}
