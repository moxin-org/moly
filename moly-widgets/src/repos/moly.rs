use futures::{future::FutureExt, stream::StreamExt, SinkExt};
use makepad_widgets::log;
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{protocol::*, utils::asynchronous::spawn};

/// A model from the models endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Model {
    id: String,
}

/// Response from the models endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Models {
    pub data: Vec<Model>,
}

/// Message being received by the completions endpoint.
///
/// Although most OpenAI-compatible APIs return a `role` field, OpenAI itself does not.
/// Also, OpenAI may return an empty object as `delta` while streaming, that's why
/// content is optional.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct IncomingMessage {
    #[serde(default)]
    pub content: String,
}

/// A message being sent to the completions endpoint.
#[derive(Clone, Debug, Serialize)]
pub struct OutcomingMessage {
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
pub enum Role {
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

#[derive(Clone, Debug, Deserialize)]
pub struct Choice {
    pub delta: IncomingMessage,
}

/// Response from the completions endpoint.
#[derive(Clone, Debug, Deserialize)]
pub struct Completation {
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, Default)]
struct MolyClientInner {
    url: String,
    headers: HeaderMap,
}

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
    fn bots(&self) -> BoxFuture<Result<Vec<Bot>, ()>> {
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

            let bots: Vec<Bot> = models
                .data
                .iter()
                .map(|m| Bot {
                    id: BotId::from(m.id.as_str()),
                    name: m.id.clone(),
                    avatar: Picture::Grapheme(m.id.chars().next().unwrap().to_string()),
                })
                .collect();

            Ok(bots)
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            future.boxed()
        }

        #[cfg(target_arch = "wasm32")]
        future.boxed_local()
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        // ref should be shared but since hardcoded it should be ok
        Box::new(self.clone())
    }

    fn send_stream(&mut self, bot: BotId, messages: &[Message]) -> BoxStream<Result<String, ()>> {
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

        // The `async-stream` crate and macro internally use a channel to create the stream
        // imperatively. Where `yield` is mapped to `sender.send().await` and `await for` is just
        // a `while let` over `stream.next().await`.
        //
        // By doing this manually we win:
        // - One less direct dependency.
        // - Proper auto-completition, auto-formatting and LSP support.
        let (mut sender, receiver) = futures::channel::mpsc::channel(0);

        spawn(async move {
            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    log!("Error {:?}", error);
                    sender.send(Err(())).await.unwrap();
                    return;
                }
            };

            let mut buffer = String::new();
            let mut bytes = response.bytes_stream();

            while let Some(chunk) = bytes.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        log!("Error {:?}", error);
                        sender.send(Err(())).await.unwrap();
                        return;
                    }
                };

                // TODO: Chunk may contain eventually valid utf8 bytes that would be discarded
                // by "from string loosly".
                //
                // This is partially safe assuming everything before `\n\n` is valid utf8 as it will be
                // splitted later.
                //
                // But this is not actually safe because it trusts the server on not sending utf8
                // before a `\n\n`.
                //
                // So, let's change the buffer type later and extract valid utf8 strings from there later.
                buffer.push_str(unsafe { &String::from_utf8_unchecked(chunk.to_vec()) });

                const EVENT_TERMINATOR: &'static str = "\n\n";

                let Some((completed_messages, incomplete_message)) =
                    buffer.rsplit_once(EVENT_TERMINATOR)
                else {
                    continue;
                };

                let messages = completed_messages
                    .split(EVENT_TERMINATOR)
                    .map(|m| m.trim_start().split("data:").nth(1).unwrap())
                    .filter(|m| m.trim() != "[DONE]");

                for m in messages {
                    let completition: Completation = match serde_json::from_str(m) {
                        Ok(completition) => completition,
                        Err(error) => {
                            log!("Error: {:?}", error);
                            sender.send(Err(())).await.unwrap();
                            return;
                        }
                    };

                    let text = completition
                        .choices
                        .iter()
                        .map(|c| c.delta.content.as_str())
                        .collect::<String>();

                    sender.send(Ok(text)).await.unwrap();
                }

                buffer = incomplete_message.to_string();
            }
        });

        #[cfg(not(target_arch = "wasm32"))]
        return receiver.boxed();

        #[cfg(target_arch = "wasm32")]
        return receiver.boxed_local();
    }
}

// fn events_from_bytes(bytes: Byt)
