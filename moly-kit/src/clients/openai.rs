use async_stream::stream;
use makepad_widgets::*;
use reqwest::header::{HeaderMap, HeaderName};
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use crate::utils::asynchronous::{BoxPlatformSendFuture, BoxPlatformSendStream};
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

#[derive(Serialize)]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    strict: Option<bool>,
}

#[derive(Serialize)]
struct FunctionTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDefinition,
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
    /// The reasoning text separated from the main content if provided.
    /// - Aggregators like OpenRouter may expose this as `reasoning`.
    /// - Other providers like Silicon Flow may use `reasoning_content` instead
    ///   for **some** models.
    /// - Local distilled DeepSeek R1 models may NOT use this, and instead return
    ///   reasoning as part of the `content` under a `<think>` tag.
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_null_default")]
    #[serde(alias = "reasoning_content")]
    pub reasoning: String,
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

impl BotClient for OpenAIClient {
    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
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
                    ClientErrorKind::Response,
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
                    avatar: Picture::Grapheme(
                        m.id.chars().next().unwrap().to_string().to_uppercase(),
                    ),
                    // TODO: base this on the provider + model combo
                    // E.g. gpt-4o might support attachments directly, but not through an aggregator like OpenRouter.
                    capabilities: BotCapabilities::new()
                        .with_capability(BotCapability::Attachments),
                })
                .filter(|b| {
                    // These will be handled by a separate client.
                    !b.id.id().starts_with("dall-e") && !b.id.id().starts_with("gpt-image")
                })
                .collect();

            bots.sort_by(|a, b| a.name.cmp(&b.name));

            ClientResult::new_ok(bots)
        };

        Box::pin(future)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }

    /// Stream pieces of content back as a ChatDelta instead of just a String.
    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
        tools: &[Tool],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let bot_id = bot_id.clone();
        let messages = messages.to_vec();

        let inner = self.0.read().unwrap().clone();
        let url = format!("{}/chat/completions", inner.url);
        let headers = inner.headers;

        let tools = tools.iter().map(|t| {
            // Use the input_schema from the MCP tool, but ensure OpenAI compatibility
            let mut parameters_map = (*t.input_schema).clone();
            
            // Ensure additionalProperties is set to false as required by OpenAI
            parameters_map.insert("additionalProperties".to_string(), serde_json::Value::Bool(false));
            
            // Ensure properties field exists for object schemas (OpenAI requirement)
            if parameters_map.get("type") == Some(&serde_json::Value::String("object".to_string())) {
                if !parameters_map.contains_key("properties") {
                    parameters_map.insert("properties".to_string(), serde_json::Value::Object(serde_json::Map::new()));
                }
            }
            
            let parameters = serde_json::Value::Object(parameters_map);
            
            FunctionTool{
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: t.name.to_string(),
                    description: t.description.as_deref().unwrap_or("").to_string(),
                    parameters,
                    strict: Some(true),
                },
            }
        }).collect::<Vec<_>>();

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
                "tools": tools,
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
                        let body = response.text().await.unwrap();
                        let original = format!("Request failed with status {}", status_code);
                        let enriched = enrich_http_error(status_code, &original, Some(&body));

                        error!("Error sending request to {}: status {}", url, status_code);
                        yield ClientError::new(
                            ClientErrorKind::Response,
                            enriched,
                        ).into();
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
            let mut full_text = String::default();
            let events = parse_sse(response.bytes_stream());

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

                // Aggregate deltas
                for choice in &completion.choices {
                    // Keep track of the full content as it came, without modifications.
                    full_text.push_str(&choice.delta.content.text());

                    // Extract the inlined reasoning if any.
                    let (reasoning, text) = split_reasoning_tag(&full_text);

                    // Set the content text without any reasoning.
                    content.text = text.to_string();

                    if reasoning.is_empty() {
                        // Append reasoning delta if reasoning was not part of the content.
                        content.reasoning.push_str(&choice.delta.reasoning);
                    } else {
                        // Otherwise, set the reasoning to what we extracted from the full text.
                        content.reasoning = reasoning.to_string();
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

        Box::pin(stream)
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

/// If a string starts with a `<think>` tag, split the content from the rest of the text.
/// - This happens in order, so first element of the tuple is the reasoning.
/// - If the tag is unclosed, everything goes to reasoning.
/// - If there is no tag, everything goes to the second element of the tuple.
fn split_reasoning_tag(text: &str) -> (&str, &str) {
    const START_TAG: &str = "<think>";
    const END_TAG: &str = "</think>";

    if let Some(text) = text.trim_start().strip_prefix(START_TAG) {
        text.split_once(END_TAG).unwrap_or((text, ""))
    } else {
        ("", text)
    }
}
