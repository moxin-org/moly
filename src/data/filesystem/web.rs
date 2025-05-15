use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{LazyLock, RwLock},
};

static FS: LazyLock<RwLock<HashMap<PathBuf, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn read_from_file(path: PathBuf) -> Result<String> {
    if let Some(content) = FS.read().unwrap().get(&path) {
        Ok(content.clone())
    } else {
        Err(anyhow!("File not found"))
    }
}

pub fn write_to_file(path: PathBuf, json: &str) -> Result<()> {
    FS.write().unwrap().insert(path.clone(), json.to_string());
    Ok(())
}

pub fn setup_model_downloads_folder() -> PathBuf {
    PathBuf::from("model_downloads")
}

pub fn setup_chats_folder() -> PathBuf {
    PathBuf::from("chats")
}

pub fn setup_preferences_folder() -> PathBuf {
    PathBuf::from("preferences")
}

pub fn remove_file(path: PathBuf) -> Result<()> {
    if FS.write().unwrap().remove(&path).is_some() {
        Ok(())
    } else {
        Err(anyhow!("File not found"))
    }
}

pub fn read_dir(path: PathBuf) -> Box<dyn Iterator<Item = Result<PathBuf>>> {
    let files = FS.read().unwrap();
    let paths: Vec<_> = files
        .keys()
        .filter(|p| p.starts_with(&path))
        .cloned()
        .collect();

    Box::new(paths.into_iter().map(Ok))
}
