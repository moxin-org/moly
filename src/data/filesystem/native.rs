use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::OnceLock,
};

pub const APP_QUALIFIER: &str = "com";
pub const APP_ORGANIZATION: &str = "moxin-org";
pub const APP_NAME: &str = "moly";

fn project_dirs() -> &'static ProjectDirs {
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

pub fn setup_preferences_folder() -> PathBuf {
    let preferences_dir = project_dirs().preference_dir();

    std::fs::create_dir_all(&preferences_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create the preferences directory at {:?}",
            preferences_dir
        )
    });
    preferences_dir.to_path_buf()
}

pub fn read_from_file(path: PathBuf) -> Result<String> {
    let mut file = File::open(path)?;

    let mut json = String::new();
    file.read_to_string(&mut json)?;
    Ok(json)
}

pub fn write_to_file(path: PathBuf, json: &str) -> Result<()> {
    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create or overwrite the file
    let mut file = File::create(path)?;

    // Write the JSON data to the file
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn remove_file(path: PathBuf) -> Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn read_dir(path: PathBuf) -> Box<dyn Iterator<Item = Result<PathBuf>>> {
    match fs::read_dir(path) {
        Ok(entries) => Box::new(entries.map(|entry| match entry {
            Ok(entry) => Ok(entry.path()),
            Err(err) => Err(anyhow!(err)),
        })),
        Err(err) => Box::new(std::iter::once(Err(anyhow!(err)))),
    }
}
