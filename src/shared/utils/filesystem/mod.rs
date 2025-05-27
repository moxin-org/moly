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
    pub fn new(adapter: A) -> Self {
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
    // async read, async exists, async list, but async queue_write to ensure no overlappings
    pub async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let mut adapter = self.adapter.lock().await;
        adapter.read(path).await
    }

    pub async fn exists(&self, path: &Path) -> Result<bool> {
        let mut adapter = self.adapter.lock().await;
        adapter.exists(path).await
    }

    pub async fn remove(&self, path: &Path) -> Result<()> {
        let mut adapter = self.adapter.lock().await;
        adapter.remove(path).await
    }

    pub async fn list(&self, path: &Path) -> Result<Vec<String>> {
        let mut adapter = self.adapter.lock().await;
        adapter.list(path).await
    }

    pub async fn queue_write(&mut self, path: PathBuf, content: Vec<u8>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.write_queue
            .send((path, content, tx))
            .await
            .map_err(|e| anyhow!("Failed to send write request: {:?}", e))?;

        rx.await.unwrap()
    }

    pub async fn read_string(&self, path: &Path) -> Result<String> {
        let content = self.read(path).await?;
        let content = String::from_utf8(content)?;
        Ok(content)
    }

    pub async fn read_json<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        let content = self.read_string(path).await?;
        let value = serde_json::from_str(&content)?;
        Ok(value)
    }

    pub async fn queue_write_string(&mut self, path: PathBuf, content: String) -> Result<()> {
        let content = content.into_bytes();
        self.queue_write(path, content).await
    }

    pub async fn queue_write_json<T: Serialize>(&mut self, path: PathBuf, value: &T) -> Result<()> {
        let content = serde_json::to_string(value)?;
        self.queue_write_string(path, content).await
    }
}

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
