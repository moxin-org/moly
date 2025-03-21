use moly_protocol::{
    data::{DownloadedFile, File, FileID, Model, PendingDownload},
    open_ai::{ChatRequestData, ChatResponse, ChatResponseChunkData, ChatResponseData, ChunkChoiceData, MessageData, Role, StopReason},
    protocol::{FileDownloadResponse, LoadModelOptions, LoadModelResponse},
};
use url::Url;
use std::sync::mpsc::Sender;
use std::io::BufRead;

#[derive(Clone, Debug)]
pub struct MolyClient {
    address: String,
    client: reqwest::Client,
}

#[allow(dead_code)]
impl MolyClient {
    pub fn new(address: String) -> Self {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("Failed to build reqwest client");

        Self {
            address,
            client
        }
    }

    pub fn get_featured_models(&self, tx: Sender<Result<Vec<Model>, anyhow::Error>>) {
        let url = format!("{}/models/featured", self.address);
        let client = self.client.clone();

        tokio::spawn(async move {
            let resp = client.get(&url).send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<Model>>().await {
                            Ok(models) => {
                                let _ = tx.send(Ok(models));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Failed to parse models: {}", e)));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn search_models(&self, query: String, tx: Sender<Result<Vec<Model>, anyhow::Error>>) {
        let url = format!("{}/models/search?q={}", self.address, query);

        let client = self.client.clone();
        tokio::spawn(async move {
            let resp = client.get(&url).send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<Model>>().await {
                            Ok(models) => {
                                let _ = tx.send(Ok(models));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Failed to parse models: {}", e)));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn get_downloaded_files(&self, tx: Sender<Result<Vec<DownloadedFile>, anyhow::Error>>) {
        let url = format!("{}/files", self.address);
        let client = self.client.clone();
        tokio::spawn(async move {
            let resp = client.get(&url).send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<DownloadedFile>>().await {
                            Ok(files) => {
                                let _ = tx.send(Ok(files));
                            }
                            Err(e) => {
                                println!("Error parsing files: {}", e);
                                let _ = tx.send(Err(anyhow::anyhow!("Failed to parse files: {}", e)));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn get_current_downloads(&self, tx: Sender<Result<Vec<PendingDownload>, anyhow::Error>>) {
        let url = format!("{}/downloads", self.address);
        let client = self.client.clone();

        tokio::spawn(async move {
            let resp = client.get(&url).send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<PendingDownload>>().await {
                            Ok(files) => {
                                let _ = tx.send(Ok(files));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Failed to parse files: {}", e)));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn download_file(&self, file: File, tx: Sender<Result<(), anyhow::Error>>) {
        let url = format!("{}/downloads", self.address);
        let client = self.client.clone();

        tokio::spawn(async move {
            let resp = client.post(&url)
                .json(&serde_json::json!({
                    "file_id": file.id
                }))
                .send().await;

            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        let _ = tx.send(Ok(()));
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    // TODO(MolyKit): Broke since switching from blocking and std::thread::spawn.
    pub fn track_download_progress(&self, file_id: FileID, tx: Sender<Result<FileDownloadResponse, anyhow::Error>>) {
        // let mut url = Url::parse(&format!("{}/downloads", self.address)).expect("Invalid Moly server URL");
        // url.path_segments_mut()
        //     .expect("Cannot modify path segments")
        //     .pop_if_empty()
        //     .push(&file_id)
        //     .push("progress");

        // let client = self.blocking_client.clone();
        // std::thread::spawn(move || {
        //     let response = client.get(url).send();
        //     match response {
        //         Ok(res) => {
        //             let mut reader = std::io::BufReader::new(res);
        //             let mut line = String::new();
        //             let mut current_event = String::new();

        //             while reader.read_line(&mut line).unwrap() > 0 {
        //                 if line.starts_with("event: ") {
        //                     current_event = line.trim_start_matches("event: ").trim().to_string();
        //                 } else if line.starts_with("data: ") {
        //                     let event_data = line.trim_start_matches("data: ").trim();
        //                     match current_event.as_str() {
        //                         "complete" => {
        //                             let _ = tx.send(Ok(FileDownloadResponse::Completed(
        //                                 moly_protocol::data::DownloadedFile::default()
        //                             )));
        //                             break;
        //                         }
        //                         "error" => {
        //                             let _ = tx.send(Err(anyhow::anyhow!("Download failed")));
        //                             break;
        //                         }
        //                         "progress" => {
        //                             if let Ok(value) = event_data.parse::<f32>() {
        //                                 let _ = tx.send(Ok(FileDownloadResponse::Progress(file_id.clone(), value)));
        //                             }
        //                         }
        //                         _ => {}
        //                     }
        //                 }
        //                 line.clear();
        //             }
        //         }
        //         Err(e) => {
        //             let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
        //         }
        //     }
        // });
    }

    pub fn pause_download_file(&self, file_id: FileID, tx: Sender<Result<(), anyhow::Error>>) {
        let mut url = Url::parse(&format!("{}/downloads", self.address)).expect("Invalid Moly server URL");

        // Add the ID as a path segment (auto-encodes special characters)
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty() // Remove the trailing slash, if any
            .push(&file_id);

        let client = self.client.clone();
        tokio::spawn(async move {
            let resp = client.post(url)
                .json(&serde_json::json!({
                    "file_id": file_id
                }))
                .send().await;

            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        let _ = tx.send(Ok(()));
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn cancel_download_file(&self, file_id: FileID, tx: Sender<Result<(), anyhow::Error>>) {
        let mut url = Url::parse(&format!("{}/downloads", self.address)).expect("Invalid Moly server URL");

        // Add the ID as a path segment (auto-encodes special characters)
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty() // Remove the trailing slash, if any
            .push(&file_id);

        let client = self.client.clone();
        tokio::spawn(async move {
            let resp = client.delete(url)
                .json(&serde_json::json!({
                    "file_id": file_id
                }))
                .send().await;

            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        let _ = tx.send(Ok(()));
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn delete_file(&self, file_id: FileID, tx: Sender<Result<(), anyhow::Error>>) {
        let mut url = Url::parse(&format!("{}/files", self.address)).expect("Invalid Moly server URL");

        // Add the ID as a path segment (auto-encodes special characters)
        url.path_segments_mut()
            .expect("Cannot modify path segments")
            .pop_if_empty() // Remove the trailing slash, if any
            .push(&file_id);

        let client = self.client.clone();
        tokio::spawn(async move {
            let resp = client.delete(url)
                .send().await;

            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        let _ = tx.send(Ok(()));
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    /// Loads a model. Should only be called from a background thread to avoid blocking the UI.
    pub fn load_model(&self, file_id: FileID, options: LoadModelOptions,
        tx: Sender<Result<LoadModelResponse, anyhow::Error>>) {
        let url = format!("{}/models/load", self.address);
        let request = serde_json::json!({
            "file_id": file_id,
            "options": options,
        });

        let client = self.client.clone();
        tokio::spawn(async move {
            let resp = client.post(&url)
                .json(&request)
                .send().await;

            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<LoadModelResponse>().await {
                            Ok(response) => {
                                let _ = tx.send(Ok(response));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Failed to parse response: {}", e)));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                },
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    pub fn eject_model(&self, tx: Sender<Result<(), anyhow::Error>>) {
        let url = format!("{}/models/eject", self.address);
        let client = self.client.clone();

        tokio::spawn(async move {
            let resp = client.post(&url).send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        let _ = tx.send(Ok(()));
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }

    // TODO(MolyKit): This will be removed entirely from this client.
    // Ideally for chat, MolyServer will just used through the OpenAI Client in MoltKit.
    // This MolyClient client will be used exclusively for local file management.
    // Load and eject model will be baked into MolyServer, if you hit v1/chat/completions, MolyServer should
    // automatically load the model.
    pub fn send_chat_message(
        &self,
        request: ChatRequestData,
        tx: Sender<Result<ChatResponse, anyhow::Error>>,
    ) {
        // let client = self.client.clone();
        // let url = format!("{}/models/v1/chat/completions", self.address);

        // tokio::spawn(async move {
        //     let response = client
        //         .post(&url)
        //         .json(&request)
        //         .send().await;

        //     match response {
        //         Ok(res) => {
        //             if request.stream.unwrap_or(false) {
        //                 let mut reader = std::io::BufReader::new(res);
        //                 let mut line = String::new();
        //                 while reader.read_line(&mut line).unwrap() > 0 {
        //                     if line.starts_with("data: [DONE]") {
        //                         let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(ChatResponseChunkData {
        //                             id: String::new(),
        //                             choices: vec![ChunkChoiceData {
        //                                 finish_reason: Some(StopReason::Stop),
        //                                 index: 0,
        //                                 delta: MessageData {
        //                                     content: String::new(),
        //                                     role: Role::Assistant,
        //                                 },
        //                                 logprobs: None,
        //                             }],
        //                             created: 0,
        //                             model: String::new(),
        //                             system_fingerprint: String::new(),
        //                             object: "chat.completion.chunk".to_string(),
        //                         })));
        //                         break;
        //                     }
        //                     if line.starts_with("data: ") {
        //                         // Skip "data: " prefix (6 bytes)
        //                         let resp: Result<ChatResponseChunkData, _> =
        //                             serde_json::from_slice(line[6..].as_bytes());

        //                         match resp {
        //                             Ok(chunk_data) => {
        //                                 let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(chunk_data)));
        //                             }
        //                             Err(e) => {
        //                                 let _ = tx.send(Err(anyhow::anyhow!("Failed to parse chunk: {}", e)));
        //                                 break;
        //                             }
        //                         }
        //                     }
        //                     line.clear();
        //                 }
        //             } else {
        //                 match res.json::<ChatResponseData>() {
        //                     Ok(data) => {
        //                         let _ = tx.send(Ok(ChatResponse::ChatFinalResponseData(data)));
        //                     }
        //                     Err(e) => {
        //                         let _ = tx.send(Err(anyhow::anyhow!("Failed to parse response: {}", e)));
        //                     }
        //                 }
        //             }
        //         }
        //         Err(e) => {
        //             let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
        //         }
        //     }
        // });
    }
}
