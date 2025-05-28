use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use super::super::adapter::Adapter;
use anyhow::Result;
use directories::ProjectDirs;
use futures::StreamExt;

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

fn validate_and_resolve(path: &Path) -> PathBuf {
    match path.strip_prefix("preferences/") {
        Ok(rest) => project_dirs().preference_dir().join(rest),
        Err(_) => project_dirs().data_dir().join(path),
    }
}

/// An [Adapter] for `FileSystem` that interacts with a traditional native filesystem.
// Note: This uses `async_fs` so its implementation is easy to port to web since
// the `web_fs` crate has a similar API.
#[derive(Default)]
pub struct NativeAdapter;

impl Adapter for NativeAdapter {
    async fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let path = validate_and_resolve(path);
        let content = async_fs::read(path).await?;
        Ok(content)
    }

    async fn exists(&mut self, path: &Path) -> Result<bool> {
        let path = validate_and_resolve(path);
        let exists = async_fs::metadata(path).await.map(|_| true).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e)
            }
        })?;
        Ok(exists)
    }

    async fn remove(&mut self, path: &Path) -> Result<()> {
        let path = validate_and_resolve(path);
        async_fs::remove_file(path).await?;
        Ok(())
    }

    async fn list(&mut self, path: &Path) -> Result<Vec<String>> {
        let path = validate_and_resolve(path);
        let mut entries = async_fs::read_dir(path).await?;
        let mut result = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                result.push(name.to_string());
            }
        }
        Ok(result)
    }

    async fn write(&mut self, path: &Path, content: &[u8]) -> Result<()> {
        let path = validate_and_resolve(path);
        async_fs::create_dir_all(path.parent().unwrap()).await?;
        async_fs::write(path, content).await?;
        Ok(())
    }
}
