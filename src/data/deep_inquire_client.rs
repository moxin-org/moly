use moly_protocol::open_ai::{MessageData, Role, StopReason};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, channel, Sender};
use tokio::task::JoinHandle;
use makepad_widgets::Cx;

use super::providers::*;
use std::io::BufRead;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct AgentId(pub String);

#[derive(Clone, Debug)]
pub struct DeepInquireClient {
    command_sender: Sender<ProviderCommand>,
}

impl ProviderClient for DeepInquireClient {
    fn cancel_task(&self) {
        self.command_sender
            .send(ProviderCommand::CancelTask)
            .unwrap();
    }

    fn fetch_models(&self) {
        self.command_sender
            .send(ProviderCommand::FetchModels())
            .unwrap();
    }

    fn send_message(&self, model: &RemoteModel, prompt: &String, tx: Sender<ChatResponse>) {
        self.command_sender
            .send(ProviderCommand::SendMessage(
                prompt.clone(),
                model.clone(),
                tx,
            ))
            .unwrap();
    }
}

impl DeepInquireClient {
    pub fn new(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone, api_key);
        });

        Self {
            command_sender,
        }
    }

    /// Handles the communication between the DeepInquireClient and the MoFa DeepInquire server.
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String, api_key: Option<String>) {
        let mut current_request: Option<JoinHandle<()>> = None;

        while let Ok(command) = command_receiver.recv() {
            match command {
                ProviderCommand::SendMessage(task, agent, tx) => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }

                    let data = ChatRequest {
                        model: agent.name,
                        messages: vec![MessageData {
                            role: Role::User,
                            content: task,
                        }],
                        stream: Some(true),
                    };
                    
                    let client = reqwest::blocking::Client::builder()
                        .no_proxy()
                        .build()
                        .expect("Failed to build a reqwest client for a remote server");

                    let mut req = client
                        .post(format!("{}/chat/completions", &address))
                        .header("Content-Type", "application/json");

                    if let Some(key) = &api_key {
                        req = req.header("Authorization", format!("Bearer {}", key));
                    }

                    let req = req.json(&data);

                    // Spawn in a separate thread to avoid blocking the tokio runtime
                    std::thread::spawn(move || {
                        let mut latest_content = String::new();
                        let mut all_articles: Vec<Article> = Vec::new();
                        let mut received_completion = false;
                        
                        match req.send() {
                            Ok(res) => {
                                // Process streaming response line by line
                                let mut reader = std::io::BufReader::new(res);
                                let mut line = String::new();
                                
                                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                                    if line.starts_with("data: ") {
                                        // Skip "data: " prefix (6 bytes)
                                        let json_str = &line[6..];
                                        
                                        // Try to parse as our custom format
                                        match serde_json::from_str::<DeepInquireResponse>(json_str) {
                                            Ok(deep_inquire_response) => {
                                                if let Some(choice) = deep_inquire_response.choices.first() {
                                                    let message_type = choice.delta.r#type.clone().unwrap_or_default();

                                                    match message_type.as_str() {
                                                        "thinking" => {
                                                            let content = DeepInquireStageContent {
                                                                content: choice.delta.content.clone(),
                                                                articles: choice.delta.articles.clone(),
                                                            };

                                                            all_articles.extend(choice.delta.articles.clone());

                                                            let _ = tx.send(ChatResponse::DeepnInquireResponse(
                                                                DeepInquireMessage::Thinking(choice.delta.id, content)
                                                            ));
                                                        },
                                                        "content" => {
                                                            let new_content = choice.delta.content.clone();
                                                            // Overwrite the latest_content with the new chunk
                                                            latest_content = new_content;
                                                            
                                                            // Extract articles
                                                            let articles = choice.delta.articles
                                                                .iter()
                                                                .map(|article| Article {
                                                                    title: article.title.clone(),
                                                                    url: article.url.clone(),
                                                                    snippet: article.snippet.clone(),
                                                                    source: article.source.clone(),
                                                                    relevance: article.relevance,
                                                                })
                                                                .collect::<Vec<Article>>();
                                                            
                                                            all_articles.extend(articles.clone());

                                                            let content = DeepInquireStageContent {
                                                                content: latest_content.clone(),
                                                                articles: articles.clone(),
                                                            };

                                                            let _ = tx.send(ChatResponse::DeepnInquireResponse(
                                                                DeepInquireMessage::Writing(choice.delta.id, content)
                                                            ));
                                                        },
                                                        "completion" => {
                                                            let final_content = choice.delta.content.clone();
                                                            
                                                            latest_content = final_content;
                                                            
                                                            let articles = choice.delta.articles
                                                                .iter()
                                                                .map(|article| Article {
                                                                    title: article.title.clone(),
                                                                    url: article.url.clone(),
                                                                    snippet: article.snippet.clone(),
                                                                    source: article.source.clone(),
                                                                    relevance: article.relevance,
                                                                })
                                                                .collect::<Vec<Article>>();
                                                            
                                                            all_articles.extend(articles.clone());
                                                            // Remove duplicates
                                                            all_articles.sort_by_key(|a| a.url.clone());
                                                            all_articles.dedup_by_key(|a| a.url.clone());

                                                            let final_content = DeepInquireStageContent {
                                                                content: latest_content.clone(),
                                                                articles: all_articles.clone(),
                                                            };

                                                            let _ = tx.send(ChatResponse::DeepnInquireResponse(
                                                                DeepInquireMessage::Completed(choice.delta.id, final_content)
                                                            ));
                                                            received_completion = true;
                                                        },
                                                        // unknown -> just log
                                                        _ => {
                                                            eprintln!("Unknown message type: {}", message_type);
                                                        }
                                                    }
                                                }
                                            },
                                            Err(e1) => {
                                                eprintln!("Error parsing as DeepInquire format: {}", e1);
                                            }
                                        }
                                    } else if line.trim().is_empty() {
                                        // ignoring empty lines
                                    }
                                    
                                    // Clear the buffer for the next iteration
                                    line.clear();
                                }
                                
                                // If we never got a "completion" and there's text to show
                                if !received_completion && !latest_content.is_empty() {
                                    let final_content = DeepInquireStageContent {
                                        content: latest_content.clone(),
                                        articles: all_articles.clone(),
                                    };
                                    let _ = tx.send(ChatResponse::DeepnInquireResponse(DeepInquireMessage::Completed(0, final_content)));
                                }
                            },
                            Err(e) => {
                                eprintln!("Provider client error: {}", e);
                            }
                        }
                    });
                }
                ProviderCommand::CancelTask => {
                    // TODO(Julian): Fix cancel task, currentl does not take effect in the UI
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    continue;
                }
                ProviderCommand::FetchModels() => {
                    let agents = vec![
                        RemoteModel {
                            id: RemoteModelId::from_model_and_server("DeepInquire", &address),
                            name: "DeepInquire".to_string(),
                            description: "A search assistant".to_string(),
                            enabled: true,
                            provider_url: address.clone(),
                        },
                    ];
                    Cx::post_action(ProviderFetchModelsResult::Success(address.clone(), agents));
                }
            }
        }

        // Clean up any pending request when the channel is closed
        if let Some(handle) = current_request {
            handle.abort();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<MessageData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeltaContent {
    content: String,
    #[serde(default)]
    articles: Vec<Article>,
    #[serde(default)]
    metadata: serde_json::Value,
    #[serde(default)]
    r#type: Option<String>,
    id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeltaChoice {
    delta: DeltaContent,
    index: u32,
    finish_reason: Option<StopReason>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeepInquireResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeltaChoice>,
}
