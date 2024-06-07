use std::{path::PathBuf, sync::OnceLock};
use directories::ProjectDirs;

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

pub fn setup_model_downloads_folder() -> PathBuf {
    let downloads_dir = project_dirs()
        .data_dir()
        .join(MODEL_DOWNLOADS_DIR_NAME);

    std::fs::create_dir_all(&downloads_dir).unwrap_or_else(|_|
        panic!("Failed to create the model downloads directory at {:?}", downloads_dir)
    );
    downloads_dir
}
