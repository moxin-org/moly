use moly_protocol::open_ai::{
    ChatResponseData, ChoiceData, MessageData, Role, StopReason, UsageData,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, channel, Sender};
use tokio::task::JoinHandle;
use makepad_widgets::Cx;
use super::providers::*;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MofaResponseReasoner {
//     pub task: String,
//     pub result: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MofaResponseResearchScholar {
//     pub task: String,
//     pub suggestion: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MofaResponseSearchAssistantResource {
//     pub name: String,
//     pub url: String,
//     pub snippet: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MofaResponseSearchAssistantResult {
//     pub web_search_results: String,
//     #[serde(deserialize_with = "parse_web_search_resource")]
//     pub web_search_resource: Vec<MofaResponseSearchAssistantResource>,
// }

// fn parse_web_search_resource<'de, D>(
//     deserializer: D,
// ) -> Result<Vec<MofaResponseSearchAssistantResource>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s: String = Deserialize::deserialize(deserializer)?;
//     let resources: Vec<MofaResponseSearchAssistantResource> =
//         serde_json::from_str(&s).map_err(serde::de::Error::custom)?;

//     Ok(resources)
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MofaResponseSearchAssistant {
//     pub task: String,
//     pub result: MofaResponseSearchAssistantResult,
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct AgentId(pub String);

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub enum AgentType {
//     Reasoner,
//     SearchAssistant,
//     ResearchScholar,
//     MakepadExpert,
// }

/// This is a OpenAI-compatible client for MoFa.
/// The only reason we have this is to return fake responses upon model fetching.
#[derive(Clone, Debug)]
pub struct MofaClient {
    command_sender: Sender<ProviderCommand>,
}

impl ProviderClient for MofaClient {
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

impl MofaClient {
    pub fn new(address: String) -> Self {
        if should_be_real() {
            Self::new_real(address)
        } else {
            Self::new_fake(address)
        }
    }

    /// Handles the communication between the MofaClient and the MoFa server.
    ///
    /// This function runs in a separate thread and processes commands received through the command channel.
    ///
    /// The loop continues until the command channel is closed or an unrecoverable error occurs.
    fn process_agent_commands(command_receiver: mpsc::Receiver<ProviderCommand>, address: String) {
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
                    };
                    let client = reqwest::Client::builder()
                        .no_proxy()
                        .build()
                        .expect("Failed to build a reqwest client for a MoFa server");

                    let req = client
                        .post(format!("{}/v1/chat/completions", &address))
                        .header("Content-Type", "application/json")
                        .json(&data);

                    current_request = Some(rt.spawn(async move {
                        let resp = req.send().await.expect("Failed to send request");

                        let resp: Result<ChatResponseData, reqwest::Error> = resp.json().await;
                        match resp {
                            Ok(resp) => {
                                let _ = tx.send(ChatResponse::ChatFinalResponseData(MolyChatResponse {
                                    content: resp.choices[0].message.content.clone(),
                                }, true));
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
                ProviderCommand::FetchModels() => {
                    let url = address.clone();
                    let resp = reqwest::blocking::ClientBuilder::new()
                        .timeout(std::time::Duration::from_secs(5))
                        .no_proxy()
                        .build()
                        .unwrap()
                        .get(format!("{}/v1/models", url))
                        .send();


                    match resp {
                        Ok(r) => {
                            match r.status() {
                                reqwest::StatusCode::OK => {
                                    let agents = vec![
                                        RemoteModel {
                                            id: RemoteModelId::from_model_and_server("Reasoner", &url),
                                            name: "Reasoner".to_string(),
                                            description: "An agent that will help you find good questions about any topic".to_string(),
                                            enabled: true,
                                            provider_url: url.clone(),
                                        },
                                    ];
                                    Cx::post_action(ProviderFetchModelsResult::Success(url, agents));
                                }
                                reqwest::StatusCode::UNAUTHORIZED => {
                                    eprintln!("Unauthorized to fetch models from: {}, your API key might be missing or invalid", url);
                                    Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::Unauthorized));
                                }
                                status => {
                                    eprintln!("Failed to fetch models from: {}, with status: {:?}", url, status);
                                    Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to fetch models from server: {e}");
                            Cx::post_action(ProviderFetchModelsResult::Failure(url, ProviderClientError::UnexpectedResponse));
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

    fn new_real(address: String) -> Self {
        let (command_sender, command_receiver) = channel();
        let address_clone = address.clone();
        std::thread::spawn(move || {
            Self::process_agent_commands(command_receiver, address_clone);
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
                        }, true));
                    }
                    ProviderCommand::FetchModels() => {
                        let agents = vec![RemoteModel {
                            id: RemoteModelId::from_model_and_server("Reasoner", &address_clone),
                            name: "Reasoner".to_string(),
                            description:
                                "An agent that will help you find good questions about any topic"
                                    .to_string(),
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

pub fn should_be_real() -> bool {
    std::env::var("MOFA_BACKEND").as_deref().unwrap_or("real") != "fake"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<MessageData>,
}
