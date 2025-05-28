mod adapter;
mod adapters;

use adapter::Adapter;
use anyhow::{anyhow, Result};
use futures::{
    channel::{mpsc, oneshot},
    SinkExt, StreamExt,
};
use moly_kit::utils::asynchronous::spawn;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

/// Filesystem implementation over abstract adapters.
///
/// It's abstract but async natiure allows it to be used on restrictive environments
/// like the web.
///
/// Writes (inside this instance) are queued and executed in order in favor of a
/// more deterministic behavior.
#[derive(Debug)]
pub struct FileSystem<A: Adapter> {
    adapter: Arc<futures::lock::Mutex<A>>,
    write_queue: mpsc::Sender<(PathBuf, Vec<u8>, oneshot::Sender<Result<()>>)>,
}

impl<A: Adapter> Clone for FileSystem<A> {
    fn clone(&self) -> Self {
        Self {
            adapter: Arc::clone(&self.adapter),
            write_queue: self.write_queue.clone(),
        }
    }
}

impl<A: Adapter> FileSystem<A> {
    fn new(adapter: A) -> Self {
        let (tx, mut rx) = mpsc::channel::<(PathBuf, Vec<u8>, oneshot::Sender<Result<()>>)>(0);
        let adapter = Arc::new(futures::lock::Mutex::new(adapter));

        let adapter_clone = Arc::clone(&adapter);
        spawn(async move {
            while let Some((path, content, response)) = rx.next().await {
                let adapter_clone = Arc::clone(&adapter_clone);
                let path_clone = path.clone();
                let write_future = async move {
                    let mut adapter = adapter_clone.lock().await;
                    adapter.write(&path_clone, &content).await
                };

                match write_future.await {
                    Ok(()) => response.send(Ok(())).unwrap(),
                    Err(e) => {
                        response
                            .send(Err(anyhow!(
                                "Failed to write to '{}': {:?}",
                                path.display(),
                                e
                            )))
                            .unwrap();
                    }
                }
            }
        });

        Self {
            adapter,
            write_queue: tx,
        }
    }
}

impl<A: Adapter> FileSystem<A> {
    /// Read the content of a file as a byte vector.
    pub async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let mut adapter = self.adapter.lock().await;
        adapter.read(path).await
    }

    /// Read the content of a file as a utf8 string.
    ///
    /// Note: Fails if the content is not valid UTF-8.
    pub async fn read_string(&self, path: &Path) -> Result<String> {
        let content = self.read(path).await?;
        let content = String::from_utf8(content)?;
        Ok(content)
    }

    /// Read and deserialize JSON content from a file to a target type.
    ///
    /// Note: Expects utf8 encoded JSON content.
    pub async fn read_json<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        let content = self.read_string(path).await?;
        let value = serde_json::from_str(&content)?;
        Ok(value)
    }

    /// Check existence of a file. Errors if it cannot be determined.
    // TODO: Consider using a `metadata` method instead.
    #[allow(dead_code)]
    pub async fn exists(&self, path: &Path) -> Result<bool> {
        let mut adapter = self.adapter.lock().await;
        adapter.exists(path).await
    }

    /// Remove a file from the filesystem.
    pub async fn remove(&self, path: &Path) -> Result<()> {
        let mut adapter = self.adapter.lock().await;
        adapter.remove(path).await
    }

    /// Get a list of the entry names in the given directory.
    pub async fn list(&self, path: &Path) -> Result<Vec<String>> {
        let mut adapter = self.adapter.lock().await;
        adapter.list(path).await
    }

    /// Write some bytes content to a given path, creating any necessary directories.
    // TODO: Is adapter responsability to create directories, but it shouldn't.
    pub async fn queue_write(&mut self, path: PathBuf, content: Vec<u8>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.write_queue
            .send((path, content, tx))
            .await
            .map_err(|e| anyhow!("Failed to send write request: {:?}", e))?;

        rx.await.unwrap()
    }

    /// Write a string content to a given path, creating any necessary directories.
    pub async fn queue_write_string(&mut self, path: PathBuf, content: String) -> Result<()> {
        let content = content.into_bytes();
        self.queue_write(path, content).await
    }

    /// Write a JSON serialized value to a given path, creating any necessary directories.
    pub async fn queue_write_json<T: Serialize>(&mut self, path: PathBuf, value: &T) -> Result<()> {
        let content = serde_json::to_string(value)?;
        self.queue_write_string(path, content).await
    }
}

/// Access the global singleton instance of the filesystem used across Moly.
///
/// Configured propertly for the target platform (native or web).
pub fn global() -> FileSystem<impl Adapter> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use adapters::web::WebAdapter;
            static FS: LazyLock<FileSystem<WebAdapter>> = LazyLock::new(|| FileSystem::new(WebAdapter::default()));
        } else {
            use adapters::native::NativeAdapter;
            static FS: LazyLock<FileSystem<NativeAdapter>> = LazyLock::new(|| FileSystem::new(NativeAdapter::default()));
        }
    }

    FS.clone()
}
