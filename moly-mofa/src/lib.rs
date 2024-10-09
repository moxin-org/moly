use moly_protocol::open_ai::{ChatResponseData, ChoiceData, MessageData, Role, StopReason, UsageData};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::mpsc::{self, channel};
use tokio::task::JoinHandle;

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum MofaAgent {
    Reasoner,
    SearchAssistant,
    ResearchScholar,
}

pub enum MofaAgentWorkflow {
    BasicReasoner(String),
    ResearchScholar,
}

impl MofaAgent {
    pub fn name(&self) -> String {
        match self {
            MofaAgent::Reasoner => "Reasoner Agent".to_string(),
            MofaAgent::SearchAssistant => "Search Assistant".to_string(),
            MofaAgent::ResearchScholar => "Research Scholar".to_string(),
        }
    }

    pub fn short_description(&self) -> String {
        match self {
            MofaAgent::Reasoner => "Helps to find good questions about any topic".to_string(),
            MofaAgent::SearchAssistant => {
                "Your assistant to find information on the web".to_string()
            }
            MofaAgent::ResearchScholar => "Expert in academic research".to_string(),
        }
    }
}

#[derive(Default)]
pub struct MofaOptions {
    pub address: Option<String>,
}

pub enum TestServerResponse {
    Success,
    Failure,
}

pub enum MofaAgentCommand {
    SendTask(String, MofaAgent, mpsc::Sender<ChatResponse>),
    CancelTask,
    UpdateServerAddress(String),
    TestServer(mpsc::Sender<TestServerResponse>),
}

pub struct MofaBackend {
    pub command_sender: mpsc::Sender<MofaAgentCommand>,
}

impl Default for MofaBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MofaBackend {
    pub fn available_agents() -> Vec<MofaAgent> {
        vec![
            MofaAgent::Reasoner,
            // Keeping only one agent for now. We will revisit this later when MoFa is more stable.

            // MofaAgent::SearchAssistant,
            // MofaAgent::ResearchScholar,
        ]
    }

    pub fn new() -> Self {
        if should_be_fake() {
            return Self::new_fake();
        }

        let (command_sender, command_receiver) = channel();
        let backend = Self { command_sender };

        std::thread::spawn(move || {
            Self::main_loop(command_receiver);
        });

        backend
    }

    pub fn main_loop(command_receiver: mpsc::Receiver<MofaAgentCommand>) {
        println!("MoFa backend started");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut current_request: Option<JoinHandle<()>> = None;
        let mut options = MofaOptions::default();

        loop {
            match command_receiver.recv().unwrap() {
                MofaAgentCommand::SendTask(task, _agent, tx) => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    let data = ChatRequest {
                        model: "simple".to_string(),
                        messages: vec![MessageData {
                            role: Role::User,
                            content: task,
                        }],
                    };
                    let client = reqwest::Client::new();
                    let url = options.address.clone().unwrap_or("http://localhost:8000".to_string());
                    current_request = Some(rt.spawn(async move {
                        let resp = client
                            .post(format!("{}/v1/chat/completions", url))
                            .json(&data)
                            .send()
                            .await
                            .expect("Failed to send request");

                        let resp: Result<ChatResponseData, reqwest::Error> = resp.json().await;
                        match resp {
                            Ok(resp) => {
                                let _ = tx.send(ChatResponse::ChatFinalResponseData(resp.clone()));
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    }));
                }
                MofaAgentCommand::CancelTask => {
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    continue;
                }
                MofaAgentCommand::UpdateServerAddress(address) => {
                    options.address = Some(address);
                }
                MofaAgentCommand::TestServer(tx) => {
                    let url = options.address.clone().unwrap_or("http://localhost:8000".to_string());
                    let resp = reqwest::blocking::ClientBuilder::new()
                        .timeout(std::time::Duration::from_secs(5))
                        .no_proxy()
                        .build()
                        .unwrap()
                        .get(format!("{}/v1/models", url))
                        .send();

                    match resp {
                        Ok(r) => {
                            if r.status().is_success() {
                                tx.send(TestServerResponse::Success).unwrap();
                            } else {
                                tx.send(TestServerResponse::Failure).unwrap();
                            }
                        }
                        Err(e) => {
                            tx.send(TestServerResponse::Failure).unwrap();
                            eprintln!("{e}");
                        }
                    };
                }
            }
        }
    }

    // For testing purposes
    pub fn new_fake() -> Self {
        let (command_sender, command_receiver) = channel();
        let backend = Self { command_sender };

        std::thread::spawn(move || {
            loop {
                // Receive command from frontend
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

        backend
    }
}

pub fn should_be_fake() -> bool {
    std::env::var("MAE_BACKEND").unwrap_or_default() == "fake"
}

#[derive(Clone, Debug)]
pub enum ChatResponse {
    // https://platform.openai.com/docs/api-reference/chat/object
    ChatFinalResponseData(ChatResponseData),
}

// ====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<MessageData>,
}
