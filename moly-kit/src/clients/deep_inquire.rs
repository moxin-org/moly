use crate::utils::errors::enrich_http_error;
use crate::{protocol::*, utils::sse::parse_sse};
use async_stream::stream;
use makepad_widgets::*;
use makepad_widgets::{Cx, LiveNew, WidgetRef};
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
use widgets::deep_inquire_content::DeepInquireContentWidgetRefExt;

pub(crate) mod widgets;

/// Article reference in a DeepInquire response
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Article {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
    pub relevance: usize,
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
            content: message.content.text,
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
    metadata: Metadata,
    #[serde(default)]
    r#type: String,
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Metadata {
    #[serde(default)]
    stage: String,
}
/// The Choice object in a streaming response
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct DeltaChoice {
    pub delta: DeltaContent,
    index: usize,
    finish_reason: Option<String>,
}

/// Response from the DeepInquire API
#[derive(Clone, Debug, Deserialize)]
struct DeepInquireResponse {
    choices: Vec<DeltaChoice>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stage {
    pub id: String,
    pub citations: Vec<Article>,
    pub substages: Vec<SubStage>,
    pub stage_type: StageType,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default, Live, LiveHook, LiveRead)]
pub enum StageType {
    #[default]
    Thinking,
    #[pick]
    Content,
    Completion,
}

impl FromStr for StageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "thinking" => Ok(StageType::Thinking),
            "content" => Ok(StageType::Content),
            "completion" => Ok(StageType::Completion),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Data {
    stages: Vec<Stage>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SubStage {
    pub id: String,
    pub text: String,
    pub name: String,
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

    pub fn set_header(&mut self, key: &str, value: &str) -> Result<(), &'static str> {
        let header_name = HeaderName::from_str(key).map_err(|_| "Invalid header name")?;

        let header_value = value.parse().map_err(|_| "Invalid header value")?;

        self.0
            .write()
            .unwrap()
            .headers
            .insert(header_name, header_value);

        Ok(())
    }

    pub fn set_key(&mut self, key: &str) -> Result<(), &'static str> {
        self.set_header("Authorization", &format!("Bearer {}", key))
    }
}

impl BotClient for DeepInquireClient {
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
        let inner = self.0.read().unwrap().clone();

        // For now we return a hardcoded bot because DeepInquire does not support a /models endpoint
        let bot = Bot {
            id: BotId::new("DeepInquire", &inner.url),
            name: "DeepInquire".to_string(),
            avatar: Picture::Grapheme("D".into()),
        };

        let future = async move { ClientResult::new_ok(vec![bot]) };

        moly_future(future)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>> {
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
                "model": bot_id.id(),
                "messages": moly_messages,
                "stream": true
            }));

        let stream = stream! {
            let response = match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        response
                    } else {
                        let status_code = response.status();
                        if let Err(error) = response.error_for_status() {
                            let original = format!("Request failed: {error}");
                            let enriched = enrich_http_error(status_code, &original);

                            yield ClientError::new_with_source(
                                ClientErrorKind::Remote,
                                enriched,
                                Some(error),
                            ).into();
                        }
                        return;
                    }
                }
                Err(error) => {
                    ::log::error!("SSE stream unexpectedly interrupted while reading from {}: {:?}", url, error);
                    yield ClientError::new_with_source(
                        ClientErrorKind::Network,
                        format!("The connection was unexpectedly closed while streaming the response from {url}. This could be due to network issues, server problems, or timeouts."),
                        Some(error),
                    ).into();
                    return;
                }
            };

            let events = parse_sse(response.bytes_stream());
            let mut content = MessageContent::default();
            let mut consecutive_timeouts = 0;
            let max_consecutive_timeouts = 3;
            let mut message_count = 0;
            // Only yield to UI every 10 messages to reduce back-pressure
            let yield_frequency = 10;

            for await event in events {
                let event = match event {
                    Ok(chunk) => {
                        consecutive_timeouts = 0; // Reset timeout counter on success
                        message_count += 1;
                        chunk
                    },
                    Err(error) => {
                        if error.is_timeout() {
                            consecutive_timeouts += 1;

                            if consecutive_timeouts >= max_consecutive_timeouts {
                                ::log::error!("SSE stream error while reading from {}: {:?}", url, error);
                                yield ClientError::new_with_source(
                                    ClientErrorKind::Network,
                                    format!("Too many consecutive timeouts ({}) while reading from {url}. Giving up.", consecutive_timeouts),
                                    Some(error),
                                ).into();
                                return;
                            }

                            continue;
                        } else {
                            ::log::error!("SSE stream error while reading from {}: {:?}", url, error);
                            yield ClientError::new_with_source(
                                ClientErrorKind::Network,
                                format!("The connection was unexpectedly closed while streaming the response from {url}. This could be due to network issues, server problems, or timeouts."),
                                Some(error),
                            ).into();
                            return;
                        }
                    }
                };

                let response: DeepInquireResponse = match serde_json::from_str(&event) {
                    Ok(c) => c,
                    Err(error) => {
                        ::log::error!("Could not parse the SSE message from {url} as JSON or its structure does not match the expected format. {}", error);
                        yield ClientError::new_with_source(
                            ClientErrorKind::Format,
                            format!("Could not parse the SSE message from {url} as JSON or its structure does not match the expected format."),
                            Some(error),
                        ).into();
                        return;
                    }
                };

                apply_response_to_content(response, &mut content);

                // Only yield to UI periodically to reduce back-pressure
                // The first 20 messages are yielded immediately to ensure the UI is updated
                if message_count % yield_frequency == 0 || message_count < 20 {
                    yield ClientResult::new_ok(content.clone());
                }
            }

            // Final yield to ensure the last state is captured
            yield ClientResult::new_ok(content.clone());
        };

        moly_stream(stream)
    }

    fn content_widget(
        &mut self,
        cx: &mut Cx,
        previous_widget: WidgetRef,
        templates: &HashMap<LiveId, LivePtr>,
        content: &MessageContent,
    ) -> Option<WidgetRef> {
        let Some(template) = templates.get(&live_id!(DeepInquireContent)).copied() else {
            return None;
        };

        let Some(data) = content.data.as_deref() else {
            return None;
        };

        let Ok(_) = serde_json::from_str::<Data>(data) else {
            return None;
        };

        let widget = if previous_widget.as_deep_inquire_content().borrow().is_some() {
            previous_widget
        } else {
            WidgetRef::new_from_ptr(cx, Some(template))
        };

        widget
            .as_deep_inquire_content()
            .borrow_mut()
            .unwrap()
            .set_content(cx, content);

        Some(widget)
    }
}

fn apply_response_to_content(response: DeepInquireResponse, content: &mut MessageContent) {
    for choice in response.choices {
        let delta = choice.delta;

        // The id from the server follows the format <substage_id>.<stream_chunk_id>, which is not useful for tracking
        // the substages. Thefore we use the metadata 'stage' (a name for the substage) as the id to use for keeping track and upadting each substage.
        // For the stage_id we use the id from the delta without the stream_chunk_id.
        let stage_id = delta.id.split('.').next().unwrap_or(&delta.id).to_string();
        let substage_id = delta.metadata.stage.clone();
        let stage_type = StageType::from_str(&delta.r#type).unwrap(); // TODO: Handle this gracefully

        create_or_update_stage(content, stage_type, stage_id, move |existing_stage| {
            // Check if the substage arriving in the response is already present in the accumulated content
            let existing_substage = existing_stage
                .substages
                .iter_mut()
                .find(|s| s.name == delta.metadata.stage);
            if let Some(existing_substage) = existing_substage {
                // SubStage exists, apply delta
                existing_substage.text.push_str(&delta.content);
            } else {
                // SubStage does not exist, create a new one
                existing_stage.substages.push(SubStage {
                    id: substage_id,
                    name: delta.metadata.stage,
                    text: delta.content,
                })
            }

            // Citations are at the Stage level, extend without duplicating
            let new_citations: Vec<_> = delta
                .articles
                .into_iter()
                .filter(|citation| !existing_stage.citations.contains(citation))
                .collect();

            existing_stage.citations.extend(new_citations);

            return;
        });
    }
}

fn create_or_update_stage(
    content: &mut MessageContent,
    stage_type: StageType,
    stage_id: String,
    update_fn: impl FnOnce(&mut Stage),
) {
    let mut data: Data = content
        .data
        .as_ref()
        .and_then(|d| serde_json::from_str(d).ok())
        .unwrap_or_default();

    // Find the existing stage by matching the enum variant
    if let Some(mut existing_stage) = data.stages.iter_mut().find(|s| s.stage_type == stage_type) {
        update_fn(&mut existing_stage);
    } else {
        let mut new_stage = Stage {
            id: stage_id,
            substages: vec![],
            citations: vec![],
            stage_type,
        };

        update_fn(&mut new_stage);
        data.stages.push(new_stage);
    }

    content.data = Some(serde_json::to_string(&data).unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
fn default_client() -> reqwest::Client {
    use std::time::Duration;

    reqwest::Client::builder()
        // Only considered while establishing the connection
        .connect_timeout(Duration::from_secs(360))
        // Keep high read timeout for word-by-word streaming
        .read_timeout(Duration::from_secs(360))
        .build()
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn default_client() -> reqwest::Client {
    // On web, reqwest timeouts are not configurable, but it uses the browser's
    // fetch API under the hood, which handles connection issues properly.
    reqwest::Client::new()
}
