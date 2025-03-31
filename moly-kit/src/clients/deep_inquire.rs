use async_stream::stream;
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::protocol::*;
use crate::utils::sse::{rsplit_once_terminator, EVENT_TERMINATOR};

/// Article reference in a DeepInquire response
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Article {
    pub title: String,
    pub url: String,
}

/// Content of a stage in a DeepInquire response
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct StageContent {
    pub content: String,
    #[serde(default)]
    pub articles: Vec<Article>,
}

/// Type of delta received from DeepInquire API
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum DeltaType {
    #[serde(rename = "thinking")]
    Thinking { id: usize, content: StageContent },
    #[serde(rename = "writing")]
    Writing { id: usize, content: StageContent },
    #[serde(rename = "completed")]
    Completed { id: usize, content: StageContent },
}

/// A message being sent to the DeepInquire API
#[derive(Clone, Debug, Serialize)]
struct OutcomingMessage {
    pub content: String,
    pub role: Role,
}

impl TryFrom<Message> for OutcomingMessage {
    type Error = ();

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let role = match message.from {
            EntityId::User => Ok(Role::User),
            EntityId::System => Ok(Role::System),
            EntityId::Bot(_) => Ok(Role::Assistant),
            EntityId::App => Err(()),
        }?;

        Ok(Self {
            content: message.body,
            role,
        })
    }
}

/// Role of a message in the DeepInquire API
#[derive(Clone, Debug, Serialize, Deserialize)]
enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// The delta content as part of a streaming response
#[derive(Clone, Debug, Deserialize)]
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

/// The Choice object in a streaming response
#[derive(Clone, Debug, Deserialize)]
struct DeltaChoice {
    delta: DeltaContent,
    index: u32,
    finish_reason: Option<String>,
}

/// Response from the DeepInquire API
#[derive(Clone, Debug, Deserialize)]
struct DeepInquireResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeltaChoice>,
}

#[derive(Clone, Debug)]
struct DeepInquireClientInner {
    url: String,
    headers: HeaderMap,
    client: reqwest::Client,
}

/// A client for interacting with the DeepInquire API
#[derive(Debug)]
pub struct DeepInquireClient(Arc<RwLock<DeepInquireClientInner>>);

impl Clone for DeepInquireClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<DeepInquireClientInner> for DeepInquireClient {
    fn from(inner: DeepInquireClientInner) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }
}

impl DeepInquireClient {
    /// Creates a new client with the given DeepInquire API URL
    pub fn new(url: String) -> Self {
        let headers = HeaderMap::new();
        let client = default_client();

        DeepInquireClientInner {
            url,
            headers,
            client,
        }
        .into()
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.0
            .write()
            .unwrap()
            .headers
            .insert(HeaderName::from_str(key).unwrap(), value.parse().unwrap());
    }

    pub fn set_key(&mut self, key: &str) {
        self.set_header("Authorization", &format!("Bearer {}", key));
    }
}

impl BotClient for DeepInquireClient {
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
        let inner = self.0.read().unwrap().clone();

        // For now we return a hardcoded bot because DeepInquire does not support a /models endpoint
        let bot = Bot {
            id: BotId::new("DeepInquire", &inner.url),
            model_id: "DeepInquire".to_string(),
            provider_url: inner.url,
            name: "DeepInquire".to_string(),
            avatar: Picture::Grapheme("D".into()),
        };

        let future = async move {
            ClientResult::new_ok(vec![bot])
        };

        moly_future(future)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    fn send_stream(
        &mut self,
        bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageDelta>> {
        let inner = self.0.read().unwrap().clone();

        let url = format!("{}/chat/completions", inner.url);
        let headers = inner.headers;

        let moly_messages: Vec<OutcomingMessage> = messages
            .iter()
            .filter_map(|m| m.clone().try_into().ok())
            .collect();

        let request = inner
            .client
            .post(&url)
            .headers(headers)
            .timeout(Duration::from_secs(120))
            .json(&serde_json::json!({
                "model": bot.model_id.clone(),
                "messages": moly_messages,
                "stream": true
            }));

        let stream = stream! {
            let response = match request.send().await {
                Ok(response) => {
                    response
                },
                Err(error) => {
                    yield ClientError::new_with_source(
                        ClientErrorKind::Network,
                        format!("Could not send request to {url}. Verify your connection and the server status."),
                        Some(error),
                    ).into();
                    return;
                }
            };

            let event_terminator_str = std::str::from_utf8(EVENT_TERMINATOR).unwrap();
            let mut buffer: Vec<u8> = Vec::new();
            let bytes = response.bytes_stream();

            for await chunk in bytes {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        if error.is_timeout() {
                            eprintln!("Timeout waiting for chunk, continuing to wait... Error: {}", error);
                            continue;
                        } else {
                            yield ClientError::new_with_source(
                                ClientErrorKind::Network,
                                format!("Response streaming got interrupted while reading from {url}. This may be a problem with your connection or the server."),
                                Some(error),
                            ).into();
                            return;
                        }
                    }
                };

                buffer.extend_from_slice(&chunk);

                let Some((completed_messages, incomplete_message)) =
                    rsplit_once_terminator(&buffer)
                else {
                    continue;
                };

                // Silently drop any invalid utf8 bytes from the completed messages
                let completed_messages = String::from_utf8_lossy(completed_messages);

                let messages =
                    completed_messages
                    .split(event_terminator_str)
                    .filter(|m| !m.starts_with(":"))
                    // TODO: Return a format error instead of unwrapping
                    .map(|m| m.trim_start().split("data:").nth(1).unwrap())
                    .filter(|m| m.trim() != "[DONE]");

                for m in messages {
                    let response: DeepInquireResponse = match serde_json::from_str(m) {
                        Ok(c) => c,
                        Err(error) => {
                            yield ClientError::new_with_source(
                                ClientErrorKind::Format,
                                format!("Could not parse the SSE message from {url} as JSON or its structure does not match the expected format."),
                                Some(error),
                            ).into();
                            return;
                        }
                    };

                    // Process each choice in the response
                    for choice in &response.choices {
                        let delta = &choice.delta;
                        
                        // Convert article URLs to citation strings (could be improved)
                        let citations = delta.articles.iter()
                            .map(|article| article.url.clone())
                            .collect::<Vec<_>>();
                        
                        // Determine the stage type based on the delta's type field
                        let stage_block = if let Some(stage_type) = &delta.r#type {
                            match stage_type.as_str() {
                                "thinking" => Some(StageBlock::Thinking {
                                    id: delta.id,
                                    content: delta.content.clone(),
                                    citations: citations.clone(),
                                }),
                                "content" => Some(StageBlock::Writing {
                                    id: delta.id,
                                    content: delta.content.clone(),
                                    citations: citations.clone(),
                                }),
                                "completion" => Some(StageBlock::Completed {
                                    id: delta.id,
                                    content: delta.content.clone(),
                                    citations: citations.clone(),
                                }),
                                _ => None,
                            }
                        } else {
                            None
                        };

                        // TODO(Julian): Send the body as empty, the delta is already in the stage_block
                        // If there are any places using the body to apply the stage_block delta, update it to use the stage_block instead
                        yield ClientResult::new_ok(MessageDelta {
                            body: delta.content.clone(),
                            citations,
                            stage_block,
                        });
                    }
                }

                buffer = incomplete_message.to_vec();
            }
        };

        moly_stream(stream)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_client() -> reqwest::Client {
    // On native, there are no default timeouts. Connection may hang if we don't
    // configure them.
    reqwest::Client::builder()
        // Only considered while establishing the connection.
        .connect_timeout(Duration::from_secs(15))
        // Increase the read timeout considerably for DeepInquire's slow responses
        // DeepInquire might be slower than OpenAI because of the multi-stage processing
        .read_timeout(Duration::from_secs(60)) // Increased from 15s to 60s
        .build()
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn default_client() -> reqwest::Client {
    // On web, reqwest timeouts are not configurable, but it uses the browser's
    // fetch API under the hood, which handles connection issues properly.
    reqwest::Client::new()
}
