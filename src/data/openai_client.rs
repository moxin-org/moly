use moly_protocol::{data::ModelID, open_ai::{
    ChatResponseData, ChoiceData, MessageData, Role, UsageData,
}};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, channel, Sender};
use tokio::task::JoinHandle;

use super::providers::{ChatResponse, ProviderClient, ProviderConnectionResult, RemoteModel, RemoteModelId, ProviderCommand};

const ALLOWED_OPENAI_MODELS: &[&str] = &[
    "gpt-4-turbo",
    "gpt-3.5",
    "gpt-4o",
    "gpt-4o-mini",
    "o1",
    "o1-mini",
    "o3-mini",
    "o1-preview",
];

const ALLOWED_GEMINI_MODELS: &[&str] = &[
    "models/gemini-1.5-flash",
    "models/gemini-1.5-pro",
    "models/gemini-2.0-flash",
    "models/gemini-2.0-pro",
];

const ALLOWED_SILICONFLOW_MODELS: &[&str] = &[
    "Qwen/Qwen2-72B-Instruct",
    "Pro/Qwen/Qwen2-7B-Instruct",
    "meta-llama/Meta-Llama-3.1-8B-Instruct",
    "deepseek-ai/DeepSeek-V2.5",
    "Pro/deepseek-ai/DeepSeek-V3",
];

const ALLOWED_OPENROUTER_MODELS: &[&str] = &[
    "anthropic/claude-3.7-sonnet",
    "perplexity/sonar",
    "deepseek/deepseek-r1",
];

fn should_include_model(url: &str, model_id: &str) -> bool {
    // First, filter out non-chat models
    if model_id.contains("dall-e") || 
       model_id.contains("whisper") || 
       model_id.contains("tts") ||
       model_id.contains("davinci") ||
       model_id.contains("audio") ||
       model_id.contains("babbage") ||
       model_id.contains("2024") || // Filtering out specific variants of popular models
       model_id.contains("moderation") ||
       model_id.contains("latest") ||
       model_id.contains("16k") ||
       model_id.contains("instruct") ||
       model_id.contains("embedding") {
        return false;
    }

    // For OpenAI specifically, only include our allowed list
    if url.contains("openai.com") {
        return ALLOWED_OPENAI_MODELS.iter().any(|&allowed| model_id.eq(allowed))
    }

    // Gemini
    if url.contains("googleapis.com") {
        return ALLOWED_GEMINI_MODELS.iter().any(|&allowed| model_id.eq(allowed))
    }

    // SiliconFlow
    if url.contains("siliconflow.cn") {
        return ALLOWED_SILICONFLOW_MODELS.iter().any(|&allowed| model_id.eq(allowed))
    }

    // OpenRouter
    if url.contains("openrouter.ai") {
        return ALLOWED_OPENROUTER_MODELS.iter().any(|&allowed| model_id.eq(allowed))
    }

    true
}

#[derive(Clone, Debug)]
pub struct OpenAIClient {
    command_sender: Sender<ProviderCommand>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendType {
    Local,
    Remote,
}

impl ProviderClient for OpenAIClient {
    fn cancel_task(&self) {
        self.command_sender
            .send(ProviderCommand::CancelTask)
            .unwrap();
    }

    fn fetch_models(&self, tx: Sender<ProviderConnectionResult>) {
        self.command_sender
            .send(ProviderCommand::FetchModels(tx))
            .unwrap();
    }

    fn send_message(
        &self,
        model: &RemoteModel,
        prompt: &String,
        tx: Sender<ChatResponse>,
    ) {
        self.command_sender
            .send(ProviderCommand::SendTask(
                prompt.clone(),
                model.clone(),
                tx,
            ))
            .unwrap();
    }
}

impl OpenAIClient {
    pub fn new(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        let api_key_clone = api_key.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone, api_key_clone);
        });

        Self {
            command_sender,
        }
    }

    /// Handles the communication between the OpenAIClient and the remote server
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String, api_key: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut current_request: Option<JoinHandle<()>> = None;

        while let Ok(command) = command_receiver.recv() {
            match command {
                ProviderCommand::SendTask(task, model, tx) => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }

                    let data = ChatRequest {
                        model: model.name,
                        messages: vec![MessageData {
                            role: Role::User,
                            content: task,
                        }],
                    };
                    let client = reqwest::Client::builder()
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

                    current_request = Some(rt.spawn(async move {
                        let resp = req.send().await.expect("Failed to send request");

                        let resp: Result<ChatResponseDataWrapper, reqwest::Error> = resp.json().await;
                        match resp {
                            Ok(resp) => {
                                let _ = tx.send(ChatResponse::ChatFinalResponseData(resp.into()));
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    }));
                }
                ProviderCommand::CancelTask => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    continue;
                }
                ProviderCommand::FetchModels(tx) => {
                    let url = address.clone();
                    let client = reqwest::blocking::ClientBuilder::new()
                        .timeout(std::time::Duration::from_secs(5))
                        .no_proxy()
                        .build()
                        .unwrap();
                    
                    let mut req = client.get(format!("{}/models", url));

                    // Add Authorization header if API key is available
                    if let Some(key) = &api_key {
                        req = req.header("Authorization", format!("Bearer {}", key));
                    }

                    let resp = req.send();

                    #[allow(dead_code)]
                    #[derive(Deserialize, Debug)]
                    struct ModelInfo {
                        id: String,
                        // may not be present
                        object: Option<String>,
                        created: Option<i64>,
                        owned_by: Option<String>,
                    }

                    #[allow(dead_code)]
                    #[derive(Deserialize, Debug)]
                    struct ModelsResponse {
                        object: Option<String>,
                        data: Vec<ModelInfo>,
                    }

                    match resp {
                        Ok(r) => {
                            match r.status() {
                                reqwest::StatusCode::OK => {
                                    match r.json::<ModelsResponse>() {
                                        Ok(models) => {
                                            let models: Vec<RemoteModel> = models.data.into_iter()
                                                .filter(|model| should_include_model(&url, &model.id))
                                                .map(|model| RemoteModel {
                                                    id: RemoteModelId::from_model_and_server(&model.id, &url),
                                                    name: model.id.clone(),
                                                    description: format!("OpenAI {} model", model.object.unwrap_or(model.id)),
                                                    provider_url: url.clone(),
                                                    enabled: true,
                                                })
                                                .collect();
                                            tx.send(ProviderConnectionResult::Connected(url, models)).unwrap();
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse models from server: {:?}", e);
                                            tx.send(ProviderConnectionResult::Unavailable(url)).unwrap();
                                        }
                                    }
                                }
                                status => {
                                    eprintln!("Failed to fetch models from server, status: {:?}", status);
                                    tx.send(ProviderConnectionResult::Unavailable(url)).unwrap();
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to fetch models from server: {e}");
                            tx.send(ProviderConnectionResult::Unavailable(url)).unwrap();
                        }
                    }
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
}

// Workaround for providers with missing fields in their responses
#[derive(Clone, Debug, Deserialize)]
struct ChatResponseDataWrapper {
    #[serde(default = "default_id")]
    id: String,
    choices: Vec<ChoiceData>,
    created: u32,
    model: ModelID,
    #[serde(default)]
    system_fingerprint: String,
    usage: UsageData,
    #[serde(default = "response_object")]
    object: String,
}

fn default_id() -> String {
    "unknown".to_string()
}

fn response_object() -> String {
    "chat.completion".to_string()
}

impl From<ChatResponseDataWrapper> for ChatResponseData {
    fn from(wrapper: ChatResponseDataWrapper) -> Self {
        Self {
            id: wrapper.id,
            choices: wrapper.choices,
            created: wrapper.created,
            model: wrapper.model,
            system_fingerprint: wrapper.system_fingerprint,
            usage: wrapper.usage,
            object: wrapper.object,
        }
    }
}
