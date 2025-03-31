use async_stream::stream;
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::protocol::*;
use crate::utils::{
    serde::deserialize_null_default,
    sse::{rsplit_once_terminator, EVENT_TERMINATOR},
};

/// A model from the models endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Model {
    id: String,
}

/// Response from the models endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Models {
    pub data: Vec<Model>,
}

/// Message being received by the completions endpoint.
///
/// Although most OpenAI-compatible APIs return a `role` field, OpenAI itself does not.
///
/// Also, OpenAI may return an empty object as `delta` while streaming, that's why
/// content is optional.
///
/// And SiliconFlow may set `content` to a `null` value, that's why the custom deserializer
/// is needed.
#[derive(Clone, Debug, Deserialize, PartialEq)]
struct IncomingMessage {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_null_default")]
    pub content: String,
}

/// A message being sent to the completions endpoint.
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

/// Role of a message that is part of the conversation context.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum Role {
    /// OpenAI o1 models seems to expect `developer` instead of `system` according
    /// to the documentation. But it also seems like `system` is converted to `developer`
    /// internally.
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// The Choice object as part of a streaming response.
#[derive(Clone, Debug, Deserialize)]
struct Choice {
    pub delta: IncomingMessage,
}

/// Response from the completions endpoint
#[derive(Clone, Debug, Deserialize)]
struct Completion {
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub citations: Vec<String>,
}

#[derive(Clone, Debug)]
struct OpenAIClientInner {
    url: String,
    headers: HeaderMap,
    client: reqwest::Client,
}

/// A client capable of interacting with Moly Server and other OpenAI-compatible APIs.
#[derive(Debug)]
pub struct OpenAIClient(Arc<RwLock<OpenAIClientInner>>);

impl Clone for OpenAIClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<OpenAIClientInner> for OpenAIClient {
    fn from(inner: OpenAIClientInner) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }
}

impl OpenAIClient {
    /// Creates a new client with the given OpenAI-compatible API URL.
    pub fn new(url: String) -> Self {
        let headers = HeaderMap::new();
        let client = default_client();

        OpenAIClientInner {
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

impl BotClient for OpenAIClient {
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
        let inner = self.0.read().unwrap().clone();

        let url = format!("{}/models", inner.url);
        let headers = inner.headers;

        let request = inner.client.get(&url).headers(headers);

        let future = async move {
            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    return ClientError::new_with_source(
                        ClientErrorKind::Network,
                        format!("An error ocurred sending a request to {url}."),
                        Some(error),
                    )
                    .into();
                }
            };

            if !response.status().is_success() {
                let code = response.status().as_u16();
                return ClientError::new(
                    ClientErrorKind::Remote,
                    format!("Got unexpected HTTP status code {code} from {url}."),
                )
                .into();
            }

            let text = match response.text().await {
                Ok(text) => text,
                Err(error) => {
                    return ClientError::new_with_source(
                        ClientErrorKind::Format,
                        format!("Could not parse the response from {url} as valid text."),
                        Some(error),
                    )
                    .into();
                }
            };

            if text.is_empty() {
                return ClientError::new(
                    ClientErrorKind::Format,
                    format!("The response from {url} is empty."),
                )
                .into();
            }

            let models: Models = match serde_json::from_str(&text) {
                Ok(models) => models,
                Err(error) => {
                    return ClientError::new_with_source(
                        ClientErrorKind::Format,
                        format!("Could not parse the response from {url} as JSON or its structure does not match the expected format."),
                        Some(error),
                    ).into();
                }
            };

            let mut bots: Vec<Bot> = models
                .data
                .iter()
                .map(|m| Bot {
                    id: BotId::new(&m.id, &inner.url),
                    model_id: m.id.clone(),
                    provider_url: inner.url.clone(),
                    name: m.id.clone(),
                    // TODO: Handle this char as a grapheme.
                    avatar: Picture::Grapheme(m.id.chars().next().unwrap().to_string()),
                })
                .collect();

            bots.sort_by(|a, b| a.name.cmp(&b.name));

            ClientResult::new_ok(bots)
        };

        moly_future(future)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    /// Stream pieces of content back as a ChatDelta instead of just a String.
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
            .json(&serde_json::json!({
                "model": bot.model_id.clone(),
                "messages": moly_messages,
                // Note: o1 only supports 1.0, it will error if other value is used.
                // "temperature": 0.7,
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
                        yield ClientError::new_with_source(
                            ClientErrorKind::Network,
                            format!("Response streaming got interrupted while reading from {url}. This may be a problem with your connection or the server."),
                            Some(error),
                        ).into();
                        return;
                    }
                };

                buffer.extend_from_slice(&chunk);

                let Some((completed_messages, incomplete_message)) =
                    rsplit_once_terminator(&buffer)
                else {
                    continue;
                };

                // Silently drop any invalid utf8 bytes from the completed messages.
                let completed_messages = String::from_utf8_lossy(completed_messages);

                let messages =
                    completed_messages
                    .split(event_terminator_str)
                    .filter(|m| !m.starts_with(":"))
                    // TODO: Return a format error instead of unwraping.
                    .map(|m| m.trim_start().split("data:").nth(1).unwrap())
                    .filter(|m| m.trim() != "[DONE]");

                for m in messages {
                    let completion: Completion = match serde_json::from_str(m) {
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

                    let body = completion
                        .choices
                        .iter()
                        .map(|c| c.delta.content.as_str())
                        .collect::<String>();

                    let citations = completion.citations;

                    yield ClientResult::new_ok(MessageDelta {
                        body,
                        citations,
                        stage_block: None,
                    });
                }

                buffer = incomplete_message.to_vec();
            }
        };

        moly_stream(stream)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_client() -> reqwest::Client {
    // On native, there are no default timeouts. Connection may hand if we don't
    // configure them.
    reqwest::Client::builder()
        // Only considered while establishing the connection.
        .connect_timeout(Duration::from_secs(15))
        // Considered while reading the response and reset on every chunk
        // received.
        //
        // Warning: Do not use normal `timeout` method as it doesn't consider
        // this.
        .read_timeout(Duration::from_secs(15))
        .build()
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn default_client() -> reqwest::Client {
    // On web, reqwest timeouts are not configurable, but it uses the browser's
    // fetch API under the hood, which handles connection issues properly.
    reqwest::Client::new()
}
