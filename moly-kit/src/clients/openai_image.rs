//! Client based on the OpenAI one, but hits the image generation API instead.

use crate::protocol::Tool;
use crate::protocol::*;
use crate::utils::asynchronous::{BoxPlatformSendFuture, BoxPlatformSendStream};
use reqwest::header::{HeaderMap, HeaderName};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
struct OpenAIImageClientInner {
    url: String,
    client: reqwest::Client,
    headers: HeaderMap,
}

/// Specific OpenAI client to hit image generation endpoints.
///
/// If used as part of a [`crate::clients::MultiClient`], it's recommended to add this
/// before the standard OpenAI client to ensure it get's priority. This is not strictly
/// necessary if the OpenAI client recognizes and filters the image models you use.
#[derive(Debug)]
pub struct OpenAIImageClient(Arc<RwLock<OpenAIImageClientInner>>);

impl Clone for OpenAIImageClient {
    fn clone(&self) -> Self {
        OpenAIImageClient(Arc::clone(&self.0))
    }
}

impl OpenAIImageClient {
    pub fn new(url: String) -> Self {
        let headers = HeaderMap::new();
        let client = default_client();

        let inner = OpenAIImageClientInner {
            url,
            client,
            headers,
        };

        OpenAIImageClient(Arc::new(RwLock::new(inner)))
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

    pub fn get_url(&self) -> String {
        self.0.read().unwrap().url.clone()
    }

    async fn generate_image(
        &self,
        bot_id: &BotId,
        messages: &[Message],
    ) -> Result<MessageContent, ClientError> {
        let inner = self.0.read().unwrap().clone();

        let prompt = messages
            .last()
            .map(|msg| msg.content.text.as_str())
            .ok_or_else(|| {
                ClientError::new(ClientErrorKind::Unknown, "No messages provided".to_string())
            })?;

        let url = format!("{}/images/generations", inner.url);

        let request_json = serde_json::json!({
            "model": bot_id.id(),
            "prompt": prompt,
            // "auto" is supported by `gpt-image` but not for `dall-e`.
            "size": "1024x1024",
            // `gpt-image` always returns base64, but `dall-e` supports
            // and defaults to `url` response format.
            "response_format": "b64_json"
        });

        let request = inner
            .client
            .post(&url)
            .headers(inner.headers.clone())
            .json(&request_json);

        let response = request.send().await.map_err(|e| {
            ClientError::new_with_source(
                ClientErrorKind::Network,
                format!(
                    "Could not send request to {url}. Verify your connection and the server status."
                ),
                Some(e),
            )
        })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(ClientError::new(
                ClientErrorKind::Response,
                format!(
                    "Request to {url} failed with status {} and content: {}",
                    status, text
                ),
            ));
        }

        let response_json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            ClientError::new_with_source(
                ClientErrorKind::Format,
                format!(
                    "Failed to parse response from {url}. It does not match the expected format."
                ),
                Some(e),
            )
        })?;

        let image_data = response_json
            .get("data")
            .and_then(|data| data.get(0))
            .and_then(|item| item.get("b64_json"))
            .and_then(|b64| b64.as_str())
            .ok_or_else(|| {
                ClientError::new(
                    ClientErrorKind::Format,
                    "Response does not contain expected 'b64_json' field".to_string(),
                )
            })?;

        let attachment =
            Attachment::from_base64("image.png".into(), Some("image/png".into()), image_data)
                .map_err(|e| {
                    ClientError::new_with_source(
                        ClientErrorKind::Format,
                        "Failed to create attachment from base64 data".to_string(),
                        Some(e),
                    )
                })?;

        let content = MessageContent {
            text: String::new(),
            attachments: vec![attachment],
            ..Default::default()
        };

        Ok(content)
    }
}

impl BotClient for OpenAIImageClient {
    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        let inner = self.0.read().unwrap().clone();

        // Hardcoded list of OpenAI-only image generation models that are currently
        // available and supported.
        let supported: Vec<Bot> = ["dall-e-2", "dall-e-3", "gpt-image-1", "gpt-image-1-mini"] 
            .into_iter()
            .map(|id| Bot {
                id: BotId::new(id, &inner.url),
                name: id.to_string(),
                avatar: Picture::Grapheme("I".into()),
                capabilities: BotCapabilities::new(),
            })
            .collect();

        Box::pin(futures::future::ready(ClientResult::new_ok(supported)))
    }

    fn send(
        &mut self,
        bot_id: &BotId,
        messages: &[Message],
        _tools: &[Tool],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let self_clone = self.clone();
        let bot_id = bot_id.clone();
        let messages = messages.to_vec();

        Box::pin(async_stream::stream! {
            match self_clone.generate_image(&bot_id, &messages).await {
                Ok(content) => yield ClientResult::new_ok(content),
                Err(e) => yield ClientResult::new_err(e.into()),
            }
        })
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }
}

// TODO: Dedup from other clients.
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
