use moly_protocol::{
    data::{DownloadedFile, File, FileID, Model, PendingDownload},
    open_ai::{ChatRequestData, ChatResponse, ChatResponseChunkData, ChatResponseData, ChunkChoiceData, MessageData, Role, StopReason},
    protocol::{LoadModelOptions, LoadModelResponse},
};
use std::sync::mpsc::Sender;
use std::io::BufRead;

#[derive(Clone, Debug)]
pub struct MolyClient {
    address: String,
    blocking_client: reqwest::blocking::Client,
}


// TODO(Julian):
// - Handle all errors properly
// - We might want to do some things async
// - We likely want to avoid spawning threads for each request

impl MolyClient {
    pub fn new(address: String) -> Self {
        let blocking_client = reqwest::blocking::Client::builder()
            .no_proxy()
            .build()
            .expect("Failed to build reqwest client");
            
        Self { 
            address, 
            blocking_client 
        }
    }

    pub fn get_featured_models(&self, tx: Sender<Result<Vec<Model>, anyhow::Error>>) {
        let url = format!("{}/models/featured", self.address);
        
        std::thread::spawn(move || {
            let resp = reqwest::blocking::get(&url);
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<Model>>() {
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
        
        std::thread::spawn(move || {
            // Using blocking for now to maintain similar behavior to current code
            let resp = reqwest::blocking::get(&url);
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<Model>>() {
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
        
        std::thread::spawn(move || {
            let resp = reqwest::blocking::get(&url);
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<DownloadedFile>>() {
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
        
        std::thread::spawn(move || {
            let resp = reqwest::blocking::get(&url);
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<Vec<PendingDownload>>() {
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
        let blocking_client = self.blocking_client.clone();

        std::thread::spawn(move || {
            let resp = blocking_client.post(&url)
                .json(&serde_json::json!({
                    "file_id": file.id
                }))
                .send();
            
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

    pub fn track_download_progress(&self, file_id: FileID, tx: Sender<Result<(), anyhow::Error>>) {
        // TODO(Julian): Implement this
        // let url = format!("{}/downloads/{}/progress", self.address, file_id);
        
        // let blocking_client = self.blocking_client.clone();
        // std::thread::spawn(move || {
        //     let resp = blocking_client.get(&url).send();
        //     match resp {
        //         Ok(r) => {
        //             if r.status().is_success() {
        //                 let _ = tx.send(Ok(()));
        //             } else {
        //                 let _ = tx.send(Err(anyhow::anyhow!("Server error: {}", r.status())));
        //             }
        //         }
        //         Err(e) => {
        //             let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
        //         }
        //     }
        // });
    }

    pub fn pause_download_file(&self, file_id: FileID, tx: Sender<Result<(), anyhow::Error>>) {
        let url = format!("{}/downloads", self.address);
        
        let blocking_client = self.blocking_client.clone();
        std::thread::spawn(move || {
            let resp = blocking_client.post(&url)
                .json(&serde_json::json!({
                    "file_id": file_id
                }))
                .send();
            
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
        let url = format!("{}/downloads", self.address);
        let blocking_client = self.blocking_client.clone();

        std::thread::spawn(move || {
            let resp = blocking_client.delete(&url)
                .json(&serde_json::json!({
                    "file_id": file_id
                }))
                .send();
            
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
        let url = format!("{}/files", self.address);
        let blocking_client = self.blocking_client.clone();

        std::thread::spawn(move || {
            let resp = blocking_client.delete(&url)
                .json(&serde_json::json!({
                    "file_id": file_id
                }))
                .send();
            
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

        let blocking_client = self.blocking_client.clone();
        std::thread::spawn(move || {
            let resp = blocking_client.post(&url)
                .json(&request)
                .send();
            
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        match r.json::<LoadModelResponse>() {
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
        let blocking_client = self.blocking_client.clone();

        std::thread::spawn(move || {
            let resp = blocking_client.post(&url).send();
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

    pub fn send_chat_message(
        &self,
        request: ChatRequestData,
        tx: Sender<Result<ChatResponse, anyhow::Error>>,
    ) {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/models/v1/chat/completions", self.address);

        std::thread::spawn(move || {
            let response = client
                .post(&url)
                .json(&request)
                .send();

            match response {
                Ok(res) => {
                    if request.stream.unwrap_or(false) {
                        let mut reader = std::io::BufReader::new(res);
                        let mut line = String::new();
                        while reader.read_line(&mut line).unwrap() > 0 {
                            if line.starts_with("data: [DONE]") {
                                let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(ChatResponseChunkData {
                                    id: String::new(),
                                    choices: vec![ChunkChoiceData {
                                        finish_reason: Some(StopReason::Stop),
                                        index: 0,
                                        delta: MessageData {
                                            content: String::new(),
                                            role: Role::Assistant,
                                        },
                                        logprobs: None,
                                    }],
                                    created: 0,
                                    model: String::new(),
                                    system_fingerprint: String::new(),
                                    object: "chat.completion.chunk".to_string(),
                                })));
                                break;
                            }
                            if line.starts_with("data: ") {
                                // Skip "data: " prefix (6 bytes)
                                let resp: Result<ChatResponseChunkData, _> = 
                                    serde_json::from_slice(line[6..].as_bytes());
                                
                                match resp {
                                    Ok(chunk_data) => {
                                        let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(chunk_data)));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(anyhow::anyhow!("Failed to parse chunk: {}", e)));
                                        break;
                                    }
                                }
                            }
                            line.clear();
                        }
                    } else {
                        match res.json::<ChatResponseData>() {
                            Ok(data) => {
                                let _ = tx.send(Ok(ChatResponse::ChatFinalResponseData(data)));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Failed to parse response: {}", e)));
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request failed: {}", e)));
                }
            }
        });
    }
}
