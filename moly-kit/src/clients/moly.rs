use async_stream::stream;
use makepad_widgets::log;
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::utils::{serde::deserialize_null_default, sse::EVENT_TERMINATOR};
use crate::{protocol::*, utils::sse::rsplit_once_terminator};

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
    pub citations: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
struct MolyClientInner {
    url: String,
    headers: HeaderMap,
}

/// A client capable of interacting with Moly Server and other OpenAI-compatible APIs.
#[derive(Debug)]
pub struct MolyClient(Arc<Mutex<MolyClientInner>>);

impl Clone for MolyClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<MolyClientInner> for MolyClient {
    fn from(inner: MolyClientInner) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }
}

impl MolyClient {
    /// Creates a new client with the given OpenAI-compatible API URL.
    pub fn new(url: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        MolyClientInner {
            url,
            headers: HeaderMap::new(),
            ..Default::default()
        }
        .into()
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.0
            .lock()
            .unwrap()
            .headers
            .insert(HeaderName::from_str(key).unwrap(), value.parse().unwrap());
    }

    pub fn set_key(&mut self, key: &str) {
        self.set_header("Authorization", &format!("Bearer {}", key));
    }
}

impl BotClient for MolyClient {
    fn bots(&self) -> MolyFuture<'static, Result<Vec<Bot>, ()>> {
        let inner = self.0.clone();

        let future = async move {
            let request = reqwest::Client::new()
                .get(format!("{}/v1/models", inner.lock().unwrap().url))
                .headers(inner.lock().unwrap().headers.clone());

            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    log!("Error {:?}", error);
                    return Err(());
                }
            };

            let models: Models = match response.json().await {
                Ok(models) => models,
                Err(error) => {
                    log!("Error {:?}", error);
                    return Err(());
                }
            };

            let mut bots: Vec<Bot> = models
                .data
                .iter()
                .map(|m| Bot {
                    id: BotId::from(m.id.as_str()),
                    name: m.id.clone(),
                    // TODO: Handle this char as a grapheme.
                    avatar: Picture::Grapheme(m.id.chars().next().unwrap().to_string()),
                })
                .collect();

            bots.sort_by(|a, b| a.name.cmp(&b.name));

            Ok(bots)
        };

        moly_future(future)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        // ref should be shared but since hardcoded it should be ok
        Box::new(self.clone())
    }

    /// Stream pieces of content back as a ChatDelta instead of just a String.
    fn send_stream(
        &mut self,
        bot: &BotId,
        messages: &[Message],
    ) -> MolyStream<'static, Result<ChatDelta, ()>> {
        let moly_messages: Vec<OutcomingMessage> = messages
            .iter()
            .filter_map(|m| m.clone().try_into().ok())
            .collect();

        let (url, headers) = {
            let inner = self.0.lock().unwrap();
            (inner.url.clone(), inner.headers.clone())
        };

        let request = reqwest::Client::new()
            .post(format!("{}/v1/chat/completions", url))
            .headers(headers)
            .json(&serde_json::json!({
                "model": bot.as_str(),
                "messages": moly_messages,
                // Note: o1 only supports 1.0, it will error if other value is used.
                // "temperature": 0.7,
                "stream": true
            }));

        let stream = stream! {
            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    log!("Error {:?}", error);
                    yield Err(());
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
                        log!("Error {:?}", error);
                        yield Err(());
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
                    .map(|m| m.trim_start().split("data:").nth(1).unwrap())
                    .filter(|m| m.trim() != "[DONE]");

                for m in messages {
                    let completion: Completion = match serde_json::from_str(m) {
                        Ok(c) => c,
                        Err(error) => {
                            log!("Error: {:?}", error);
                            yield Err(());
                            return;
                        }
                    };

                    // Combine all partial choices content
                    let content_delta = completion
                        .choices
                        .iter()
                        .map(|c| c.delta.content.as_str())
                        .collect::<String>();

                    let citations = completion.citations.clone();

                    yield Ok(ChatDelta {
                        content_delta,
                        citations,
                    });
                }

                buffer = incomplete_message.to_vec();
            }
        };

        moly_stream(stream)
    }
}
