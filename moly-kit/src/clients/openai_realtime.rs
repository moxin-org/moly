use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::protocol::*;
use crate::utils::asynchronous::{BoxPlatformSendFuture, BoxPlatformSendStream, spawn};
use futures::StreamExt;

// Realtime enabled + not wasm
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
use {futures::SinkExt, tokio_tungstenite::tungstenite::Message as WsMessage};

// OpenAI Realtime API message structures
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OpenAIRealtimeMessage {
    #[serde(rename = "session.update")]
    SessionUpdate { session: SessionConfig },
    #[serde(rename = "input_audio_buffer.append")]
    InputAudioBufferAppend {
        audio: String, // base64 encoded audio
    },
    #[serde(rename = "input_audio_buffer.commit")]
    InputAudioBufferCommit,
    #[serde(rename = "response.create")]
    ResponseCreate { response: ResponseConfig },
    #[serde(rename = "conversation.item.create")]
    ConversationItemCreate { item: ConversationItem },
    #[serde(rename = "conversation.item.truncate")]
    ConversationItemTruncate {
        item_id: String,
        content_index: u32,
        audio_end_ms: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionConfig {
    pub modalities: Vec<String>,
    pub instructions: String,
    pub voice: String,
    pub model: String,
    pub input_audio_format: String,
    pub output_audio_format: String,
    pub input_audio_transcription: Option<TranscriptionConfig>,
    pub input_audio_noise_reduction: Option<NoiseReductionConfig>,
    pub turn_detection: Option<TurnDetectionConfig>,
    pub tools: Vec<serde_json::Value>,
    pub tool_choice: String,
    pub temperature: f32,
    pub max_response_output_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TranscriptionConfig {
    pub model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoiseReductionConfig {
    #[serde(rename = "type")]
    pub noise_reduction_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TurnDetectionConfig {
    #[serde(rename = "type")]
    pub detection_type: String,
    pub threshold: f32,
    pub prefix_padding_ms: u32,
    pub silence_duration_ms: u32,
    pub interrupt_response: bool,
    pub create_response: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseConfig {
    pub modalities: Vec<String>,
    pub instructions: Option<String>,
    pub voice: Option<String>,
    pub output_audio_format: Option<String>,
    pub tools: Vec<serde_json::Value>,
    pub tool_choice: String,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConversationItem {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub status: Option<String>,
    pub role: String,
    pub content: Vec<ContentPart>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "input_text")]
    InputText { text: String },
    #[serde(rename = "input_audio")]
    InputAudio {
        audio: String,
        transcript: Option<String>,
    },
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "audio")]
    Audio {
        audio: String,
        transcript: Option<String>,
    },
}

// Incoming message types from OpenAI
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OpenAIRealtimeResponse {
    #[serde(rename = "error")]
    Error { error: ErrorDetails },
    #[serde(rename = "session.created")]
    SessionCreated { session: serde_json::Value },
    #[serde(rename = "session.updated")]
    SessionUpdated { session: serde_json::Value },
    #[serde(rename = "conversation.item.created")]
    ConversationItemCreated { item: serde_json::Value },
    #[serde(rename = "conversation.item.truncated")]
    ConversationItemTruncated { item: serde_json::Value },
    #[serde(rename = "response.audio.delta")]
    ResponseAudioDelta {
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String, // base64 encoded audio
    },
    #[serde(rename = "response.audio.done")]
    ResponseAudioDone {
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
    },
    #[serde(rename = "response.text.delta")]
    ResponseTextDelta {
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },
    #[serde(rename = "response.audio_transcript.delta")]
    ResponseAudioTranscriptDelta {
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },
    #[serde(rename = "response.audio_transcript.done")]
    ResponseAudioTranscriptDone {
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        transcript: String,
    },
    #[serde(rename = "conversation.item.input_audio_transcription.completed")]
    ConversationItemInputAudioTranscriptionCompleted {
        item_id: String,
        content_index: u32,
        transcript: String,
    },
    #[serde(rename = "response.done")]
    ResponseDone { response: serde_json::Value },
    #[serde(rename = "input_audio_buffer.speech_started")]
    InputAudioBufferSpeechStarted {
        audio_start_ms: u32,
        item_id: String,
    },
    #[serde(rename = "input_audio_buffer.speech_stopped")]
    InputAudioBufferSpeechStopped { audio_end_ms: u32, item_id: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
pub struct ErrorDetails {
    pub code: Option<String>,
    pub message: String,
    pub param: Option<String>,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

// Use the protocol definitions
pub use crate::protocol::{RealtimeChannel, RealtimeCommand, RealtimeEvent};

#[derive(Clone, Debug)]
pub struct OpenAIRealtimeClient {
    address: String,
    api_key: Option<String>,
}

impl OpenAIRealtimeClient {
    pub fn new(address: String) -> Self {
        Self {
            address,
            api_key: None,
        }
    }

    pub fn set_key(&mut self, api_key: &str) -> Result<(), String> {
        self.api_key = Some(api_key.to_string());
        Ok(())
    }

    pub fn create_realtime_session(
        &self,
        bot_id: &BotId,
    ) -> BoxPlatformSendFuture<'static, ClientResult<RealtimeChannel>> {
        let address = self.address.clone();
        let api_key = self.api_key.clone().expect("No API key provided");

        let bot_id = bot_id.clone();
        let future = async move {
            let (event_sender, event_receiver) = futures::channel::mpsc::unbounded();
            let (command_sender, mut command_receiver) = futures::channel::mpsc::unbounded();

            #[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
            {
                // Create WebSocket connection to OpenAI Realtime API
                // If the provider is OpenAI, include the model to the url
                let url_str = if address.starts_with("wss://api.openai.com") {
                    format!("{}?model={}", address, bot_id.id())
                } else {
                    address
                };

                // Use connect_async_with_config for proper header handling
                use tokio_tungstenite::tungstenite::handshake::client::Request;

                // We need to setup all of these headers manually if we want to setup our Auhtorization header.
                let request = Request::builder()
                    .uri(&url_str)
                    .header("Host", "api.openai.com")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Version", "13")
                    .header("OpenAI-Beta", "realtime=v1")
                    .header(
                        "Sec-WebSocket-Key",
                        tokio_tungstenite::tungstenite::handshake::client::generate_key(),
                    )
                    .body(())
                    .unwrap();

                let (ws_stream, _) = match tokio_tungstenite::connect_async(request).await {
                    Ok(result) => result,
                    Err(e) => {
                        log::error!("Error connecting to OpenAI Realtime API: {}", e);
                        return ClientResult::new_err(vec![ClientError::new_with_source(
                            ClientErrorKind::Network,
                            "Failed to connect to OpenAI Realtime API".to_string(),
                            Some(e),
                        )]);
                    }
                };

                let (mut write, mut read) = ws_stream.split();
                log::debug!("WebSocket connection created");

                // Spawn task to handle incoming messages
                let event_sender_clone = event_sender.clone();
                spawn(async move {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(WsMessage::Text(text)) => {
                                log::debug!("Received WebSocket message: {}", text);
                                // log::info!("Received WebSocket message: {}", text);
                                if let Ok(response) =
                                    serde_json::from_str::<OpenAIRealtimeResponse>(&text)
                                {
                                    let event = match response {
                                        OpenAIRealtimeResponse::SessionCreated { .. } => {
                                            Some(RealtimeEvent::SessionReady)
                                        }
                                        OpenAIRealtimeResponse::ResponseAudioDelta {
                                            delta,
                                            ..
                                        } => {
                                            if let Ok(audio_bytes) =
                                                general_purpose::STANDARD.decode(&delta)
                                            {
                                                Some(RealtimeEvent::AudioData(audio_bytes))
                                            } else {
                                                None
                                            }
                                        }
                                        OpenAIRealtimeResponse::ResponseAudioTranscriptDelta {
                                            delta,
                                            ..
                                        } => Some(RealtimeEvent::AudioTranscript(delta)),
                                        OpenAIRealtimeResponse::ResponseAudioTranscriptDone {
                                            transcript,
                                            item_id,
                                            ..
                                        } => Some(RealtimeEvent::AudioTranscriptCompleted(transcript, item_id)),
                                        OpenAIRealtimeResponse::ConversationItemInputAudioTranscriptionCompleted {
                                            transcript,
                                            item_id,
                                            ..
                                        } => Some(RealtimeEvent::UserTranscriptCompleted(transcript, item_id)),
                                        OpenAIRealtimeResponse::InputAudioBufferSpeechStarted {
                                            ..
                                        } => Some(RealtimeEvent::SpeechStarted),
                                        OpenAIRealtimeResponse::InputAudioBufferSpeechStopped {
                                            ..
                                        } => Some(RealtimeEvent::SpeechStopped),
                                        OpenAIRealtimeResponse::ResponseDone { .. } => {
                                            Some(RealtimeEvent::ResponseCompleted)
                                        }
                                        OpenAIRealtimeResponse::Error { error } => {
                                            Some(RealtimeEvent::Error(error.message))
                                        }
                                        _ => None,
                                    };

                                    if let Some(event) = event {
                                        let _ = event_sender_clone.unbounded_send(event);
                                    }
                                }
                            }
                            Ok(WsMessage::Close(_)) => {
                                log::info!("WebSocket closed");
                                break;
                            }
                            Err(e) => {
                                log::error!("WebSocket error: {}", e);
                                let _ = event_sender_clone
                                    .unbounded_send(RealtimeEvent::Error(e.to_string()));
                                break;
                            }
                            _ => {}
                        }
                    }
                });

                // Spawn task to handle outgoing commands
                spawn(async move {
                    let model = bot_id.id().to_string();
                    // Handle commands
                    while let Some(command) = command_receiver.next().await {
                        match command {
                            RealtimeCommand::UpdateSessionConfig {
                                voice,
                                transcription_model,
                            } => {
                                log::debug!(
                                    "Updating session config with voice: {}, transcription: {}",
                                    voice,
                                    transcription_model
                                );
                                let session_config = SessionConfig {
                                    modalities: vec!["text".to_string(), "audio".to_string()],
                                    instructions: "You are a helpful AI assistant. Respond naturally and conversationally. Always respond in the same language as the user.".to_string(),
                                    voice: voice.clone(),
                                    model: model.clone(),
                                    input_audio_format: "pcm16".to_string(),
                                    output_audio_format: "pcm16".to_string(),
                                    input_audio_transcription: Some(TranscriptionConfig {
                                        model: transcription_model,
                                    }),
                                    input_audio_noise_reduction: Some(NoiseReductionConfig {
                                        noise_reduction_type: "far_field".to_string(),
                                    }),
                                    turn_detection: Some(TurnDetectionConfig {
                                        detection_type: "server_vad".to_string(),
                                        threshold: 0.5,
                                        prefix_padding_ms: 300,
                                        silence_duration_ms: 200,
                                        interrupt_response: true,
                                        create_response: true,
                                    }),
                                    tools: vec![],
                                    tool_choice: "none".to_string(),
                                    temperature: 0.8,
                                    max_response_output_tokens: Some(4096),
                                };

                                let session_message = OpenAIRealtimeMessage::SessionUpdate {
                                    session: session_config,
                                };

                                if let Ok(json) = serde_json::to_string(&session_message) {
                                    log::debug!("Sending session update: {}", json);
                                    let _ = write.send(WsMessage::Text(json)).await;
                                }
                            }
                            RealtimeCommand::CreateGreetingResponse => {
                                log::debug!("Creating AI greeting response");
                                let response_config = ResponseConfig {
                                    modalities: vec!["text".to_string(), "audio".to_string()],
                                    instructions: Some("You are a helpful AI assistant. Respond naturally and conversationally, start with a very short but enthusiastic and playful greeting in English, the greeting must not exceed 3 words".to_string()),
                                    voice: None,
                                    output_audio_format: Some("pcm16".to_string()),
                                    tools: vec![],
                                    tool_choice: "none".to_string(),
                                    temperature: Some(0.8),
                                    max_output_tokens: Some(4096),
                                };

                                let message = OpenAIRealtimeMessage::ResponseCreate {
                                    response: response_config,
                                };

                                if let Ok(json) = serde_json::to_string(&message) {
                                    log::debug!("Sending greeting response: {}", json);
                                    let _ = write.send(WsMessage::Text(json)).await;
                                }
                            }
                            RealtimeCommand::SendAudio(audio_data) => {
                                let base64_audio = general_purpose::STANDARD.encode(&audio_data);
                                let message = OpenAIRealtimeMessage::InputAudioBufferAppend {
                                    audio: base64_audio,
                                };
                                if let Ok(json) = serde_json::to_string(&message) {
                                    log::debug!("Sending audio data: {}", json);
                                    let _ = write.send(WsMessage::Text(json)).await;
                                }
                            }
                            RealtimeCommand::SendText(text) => {
                                let item = ConversationItem {
                                    id: None,
                                    item_type: "message".to_string(),
                                    status: None,
                                    role: "user".to_string(),
                                    content: vec![ContentPart::InputText { text }],
                                };
                                let message =
                                    OpenAIRealtimeMessage::ConversationItemCreate { item };
                                if let Ok(json) = serde_json::to_string(&message) {
                                    log::debug!("Sending text message: {}", json);
                                    let _ = write.send(WsMessage::Text(json)).await;
                                }
                            }
                            RealtimeCommand::Interrupt => {
                                // Send truncate message to interrupt current response
                                let message = OpenAIRealtimeMessage::InputAudioBufferCommit;
                                if let Ok(json) = serde_json::to_string(&message) {
                                    // log::info!("Sending truncate message: {}", json);
                                    log::debug!("Sending truncate message: {}", json);
                                    let _ = write.send(WsMessage::Text(json)).await;
                                }
                            }
                            RealtimeCommand::StopSession => {
                                // Close the WebSocket connection
                                let _ = write.send(WsMessage::Close(None)).await;
                                break;
                            }
                        }
                    }
                });
            }

            #[cfg(not(all(feature = "realtime", not(target_arch = "wasm32"))))]
            {
                // Fallback mock implementation when websocket feature is not enabled or on WASM
                let mut event_sender_clone = event_sender.clone();
                spawn(async move {
                    let _ = event_sender_clone.unbounded_send(RealtimeEvent::Error(
                        "Realtime feature not available on this platform".to_string(),
                    ));
                });
            }

            ClientResult::new_ok(RealtimeChannel {
                event_sender,
                event_receiver: Arc::new(Mutex::new(Some(event_receiver))),
                command_sender,
            })
        };

        Box::pin(future)
    }
}

impl BotClient for OpenAIRealtimeClient {
    fn send(
        &mut self,
        bot_id: &BotId,
        _messages: &[crate::protocol::Message],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        // For realtime, we create a session and return the upgrade in the message content
        let future = self.create_realtime_session(bot_id);

        let stream = async_stream::stream! {
            match future.await.into_result() {
                Ok(channel) => {
                    // Return a message with the realtime upgrade
                    let content = MessageContent {
                        text: "Realtime session established. Starting voice conversation...".to_string(),
                        upgrade: Some(Upgrade::Realtime(channel)),
                        ..Default::default()
                    };
                    yield ClientResult::new_ok(content);
                }
                Err(errors) => {
                    // Return error message
                    let error_msg = errors.first().map(|e| e.to_string()).unwrap_or_default();
                    let content = MessageContent {
                        text: format!("Failed to establish realtime session: {}", error_msg),
                        ..Default::default()
                    };
                    yield ClientResult::new_ok(content);
                }
            }
        };

        Box::pin(stream)
    }

    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        // Hardcoded list of OpenAI-only realtime models that are currently
        // available and supported.
        let supported: Vec<Bot> = ["gpt-4o-realtime-preview-2025-06-03"]
            .into_iter()
            .map(|id| Bot {
                id: BotId::new(id, &self.address),
                name: id.to_string(),
                avatar: Picture::Grapheme("🎤".into()),
                capabilities: BotCapabilities::new().with_capability(BotCapability::Realtime),
            })
            .collect();

        Box::pin(futures::future::ready(ClientResult::new_ok(supported)))
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }
}
