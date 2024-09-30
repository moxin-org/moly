use moxin_protocol::open_ai::{MessageData, Role, UsageData};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashMap,
    sync::mpsc::{self, channel},
};
use tokio::task::JoinHandle;


#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseReasoner {
    pub task: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseResearchScholar {
    pub task: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseSearchAssistantResource {
    pub name: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseSearchAssistantResult {
    pub web_search_results: String,
    #[serde(deserialize_with = "parse_web_search_resource")]
    pub web_search_resource: Vec<MaeResponseSearchAssistantResource>,
}

fn parse_web_search_resource<'de, D>(
    deserializer: D,
) -> Result<Vec<MaeResponseSearchAssistantResource>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let resources: Vec<MaeResponseSearchAssistantResource> =
        serde_json::from_str(&s).map_err(serde::de::Error::custom)?;

    Ok(resources)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseSearchAssistant {
    pub task: String,
    pub result: MaeResponseSearchAssistantResult,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaeAgent {
    Reasoner,
    SearchAssistant,
    ResearchScholar,
}

pub enum MaeAgentWorkflow {
    BasicReasoner(String),
    ResearchScholar,
}

impl MaeAgent {
    pub fn name(&self) -> String {
        match self {
            MaeAgent::Reasoner => "Reasoner Agent".to_string(),
            MaeAgent::SearchAssistant => "Search Assistant".to_string(),
            MaeAgent::ResearchScholar => "Research Scholar".to_string(),
        }
    }

    pub fn short_description(&self) -> String {
        match self {
            MaeAgent::Reasoner => "Helps to find good questions about any topic".to_string(),
            MaeAgent::SearchAssistant => {
                "Your assistant to find information on the web".to_string()
            }
            MaeAgent::ResearchScholar => "Expert in academic research".to_string(),
        }
    }
}

pub enum MaeAgentCommand {
    SendTask(String, MaeAgent, mpsc::Sender<ChatResponse>),
    CancelTask,
}

pub struct MaeBackend {
    pub command_sender: mpsc::Sender<MaeAgentCommand>,
}

impl Default for MaeBackend {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl MaeBackend {
    pub fn available_agents() -> Vec<MaeAgent> {
        vec![
            MaeAgent::Reasoner,
            MaeAgent::SearchAssistant,
            MaeAgent::ResearchScholar,
        ]
    }

    pub fn new(options: HashMap<String, String>) -> Self {
        if should_be_fake() {
            return Self::new_fake();
        }

        Self::new_with_options(options)
    }

    pub fn new_with_options(options: HashMap<String, String>) -> Self {
        let (command_sender, command_receiver) = channel();

        let backend = Self { command_sender };

        std::thread::spawn(move || {
            Self::main_loop(command_receiver, options);
        });

        backend
    }

    pub fn main_loop(
        command_receiver: mpsc::Receiver<MaeAgentCommand>,
        options: HashMap<String, String>,
    ) {
        println!("MoFa backend started");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut current_request: Option<JoinHandle<()>> = None;

        loop {
            match command_receiver.recv().unwrap() {
                MaeAgentCommand::SendTask(task, _agent, tx) => {
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
                    let request_body = serde_json::to_string(&data).unwrap();
                    let client = reqwest::Client::new();
                    current_request = Some(rt.spawn(async move {
                        let resp = client.post("http://localhost:9901/api/chat/completions")
                            .json(&data)
                            .send()
                            .await
                            .expect("Failed to send request");

                        let resp: Result<ChatResponseData, reqwest::Error> = resp.json().await;
                        match resp {
                            Ok(resp) => {
                                dbg!(request_body, &resp);
                                let _ = tx.send(ChatResponse::ChatFinalResponseData(resp.clone()));
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    }));
                },
                MaeAgentCommand::CancelTask => {
                    dbg!("Canceling task");
                    if let Some(handle) = current_request.take() {
                        handle.abort();
                    }
                    continue;
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
                    MaeAgentCommand::SendTask(_task, _agent, tx) => {
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
                    },
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

// TODO Fix stop reason "complete" in MoFa

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StopReason {
    #[serde(rename = "complete")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "content_filter")]
    ContentFilter,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChoiceData {
    pub finish_reason: StopReason,
    pub index: u32,
    pub message: MessageData,
    // todo: ask for a fix in MoFa
    pub logprobs: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponseData {
    pub id: String,
    pub choices: Vec<ChoiceData>,
    pub created: u32,
    pub model: String,
    #[serde(default)]
    pub system_fingerprint: String,
    pub usage: UsageData,

    #[serde(default = "response_object")]
    pub object: String,
}

fn response_object() -> String {
    "chat.completion".to_string()
}

// TODO remove this, use the one defined in moxin-protocol when possible
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
