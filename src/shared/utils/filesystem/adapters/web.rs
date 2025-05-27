use std::path::Path;

use super::super::adapter::Adapter;
use anyhow::Result;
use futures::StreamExt;

#[derive(Default)]
pub struct WebAdapter;

impl Adapter for WebAdapter {
    async fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let content = web_fs::read(path).await?;
        Ok(content)
    }

    async fn exists(&mut self, path: &Path) -> Result<bool> {
        let file = web_fs::File::open(path).await?;
        let exists = file.metadata().await.map(|_| true).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e)
            }
        })?;
        Ok(exists)
    }

    async fn remove(&mut self, path: &Path) -> Result<()> {
        web_fs::remove_file(path).await?;
        Ok(())
    }

    async fn list(&mut self, path: &Path) -> Result<Vec<String>> {
        let mut entries = web_fs::read_dir(path).await?;
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
        web_fs::create_dir_all(path.parent().unwrap()).await?;
        web_fs::write(path, content).await?;
        Ok(())
    }
}
