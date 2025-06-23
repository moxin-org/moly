use async_stream::stream;
use chrono::Utc;
use log::error;
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use crate::utils::{serde::deserialize_null_default, sse::parse_sse};
use crate::{protocol::*, utils::errors::enrich_http_error};

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

/// The content of a [`ContentPart::ImageUrl`].
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageUrlDetail {
    url: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // detail: Option<String>,
}

/// The content of a [`ContentPart::File`].
#[derive(Serialize, Deserialize, Debug, Clone)]
struct File {
    filename: String,
    file_data: String,
}

/// Represents a single part in a multi-part content array of [`Content`].
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrlDetail },
    File { file: File },
}

/// Represents the 'content' field, which can be a string or an array of ContentPart
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)] // Tells Serde to try deserializing into variants without a specific tag
enum Content {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl Default for Content {
    fn default() -> Self {
        Content::Text(String::new())
    }
}

impl Content {
    /// Returns the text content if available, otherwise an empty string.
    pub fn text(&self) -> String {
        match self {
            Content::Text(text) => text.clone(),
            Content::Parts(parts) => parts
                .iter()
                .filter_map(|part| match part {
                    ContentPart::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<String>>()
                .join(" "),
        }
    }
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
#[derive(Clone, Debug, Deserialize)]
struct IncomingMessage {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_null_default")]
    pub content: Content,
    /// The reasoning text, if provided.
    ///
    /// Used by agregators like OpenRouter.
    #[serde(default)]
    pub reasoning: Option<String>,
    /// Wait, another reasoning text? Well, yes.
    /// Some developers take API definitions as suggestions rather than standards.
    ///
    /// Used by providers like Sillicon flow for *some* models.
    #[serde(default)]
    pub reasoning_content: Option<String>,
}

/// A message being sent to the completions endpoint.
#[derive(Clone, Debug, Serialize)]
struct OutcomingMessage {
    pub content: Content,
    pub role: Role,
}

async fn to_outcoming_message(message: Message) -> Result<OutcomingMessage, ()> {
    let role = match message.from {
        EntityId::User => Ok(Role::User),
        EntityId::System => Ok(Role::System),
        EntityId::Bot(_) => Ok(Role::Assistant),
        EntityId::App => Err(()),
    }?;

    let content = if message.content.attachments.is_empty() {
        Content::Text(message.content.text)
    } else {
        let mut parts = Vec::new();
        for attachment in message.content.attachments {
            if !attachment.is_available() {
                makepad_widgets::warning!("Skipping unavailable attachment: {}", attachment.name);
                continue;
            }

            let content = attachment.read_base64().await.map_err(|_| ())?;
            let data_url = format!(
                "data:{};base64,{}",
                attachment
                    .content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
                content
            );

            if attachment.is_image() {
                parts.push(ContentPart::ImageUrl {
                    image_url: ImageUrlDetail { url: data_url },
                });
            } else {
                parts.push(ContentPart::File {
                    file: File {
                        filename: attachment.name,
                        file_data: data_url,
                    },
                });
            }
        }
        parts.push(ContentPart::Text {
            text: message.content.text,
        });
        Content::Parts(parts)
    };

    Ok(OutcomingMessage { content, role })
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
    provider_avatar: Option<Picture>,
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
            provider_avatar: None,
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

    /// Set a custom provider avatar for this client
    pub fn set_provider_avatar(&mut self, avatar: Picture) {
        self.0.write().unwrap().provider_avatar = Some(avatar);
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
                    name: m.id.clone(),
                    avatar: if let Some(avatar) = &inner.provider_avatar {
                        avatar.clone()
                    } else {
                        Picture::Grapheme(m.id.chars().next().unwrap().to_string().to_uppercase())
                    },
                })
                .filter(|b| {
                    // These will be handled by a separate client.
                    !b.id.id().starts_with("dall-e") && !b.id.id().starts_with("gpt-image")
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
    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>> {
        let bot_id = bot_id.clone();
        let messages = messages.to_vec();

        let inner = self.0.read().unwrap().clone();
        let url = format!("{}/chat/completions", inner.url);
        let headers = inner.headers;

        let stream = stream! {
            let mut outgoing_messages: Vec<OutcomingMessage> = Vec::with_capacity(messages.len());
            for message in messages {
                match to_outcoming_message(message.clone()).await {
                    Ok(outgoing_message) => outgoing_messages.push(outgoing_message),
                    Err(_) => {
                        error!("Could not convert message to outgoing format: {:?}", message);
                        yield ClientError::new(
                            ClientErrorKind::Format,
                            "Could not convert message to outgoing format.".into(),
                        ).into();
                        return;
                    }
                }
            }

            let json = serde_json::json!({
                "model": bot_id.id(),
                "messages": outgoing_messages,
                // Note: o1 only supports 1.0, it will error if other value is used.
                // "temperature": 0.7,
                "stream": true
            });


            let request = inner
                .client
                .post(&url)
                .headers(headers)
                .json(&json);

            let response = match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        response
                    } else {
                        let status_code = response.status();
                        if let Err(error) = response.error_for_status() {
                            let original = format!("Request failed: {error}");
                            let enriched = enrich_http_error(status_code, &original);

                            error!("Error sending request to {}: {:?}", url, error);
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
                    error!("Error sending request to {}: {:?}", url, error);
                    yield ClientError::new_with_source(
                        ClientErrorKind::Network,
                        format!("Could not send request to {url}. Verify your connection and the server status."),
                        Some(error),
                    ).into();
                    return;
                }
            };

            let mut content = MessageContent::default();
            let events = parse_sse(response.bytes_stream());

            let mut reasoning_start_time: Option<chrono::DateTime<Utc>> = None;

            for await event in events {
                let event = match event {
                    Ok(event) => event,
                    Err(error) => {
                        error!("Response streaming got interrupted while reading from {}: {:?}", url, error);
                        yield ClientError::new_with_source(
                            ClientErrorKind::Network,
                            format!("Response streaming got interrupted while reading from {url}. This may be a problem with your connection or the server."),
                            Some(error),
                        ).into();
                        return;
                    }
                };

                let completion: Completion = match serde_json::from_str(&event) {
                    Ok(c) => c,
                    Err(error) => {
                        error!("Could not parse the SSE message from {url} as JSON or its structure does not match the expected format. {}", error);
                        yield ClientError::new_with_source(
                            ClientErrorKind::Format,
                            format!("Could not parse the SSE message from {url} as JSON or its structure does not match the expected format."),
                            Some(error),
                        ).into();
                        return;
                    }
                };

                for choice in &completion.choices {
                    // Append main content delta
                    if !choice.delta.content.text().is_empty() {
                        // Main content arrived, we can assume reasoning is done
                        if let Some(start_time) = reasoning_start_time {
                            if let Some(reasoning) = &mut content.reasoning {
                                if reasoning.time_taken_seconds.is_none() {
                                    let end_time = Utc::now();
                                    let time_taken_duration = end_time.signed_duration_since(start_time);
                                    let time_taken = time_taken_duration.num_milliseconds() as f64 / 1000.0;
                                    reasoning.time_taken_seconds = Some(time_taken);
                                }
                            }
                        }
                        content.text.push_str(&choice.delta.content.text());
                    }

                    // Extract reasoning text, could be found in "reasoning" or "reasoning_content"
                    let mut actual_reasoning_delta_text: Option<&str> = None;
                    if let Some(r_text) = &choice.delta.reasoning {
                        if !r_text.is_empty() {
                            if reasoning_start_time.is_none() {
                                reasoning_start_time = Some(Utc::now());
                            }
                            actual_reasoning_delta_text = Some(r_text);
                        }
                    }
                    if actual_reasoning_delta_text.is_none() {
                        if let Some(rc_text) = &choice.delta.reasoning_content {
                            if !rc_text.is_empty() {
                                if reasoning_start_time.is_none() {
                                    reasoning_start_time = Some(Utc::now());
                                }
                                actual_reasoning_delta_text = Some(rc_text);
                            }
                        }
                    }

                    // Append reasoning delta if found
                    if let Some(reasoning_text_to_append) = actual_reasoning_delta_text {
                        if let Some(reasoning) = &mut content.reasoning {
                            reasoning.text.push_str(reasoning_text_to_append);
                        } else {
                            content.reasoning = Some(Reasoning {
                                text: reasoning_text_to_append.to_string(),
                                time_taken_seconds: None,
                            });
                        }
                    }
                }

                for citation in completion.citations {
                    if !content.citations.contains(&citation) {
                        content.citations.push(citation.clone());
                    }
                }

                yield ClientResult::new_ok(content.clone());
            }
        };

        moly_stream(stream)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_client() -> reqwest::Client {
    use std::time::Duration;

    // On native, there are no default timeouts. Connection may hang if we don't
    // configure them.
    reqwest::Client::builder()
        // Only considered while establishing the connection.
        .connect_timeout(Duration::from_secs(90))
        // Considered while reading the response and reset on every chunk
        // received.
        //
        // Warning: Do not use normal `timeout` method as it doesn't consider
        // this.
        .read_timeout(Duration::from_secs(90))
        .build()
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn default_client() -> reqwest::Client {
    // On web, reqwest timeouts are not configurable, but it uses the browser's
    // fetch API under the hood, which handles connection issues properly.
    reqwest::Client::new()
}
