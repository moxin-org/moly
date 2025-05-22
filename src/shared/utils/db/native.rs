use super::Adapter;
use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "moxin-org";
const APP_NAME: &str = "moly";

fn project_dirs() -> &'static ProjectDirs {
    static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
        ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .expect("Failed to obtain Moly project directories")
    });

    &PROJECT_DIRS
}

fn get_key_file_path(key: &str) -> PathBuf {
    match key.strip_prefix("preferences/") {
        Some(filename) => project_dirs()
            .preference_dir()
            .join(format!("{filename}.json")),
        None => project_dirs().data_dir().join(format!("{key}.json")),
    }
}

#[derive(Clone, Default)]
pub(super) struct NativeAdapter;

impl Adapter for NativeAdapter {
    fn get(&mut self, key: &str) -> Result<Option<String>> {
        let file_path = get_key_file_path(key);
        match std::fs::read_to_string(&file_path) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(anyhow!(
                "Failed to read from file '{}': {:?}",
                file_path.display(),
                e
            )),
        }
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let file_path = get_key_file_path(key);
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        std::fs::write(&file_path, value)?;
        Ok(())
    }

    fn has(&mut self, key: &str) -> Result<bool> {
        let file_path = get_key_file_path(key);
        let exists = file_path.try_exists()?;
        Ok(exists)
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        let file_path = get_key_file_path(key);
        std::fs::remove_file(file_path)?;
        Ok(())
    }

    fn keys(&mut self) -> Result<Vec<String>> {
        let mut keys = Vec::new();

        let data_dir = project_dirs().data_dir();
        for data_entry in std::fs::read_dir(data_dir)? {
            let data_entry = data_entry?;
            let data_path = data_entry.path();

            for file_entry in std::fs::read_dir(&data_path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if file_path.ends_with(".json") {
                    // Ensure `/` is used as a separator and not `\`.
                    // Remove file extension.
                    let key = file_path
                        .strip_prefix(&data_path)
                        .unwrap()
                        .components()
                        .map(|c| c.as_os_str().to_str().unwrap())
                        .collect::<Vec<_>>()
                        .join("/")
                        .strip_suffix(".json")
                        .unwrap()
                        .to_string();

                    keys.push(key);
                }
            }
        }

        let preferences_dir = project_dirs().preference_dir();
        for file_entry in std::fs::read_dir(preferences_dir)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();

            if file_path.ends_with(".json") {
                // Remove file extension.
                let key = file_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .strip_suffix(".json")
                    .unwrap();

                keys.push(format!("preferences/{}", key));
            }
        }

        Ok(keys)
    }
}
