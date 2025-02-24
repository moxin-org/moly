use moly_protocol::{data::ModelID, open_ai::{
    ChatResponseData, ChoiceData, MessageData, Role, UsageData,
}};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, channel, Sender};
use tokio::task::JoinHandle;

use super::chats::ServerConnectionStatus;

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
        return model_id.contains("gemini")
    }

    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct RemoteModelId(pub String);

impl RemoteModelId {
    pub fn from_model_and_server(agent_name: &str, server_address: &str) -> Self {
        RemoteModelId(format!("{}-{}", agent_name, server_address))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    Reasoner,
    SearchAssistant,
    ResearchScholar,
    MakepadExpert,
}

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoteServerId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteModel {
    pub id: RemoteModelId,
    pub name: String,
    pub description: String,
    pub server_id: RemoteServerId,
    pub enabled: bool,
}

impl RemoteModel {
    /// Returns a dummy agent whenever the corresponding Agent cannot be found
    /// (due to the server not being available, the server no longer providing the agent, etc.).
    pub fn unknown() -> Self {
        RemoteModel {
            id: RemoteModelId("unknown".to_string()),
            name: "Inaccesible model - check your connections".to_string(),
            description: "This model is not currently reachable, its information is not available"
                .to_string(),
            server_id: RemoteServerId("Unknown".to_string()),
            enabled: true,
        }
    }
}

pub enum OpenAIServerResponse {
    Connected(String, Vec<RemoteModel>),
    Unavailable(String),
}

pub enum RemoteModelCommand {
    SendTask(String, RemoteModel, Sender<ChatResponse>),
    CancelTask,
    FetchAgentsFromServer(Sender<OpenAIServerResponse>),
}

#[derive(Clone, Debug)]
pub struct OpenAIClient {
    command_sender: Sender<RemoteModelCommand>,
    pub address: String,
    api_key: Option<String>,
    pub connection_status: ServerConnectionStatus,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendType {
    Local,
    Remote,
}

impl OpenAIClient {
    pub fn cancel_task(&self) {
        self.command_sender
            .send(RemoteModelCommand::CancelTask)
            .unwrap();
    }

    pub fn fetch_agents(&self, tx: Sender<OpenAIServerResponse>) {
        self.command_sender
            .send(RemoteModelCommand::FetchAgentsFromServer(tx))
            .unwrap();
    }

    pub fn send_message_to_agent(
        &self,
        agent: &RemoteModel,
        prompt: &String,
        tx: Sender<ChatResponse>,
    ) {
        self.command_sender
            .send(RemoteModelCommand::SendTask(
                prompt.clone(),
                agent.clone(),
                tx,
            ))
            .unwrap();
    }

    pub fn with_api_key(address: String, api_key: String) -> Self {
        let mut client = Self::new_real(address, Some(api_key.clone()));
        client.api_key = Some(api_key);
        client
    }

    /// Handles the communication between the OpenAIClient and the remote server
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<RemoteModelCommand>, address: String, api_key: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut current_request: Option<JoinHandle<()>> = None;

        while let Ok(command) = command_receiver.recv() {
            match command {
                RemoteModelCommand::SendTask(task, agent, tx) => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }

                    let data = ChatRequest {
                        model: agent.name,
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
                RemoteModelCommand::CancelTask => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    continue;
                }
                RemoteModelCommand::FetchAgentsFromServer(tx) => {
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
                        object: String,
                        // may not be present
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
                                                    name: model.id,
                                                    description: format!("OpenAI {} model", model.object),
                                                    server_id: RemoteServerId(url.clone()),
                                                    enabled: true,
                                                })
                                                .collect();
                                            tx.send(OpenAIServerResponse::Connected(url, models)).unwrap();
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse models from server: {:?}", e);
                                            tx.send(OpenAIServerResponse::Unavailable(url)).unwrap();
                                        }
                                    }
                                }
                                status => {
                                    eprintln!("Failed to fetch models from server {:?}", status);
                                    tx.send(OpenAIServerResponse::Unavailable(url)).unwrap();
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to fetch models from server: {e}");
                            tx.send(OpenAIServerResponse::Unavailable(url)).unwrap();
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

    fn new_real(address: String, api_key: Option<String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        let api_key_clone = api_key.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone, api_key_clone);
        });

        Self {
            command_sender,
            address,
            api_key: None,
            connection_status: ServerConnectionStatus::Disconnected,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChatResponse {
    // https://platform.openai.com/docs/api-reference/chat/object
    ChatFinalResponseData(ChatResponseData),
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