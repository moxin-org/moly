use futures::{future::FutureExt, stream::StreamExt, SinkExt};
use makepad_widgets::log;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::{protocol::*, utils::asynchronous::spawn};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Model {
    id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Models {
    pub data: Vec<Model>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MolyMessage {
    pub content: String,
    pub role: Role,
}

impl TryFrom<Message> for MolyMessage {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub delta: MolyMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Completation {
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, Default)]
struct MolyServiceInner {
    url: String,
    key: Option<String>,
}

#[derive(Debug)]
pub struct MolyService(Arc<Mutex<MolyServiceInner>>);

impl Clone for MolyService {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<MolyServiceInner> for MolyService {
    fn from(inner: MolyServiceInner) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }
}

impl MolyService {
    pub fn new(url: String, key: Option<String>) -> Self {
        MolyServiceInner {
            url,
            key,
            ..Default::default()
        }
        .into()
    }
}

impl BotService for MolyService {
    fn bots(&self) -> BoxStream<Result<Bot, ()>> {
        let (mut sender, receiver) = futures::channel::mpsc::channel(0);

        let request =
            reqwest::Client::new().get(format!("{}/v1/models", self.0.lock().unwrap().url));

        spawn(async move {
            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    log!("Error {:?}", error);
                    sender.send(Err(())).await.unwrap();
                    return;
                }
            };

            let models: Models = match response.json().await {
                Ok(models) => models,
                Err(error) => {
                    log!("Error {:?}", error);
                    sender.send(Err(())).await.unwrap();
                    return;
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

            for bot in bots {
                sender.send(Ok(bot)).await.unwrap();
            }
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            receiver.boxed()
        }

        #[cfg(target_arch = "wasm32")]
        receiver.boxed_local()
    }

    fn clone_box(&self) -> Box<dyn BotService> {
        // ref should be shared but since hardcoded it should be ok
        Box::new(self.clone())
    }

    fn send_stream(&mut self, _bot: BotId, messages: &[Message]) -> BoxStream<Result<String, ()>> {
        let mut moly_messages: Vec<MolyMessage> = Vec::new();

        if !messages.iter().any(|m| m.from == EntityId::System) {
            moly_messages.push(MolyMessage {
                content: "You're a helpful assistant. You can speak English (default), Spanish and Chinese.".to_string(),
                role: Role::System,
            });
        }

        moly_messages.extend(messages.iter().filter_map(|m| m.clone().try_into().ok()));

        let request = reqwest::Client::new()
            .post(format!(
                "{}/v1/chat/completions",
                self.0.lock().unwrap().url
            ))
            .json(&serde_json::json!({
                "model": "moly",
                "messages": moly_messages,
                "temperature": 0.7,
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

    fn send(&mut self, bot: BotId, messages: &[Message]) -> BoxFuture<Result<String, ()>> {
        let stream = self.send_stream(bot, messages);

        let future = async move {
            let parts = stream.collect::<Vec<_>>().await;

            if parts.contains(&Err(())) {
                return Err(());
            }

            let message = parts.into_iter().filter_map(Result::ok).collect::<String>();
            Ok(message)
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            future.boxed()
        }

        #[cfg(target_arch = "wasm32")]
        future.boxed_local()
    }
}

// fn events_from_bytes(bytes: Byt)
