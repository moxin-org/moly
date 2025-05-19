use anyhow::{anyhow, Result};
use makepad_widgets::*;
use moly_protocol::{
    data::{DownloadedFile, File, FileID, Model, PendingDownload},
    protocol::FileDownloadResponse,
};
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug)]
struct Inner {
    address: String,
    client: reqwest::Client,
    is_connected: bool,
}

#[derive(Clone, Debug)]
pub struct MolyClient {
    inner: Arc<Mutex<Inner>>,
}

#[allow(dead_code)]
impl MolyClient {
    pub fn new(address: String) -> Self {
        let client = reqwest::Client::builder();

        // web doesn't support these
        #[cfg(not(target_arch = "wasm32"))]
        let client = client.no_proxy();

        let client = client.build().expect("Failed to build reqwest client");

        Self {
            inner: Arc::new(Mutex::new(Inner {
                address,
                client,
                is_connected: false,
            })),
        }
    }

    pub fn address(&self) -> String {
        self.inner.lock().unwrap().address.clone()
    }

    pub fn is_connected(&self) -> bool {
        self.inner.lock().unwrap().is_connected
    }

    fn set_is_connected(&self, is_connected: bool) {
        self.inner.lock().unwrap().is_connected = is_connected;
    }

    fn client(&self) -> reqwest::Client {
        self.inner.lock().unwrap().client.clone()
    }

    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/ping", self.address());
        match self.client().get(&url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    self.set_is_connected(true);
                    Ok(())
                } else {
                    self.set_is_connected(false);
                    Cx::post_action(MolyClientAction::ServerUnreachable);
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn get_featured_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}/models/featured", self.address());

        match self.client().get(&url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    match r.json::<Vec<Model>>().await {
                        Ok(models) => Ok(models),
                        Err(e) => Err(anyhow!("Failed to parse models: {}", e)),
                    }
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn search_models(&self, query: String) -> Result<Vec<Model>> {
        let url = format!("{}/models/search?q={}", self.address(), query);

        match self.client().get(&url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    match r.json::<Vec<Model>>().await {
                        Ok(models) => Ok(models),
                        Err(e) => Err(anyhow!("Failed to parse models: {}", e)),
                    }
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn get_downloaded_files(&self) -> Result<Vec<DownloadedFile>> {
        let url = format!("{}/files", self.address());

        match self.client().get(&url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    match r.json::<Vec<DownloadedFile>>().await {
                        Ok(files) => Ok(files),
                        Err(e) => {
                            eprintln!("Error parsing files: {}", e);
                            Err(anyhow!("Failed to parse files: {}", e))
                        }
                    }
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn get_current_downloads(&self) -> Result<Vec<PendingDownload>> {
        let url = format!("{}/downloads", self.address());

        match self.client().get(&url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    match r.json::<Vec<PendingDownload>>().await {
                        Ok(files) => Ok(files),
                        Err(e) => Err(anyhow!("Failed to parse files: {}", e)),
                    }
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn download_file(&self, file: File) -> Result<()> {
        let url = format!("{}/downloads", self.address());

        let resp = self
            .client()
            .post(&url)
            .json(&serde_json::json!({
                "file_id": file.id
            }))
            .send()
            .await;

        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn track_download_progress(
        &self,
        file_id: FileID,
        mut tx: futures::channel::mpsc::UnboundedSender<
            Result<FileDownloadResponse, anyhow::Error>,
        >,
    ) {
        use futures::{stream::TryStreamExt, SinkExt};

        let mut url =
            Url::parse(&format!("{}/downloads", self.address())).expect("Invalid Moly server URL");
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty()
            .push(&file_id)
            .push("progress");

        match self.client().get(url).send().await {
            Ok(res) => {
                let mut bytes = res.bytes_stream();
                let mut buffer = String::new();
                let mut current_event = String::new();

                while let Ok(Some(chunk)) = bytes.try_next().await {
                    if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                        buffer.push_str(&text);

                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].trim().to_string();
                            buffer = buffer[pos + 1..].to_string();

                            if line.starts_with("event: ") {
                                current_event =
                                    line.trim_start_matches("event: ").trim().to_string();
                            } else if line.starts_with("data: ") {
                                let event_data = line.trim_start_matches("data: ").trim();
                                match current_event.as_str() {
                                    "complete" => {
                                        if let Err(e) = tx
                                            .send(Ok(FileDownloadResponse::Completed(
                                                moly_protocol::data::DownloadedFile::default(),
                                            )))
                                            .await
                                        {
                                            eprintln!("Failed to send completion message: {}", e);
                                        }
                                        break;
                                    }
                                    "error" => {
                                        if let Err(e) =
                                            tx.send(Err(anyhow!("Download failed"))).await
                                        {
                                            eprintln!("Failed to send error message: {}", e);
                                        }
                                        break;
                                    }
                                    "progress" => {
                                        if let Ok(value) = event_data.parse::<f32>() {
                                            let _ = tx
                                                .send(Ok(FileDownloadResponse::Progress(
                                                    file_id.clone(),
                                                    value,
                                                )))
                                                .await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if current_event == "complete" || current_event == "error" {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Err(anyhow!("Request failed: {}", e))).await;
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
            }
        }
    }

    pub async fn pause_download_file(&self, file_id: FileID) -> Result<()> {
        let mut url =
            Url::parse(&format!("{}/downloads", self.address())).expect("Invalid Moly server URL");

        // Add the ID as a path segment (auto-encodes special characters)
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty() // Remove the trailing slash, if any
            .push(&file_id);

        let resp = self
            .client()
            .post(url)
            .json(&serde_json::json!({
                "file_id": file_id
            }))
            .send()
            .await;

        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn cancel_download_file(&self, file_id: FileID) -> Result<()> {
        let mut url =
            Url::parse(&format!("{}/downloads", self.address())).expect("Invalid Moly server URL");

        // Add the ID as a path segment (auto-encodes special characters)
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty() // Remove the trailing slash, if any
            .push(&file_id);

        let resp = self
            .client()
            .delete(url)
            .json(&serde_json::json!({
                "file_id": file_id
            }))
            .send()
            .await;

        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn delete_file(&self, file_id: FileID) -> Result<()> {
        let mut url =
            Url::parse(&format!("{}/files", self.address())).expect("Invalid Moly server URL");

        // Add the ID as a path segment (auto-encodes special characters)
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty() // Remove the trailing slash, if any
            .push(&file_id);

        match self.client().delete(url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn eject_model(&self) -> Result<()> {
        let url = format!("{}/models/eject", self.address());

        match self.client().post(&url).send().await {
            Ok(r) => {
                if r.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!("Server error: {}", r.status()))
                }
            }
            Err(e) => {
                self.set_is_connected(false);
                Cx::post_action(MolyClientAction::ServerUnreachable);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]

pub enum MolyClientAction {
    None,
    ServerUnreachable,
}
