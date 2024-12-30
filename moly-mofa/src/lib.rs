use moly_protocol::open_ai::{
    ChatResponseData, ChoiceData, MessageData, Role, StopReason, UsageData,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::mpsc::{self, channel, Sender};
use tokio::task::JoinHandle;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct MofaResponseReasoner {
    pub task: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MofaResponseResearchScholar {
    pub task: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MofaResponseSearchAssistantResource {
    pub name: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MofaResponseSearchAssistantResult {
    pub web_search_results: String,
    #[serde(deserialize_with = "parse_web_search_resource")]
    pub web_search_resource: Vec<MofaResponseSearchAssistantResource>,
}

fn parse_web_search_resource<'de, D>(
    deserializer: D,
) -> Result<Vec<MofaResponseSearchAssistantResource>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let resources: Vec<MofaResponseSearchAssistantResource> =
        serde_json::from_str(&s).map_err(serde::de::Error::custom)?;

    Ok(resources)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MofaResponseSearchAssistant {
    pub task: String,
    pub result: MofaResponseSearchAssistantResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(agent_name: &str, server_address: &str) -> Self {
        AgentId(format!("{}-{}", agent_name, server_address))
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
pub struct MofaServerId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MofaAgent {
    pub id: AgentId,
    pub name: String,
    pub description: String,
    pub agent_type: AgentType,
    pub server_id: MofaServerId,
}

impl MofaAgent {
    /// Returns a dummy agent whenever the corresponding Agent cannot be found
    /// (due to the server not being available, the server no longer providing the agent, etc.).
    pub fn unknown() -> Self {
        MofaAgent {
            id: AgentId("unknown".to_string()),
            name: "Inaccesible Agent".to_string(),
            description: "This agent is not currently reachable, its information is not available".to_string(),
            agent_type: AgentType::Reasoner,
            server_id: MofaServerId("Unknown".to_string()),
        }
    }
}

pub enum MofaServerResponse {
    Connected(String, Vec<MofaAgent>),
    Unavailable(String),
}

pub enum MofaAgentCommand {
    SendTask(String, MofaAgent, Sender<ChatResponse>),
    CancelTask,
    FetchAgentsFromServer(Sender<MofaServerResponse>),
}

#[derive(Debug, Serialize, Deserialize)]
struct MofaContent {
    step_name: String,
    node_results: String,
    dataflow_status: bool,
}

#[derive(Clone, Debug)]
pub struct MofaClient {
    command_sender: Sender<MofaAgentCommand>,
    pub address: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendType {
    Local,
    Remote,
}

impl MofaClient {
    pub fn cancel_task(&self) {
        self.command_sender.send(MofaAgentCommand::CancelTask).unwrap();
    }

    pub fn fetch_agents(&self, tx: Sender<MofaServerResponse>) {
        self.command_sender.send(MofaAgentCommand::FetchAgentsFromServer(tx)).unwrap();
    }

    pub fn send_message_to_agent(&self, agent: &MofaAgent, prompt: &String, tx: Sender<ChatResponse>) {
        self.command_sender.send(MofaAgentCommand::SendTask(prompt.clone(), agent.clone(), tx)).unwrap();
    }

    pub fn new(address: String) -> Self {
        if should_be_real() {
            let (command_sender, command_receiver) = channel();
            let address_clone = address.clone();
            std::thread::spawn(move || {
                Self::process_agent_commands(command_receiver, address_clone);
            });

            Self { command_sender, address }
        } else {
            Self::new_fake()
        }
    }

    /// Handles the communication between the MofaClient and the MoFa server.
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<MofaAgentCommand>, address: String) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut current_request: Option<JoinHandle<()>> = None;

        while let Ok(command) = command_receiver.recv() {
            match command {
                MofaAgentCommand::SendTask(task, agent, tx) => {
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

                    // If access to OpenAI or Claude is restricted in certain regions,  a VPN is typically used,
                    // so communication with the local MoFA service here must bypass the proxy.
                    let client = reqwest::Client::builder()
                        .no_proxy()
                        .build()
                        .expect("Failed to build client");

                    let req = client
                        .post(format!("{}/v1/chat/completions", &address))
                        .header("Content-Type", "application/json")
                        .json(&data);

                    current_request = Some(rt.spawn(async move {
                        match req.send().await {
                            Ok(response) => {
                                if response.status().is_success() {
                                    match response.text().await {
                                        Ok(text) => {
                                            match serde_json::from_str::<Value>(&text) {
                                                Ok(value) => {
                                                    if let Some(content) = value
                                                        .get("choices")
                                                        .and_then(|choices| choices.get(0))
                                                        .and_then(|choice| choice.get("message"))
                                                        .and_then(|message| message.get("content"))
                                                        .and_then(|content| content.as_str())
                                                    {
                                                        // parsing inner json
                                                        match serde_json::from_str::<MofaContent>(content) {
                                                            Ok(mofa_content) => {
                                                                let response_data = ChatResponseData {
                                                                    id: value.get("id").and_then(|id| id.as_str()).unwrap_or("").to_string(),
                                                                    choices: vec![ChoiceData {
                                                                        finish_reason: StopReason::Stop,
                                                                        index: 0,
                                                                        message: MessageData {
                                                                            content: mofa_content.node_results,
                                                                            role: Role::Assistant,
                                                                        },
                                                                        logprobs: None,
                                                                    }],
                                                                    created: value.get("created")
                                                                        .and_then(|c| c.as_i64())
                                                                        .unwrap_or(0),
                                                                    model: value.get("model").and_then(|m| m.as_str()).unwrap_or("").to_string(),
                                                                    system_fingerprint: "".to_string(),
                                                                    usage: UsageData {
                                                                        completion_tokens: 0,
                                                                        prompt_tokens: 0,
                                                                        total_tokens: 0,
                                                                    },
                                                                    object: value.get("object").and_then(|o| o.as_str()).unwrap_or("").to_string(),
                                                                };

                                                                let _ = tx.send(ChatResponse::ChatFinalResponseData(response_data));
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to parse content JSON: {}", e);
                                                                eprintln!("Content: {}", content);
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to parse response JSON: {}", e);
                                                    eprintln!("Response: {}", text);
                                                }
                                            }
                                        }
                                        Err(e) => eprintln!("Failed to get response text: {}", e),
                                    }
                                } else {
                                    eprintln!("HTTP error: {}", response.status());
                                    if let Ok(error_text) = response.text().await {
                                        eprintln!("Error details: {}", error_text);
                                    }
                                }
                            }
                            Err(e) => eprintln!("Request failed: {}", e),
                        }
                    }));
                }
                MofaAgentCommand::CancelTask => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    continue;
                }
                MofaAgentCommand::FetchAgentsFromServer(tx) => {
                    let url = address.clone();
                    let resp = reqwest::blocking::ClientBuilder::new()
                        .timeout(std::time::Duration::from_secs(5))
                        .no_proxy()
                        .build()
                        .unwrap()
                        .get(format!("{}/v1/models", url))
                        .send();

                    match resp {
                        Ok(r) if r.status().is_success() => {
                            let agents = vec![
                                MofaAgent {
                                    id: AgentId::new("Reasoner", &url),
                                    name: "Reasoner".to_string(),
                                    description: "An agent that will help you find good questions about any topic".to_string(),
                                    agent_type: AgentType::Reasoner,
                                    server_id: MofaServerId(url.clone()),
                                },
                            ];
                            tx.send(MofaServerResponse::Connected(url, agents)).unwrap();
                        }
                        _ => {
                            tx.send(MofaServerResponse::Unavailable(url)).unwrap();
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

    // For testing purposes
    pub fn new_fake() -> Self {
        let (command_sender, command_receiver) = channel();

        std::thread::spawn(move || {
            loop {
                match command_receiver.recv().unwrap() {
                    MofaAgentCommand::SendTask(_task, _agent, tx) => {
                        let data = ChatResponseData {
                            id: "fake".to_string(),
                            choices: vec![ChoiceData {
                                finish_reason: StopReason::Stop,
                                index: 0,
                                message: MessageData {
                                    content: "This is a fake response".to_string(),
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
                        let _ = tx.send(ChatResponse::ChatFinalResponseData(data));
                    }
                    _ => (),
                }
            }
        });

        Self { command_sender, address: "localhost:8000".to_string() }
    }
}

pub fn should_be_visible() -> bool {
    std::env::var("MOFA_FRONTEND").unwrap_or_default() == "visible"
}

pub fn should_be_real() -> bool {
    std::env::var("MOFA_BACKEND").unwrap_or_default() == "real"
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
