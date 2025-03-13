use moly_protocol::open_ai::{
    ChatResponseData, ChoiceData, MessageData, Role, StopReason, UsageData,
};
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
        if should_be_real() {
            Self::new_real(address, api_key)
        } else {
            Self::new_fake(address)
        }
    }

    /// Handles the communication between the DeepInquireClient and the MoFa DeepInquire server.
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String, api_key: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
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

                    println!("Sending request to {}", address);

                    // Spawn in a separate thread to avoid blocking the tokio runtime
                    std::thread::spawn(move || {
                        // Create accumulated_content inside the thread
                        let mut accumulated_content = String::new();
                        
                        match req.send() {
                            Ok(res) => {
                                println!("Got response with status: {}", res.status());
                                
                                // Process streaming response line by line, similar to moly_client
                                let mut reader = std::io::BufReader::new(res);
                                let mut line = String::new();
                                
                                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                                    println!("Read line: {}", line.trim());
                                    
                                    if line.starts_with("data: [DONE]") {
                                        // Send the accumulated content as the final response
                                        let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                                            content: accumulated_content.clone(),
                                            articles: vec![],
                                        }));
                                        break;
                                    }
                                    
                                    if line.starts_with("data: ") {
                                        // Skip "data: " prefix (6 bytes)
                                        let json_str = &line[6..];
                                        
                                        if json_str == "[DONE]" {
                                            println!("Received [DONE] marker");
                                            // Send the accumulated content as the final response
                                            let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                                                content: accumulated_content.clone(),
                                                articles: vec![],
                                            }));
                                            break;
                                        }
                                        
                                        // Try to parse as our custom format first
                                        match serde_json::from_str::<DeepInquireResponse>(json_str) {
                                            Ok(deep_inquire_response) => {
                                                println!("Successfully parsed DeepInquire response");
                                                
                                                // Convert to standard ChatResponseData format
                                                if let Some(choice) = deep_inquire_response.choices.first() {
                                                    let content = choice.delta.content.clone();
                                                    println!("Content length: {}", content.len());
                                                    
                                                    // Accumulate content
                                                    accumulated_content.push_str(&content);
                                                    
                                                    // Extract articles from the response
                                                    let articles = choice.delta.articles.iter().map(|article| {
                                                        Article {
                                                            title: article.title.clone(),
                                                            url: article.url.clone(),
                                                            snippet: article.snippet.clone(),
                                                            source: article.source.clone(),
                                                            relevance: article.relevance,
                                                        }
                                                    }).collect::<Vec<Article>>();
                                                    
                                                    // Create a standard response with the content and articles
                                                    let response_data = ChatResponseData {
                                                        id: deep_inquire_response.id,
                                                        choices: vec![ChoiceData {
                                                            finish_reason: choice.finish_reason.clone().unwrap_or(StopReason::Stop),
                                                            index: choice.index,
                                                            message: MessageData {
                                                                content: accumulated_content.clone(), // Use accumulated content
                                                                role: Role::Assistant,
                                                            },
                                                            logprobs: None,
                                                        }],
                                                        created: deep_inquire_response.created as u32,
                                                        model: deep_inquire_response.model,
                                                        system_fingerprint: String::new(),
                                                        usage: UsageData {
                                                            completion_tokens: 0,
                                                            prompt_tokens: 0,
                                                            total_tokens: 0,
                                                        },
                                                        object: deep_inquire_response.object,
                                                    };
                                                    
                                                    let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                                                        content: response_data.choices[0].message.content.clone(),
                                                        articles
                                                    }));
                                                    
                                                    // Check if we're done
                                                    if choice.finish_reason.is_some() {
                                                        println!("Found finish_reason, breaking");
                                                        break;
                                                    }
                                                }
                                            },
                                            Err(e1) => {
                                                println!("Error parsing as DeepInquire format: {}", e1);
                                            }
                                        }
                                    } else if line.trim().is_empty() {
                                        // If we get an empty line and we have accumulated content,
                                        // send a final response with the accumulated content
                                        if !accumulated_content.is_empty() {
                                            println!("Received empty line, sending final response with accumulated content");
                                            let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                                                content: accumulated_content.clone(),
                                                articles: vec![],
                                            }));
                                        }
                                    }
                                    
                                    // Clear the line for the next iteration
                                    line.clear();
                                }
                                
                                // Send a final response with the accumulated content
                                if !accumulated_content.is_empty() {
                                    println!("Streaming complete, sending final response with accumulated content");
                                    let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                                        content: accumulated_content.clone(),
                                        articles: vec![],
                                    }));
                                }
                                
                                println!("Streaming complete");
                            },
                            Err(e) => {
                                eprintln!("Provider client error: {}", e);
                            }
                        }
                    });
                }
                ProviderCommand::CancelTask => {
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

    fn new_real(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone, api_key);
        });

        Self {
            command_sender,
        }
    }

    fn new_fake(address: String) -> Self {
        let (command_sender, command_receiver) = channel();

        let address_clone = address.clone();
        std::thread::spawn(move || {
            while let Ok(command) = command_receiver.recv() {
                match command {
                    ProviderCommand::SendMessage(_task, _agent, tx) => {
                        let content = r#"{
                            "step_name": "fake",
                            "node_results": "This is a fake response",
                            "dataflow_status": true
                        }"#
                        .to_string();

                        let data = ChatResponseData {
                            id: "fake".to_string(),
                            choices: vec![ChoiceData {
                                finish_reason: StopReason::Stop,
                                index: 0,
                                message: MessageData {
                                    content,
                                    role: Role::System,
                                },
                                logprobs: None,
                            }],
                            created: 0,
                            model: "fake".to_string(),
                            system_fingerprint: "".to_string(),
                            usage: UsageData {
                                completion_tokens: 0,
                                prompt_tokens: 0,
                                total_tokens: 0,
                            },
                            object: "".to_string(),
                        };
                        let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                            content: data.choices[0].message.content.clone(),
                            articles: vec![],
                        }));
                    }
                    ProviderCommand::FetchModels() => {
                        let agents = vec![RemoteModel {
                            id: RemoteModelId::from_model_and_server("DeepInquire", &address_clone),
                            name: "DeepInquire".to_string(),
                            description:
                                "A search assistant".to_string(),
                            enabled: true,
                            provider_url: address_clone.clone(),
                        }];
                        Cx::post_action(ProviderFetchModelsResult::Success(address_clone.clone(), agents));
                    }
                    ProviderCommand::CancelTask => {}
                }
            }
        });

        Self {
            command_sender,
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
struct Metadata {
    stage: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeltaContent {
    content: String,
    #[serde(default)]
    articles: Vec<Article>,
    #[serde(default)]
    metadata: Option<Metadata>,
    #[serde(default)]
    r#type: Option<String>,
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

fn should_be_real() -> bool {
    std::env::var("DEEPINQUIRE_BACKEND").as_deref().unwrap_or("real") != "fake"
}