use futures::{Stream, StreamExt, TryStreamExt};
use moly_widgets::protocol::*;
use serde_json::Value;
use std::sync::Arc;

struct MolyBot {
    avatar: Picture,
}

impl MolyBot {
    fn new() -> Self {
        Self {
            avatar: Picture::Grapheme("M".to_string()),
        }
    }
}

impl Bot for MolyBot {
    fn id(&self) -> BotId {
        BotId::from("moly")
    }

    fn name(&self) -> &str {
        "Moly"
    }

    fn avatar(&self) -> &Picture {
        &self.avatar
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MolyMessage {
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub message: MolyMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Completation {
    pub choices: Vec<Choice>,
}

/// Connect to Moly's already running background server.
struct MolyClient;

impl BotClient for MolyClient {
    fn bots(&self) -> Box<dyn Stream<Item = Result<Arc<dyn Bot>, ()>>> {
        futures::stream::once(async { Ok(Arc::new(MolyBot::new())) })
    }

    fn get_bot(
        &self,
        id: BotId,
    ) -> Box<dyn std::future::Future<Output = Result<Arc<dyn Bot>, ()>>> {
        if id == BotId::from("moly") {
            Box::new(async { Ok(Arc::new(MolyBot::new())) })
        } else {
            Box::new(async { Err(()) })
        }
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(Self)
    }

    fn send(
        &mut self,
        bot: BotId,
        message: &str,
    ) -> Box<dyn std::future::Future<Output = Result<String, ()>>> {
        Box::new(async move {
            // implemented by leveraging the send_stream method

            let a = self.send_stream(bot, message).collect::<Vec<_>>().await;

            todo!()
        })
    }

    fn send_stream(
        &mut self,
        bot: BotId,
        message: &str,
    ) -> Box<dyn Stream<Item = Result<String, ()>>> {
        /*
                ➜  ~ curl http://localhost:50192/v1/chat/completions \
        -H "Content-Type: application/json" \
        -d '{
        "model": "moly",
        "messages": [
        { "role": "system", "content": "Use positive language and offer helpful solutions to their problems." },
        { "role": "user", "content": "What is the currency used in Spain?" }
        ],
        "temperature": 0.7,
        "stream": true
        }'
        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" Spain","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" uses","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" the","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" Spanish","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" P","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":"eso","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" (","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":"ES","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":"P","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":")","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" as","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" its","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" official","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":" currency","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: {"id":"chatcmpl-65724f86-48e0-4c7f-aab7-93b83ad76c62","choices":[{"index":0,"delta":{"content":".","role":"assistant"},"logprobs":null,"finish_reason":null}],"created":1737045585,"model":"moly-chat","system_fingerprint":"fp_44709d6fcb","object":"chat.completion.chunk"}

        data: [DONE]
                 */

        /*

        let mut stream = resp.bytes_stream();

                while let Some(chunk) = tokio::select! {
                    chunk = stream.next() => chunk,
                    _ = cancel.recv() => None,
                } {
                    match chunk {
                        Ok(chunk) => {
                            if chunk.starts_with(b"data: [DONE]") {
                                break;
                            }
                            let resp: Result<ChatResponseChunkData, anyhow::Error> =
                                serde_json::from_slice(&chunk[5..]).map_err(|e| anyhow!(e));
                            let _ = tx.send(resp.map(ChatResponse::ChatResponseChunk));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow!(e)));
                            return;
                        }
                    }
                }

                let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(stop_chunk(
                    StopReason::Stop,
                ))));
         */

        let request = reqwest::Client::new()
            .post("http://localhost:50192/v1/chat/completions")
            .json(&serde_json::json!({
                "model": "moly",
                "messages": [
                    { "role": "system", "content": "Use positive language and offer helpful solutions to their problems." },
                    { "role": "user", "content": message }
                ],
                "temperature": 0.7,
                "stream": true
            }));

        let stream = futures::stream::once(request.send()).flat_map(|response| {
            let Ok(response) = response else {
                return futures::stream::once(async { Err(()) });
            };

            let stream = response.bytes_stream().map(|chunk| {
                let chunk = chunk?;

                if chunk.starts_with(b"data: [DONE]") {
                    return Ok("".to_string());
                }

                let completation: Completation = serde_json::from_slice(&chunk[5..])?;

                completation
                    .choices
                    .iter()
                    .map(|c| c.message)
                    .collect::<String>()
                    .into()
            });
        });

        Box::new(stream)
    }
}
