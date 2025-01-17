use async_stream::stream;

use futures::{
    future::{BoxFuture, FutureExt},
    stream::{BoxStream, StreamExt},
    Stream,
};
use moly_widgets::protocol::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MolyMessage {
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub delta: MolyMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Completation {
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug)]
pub struct MolyRepo {
    bots: Vec<Bot>,
    pub port: u16,
}

impl Default for MolyRepo {
    fn default() -> Self {
        Self {
            bots: vec![Bot {
                id: BotId::from("moly"),
                name: "Moly".to_string(),
                avatar: Picture::Grapheme("M".to_string()),
            }],
            port: 0,
        }
    }
}

impl BotRepo for MolyRepo {
    fn get_bot(&self, id: BotId) -> Option<Bot> {
        self.bots.iter().find(|bot| bot.id == id).cloned()
    }

    fn bots(&self) -> Box<dyn Iterator<Item = Bot>> {
        Box::new(self.bots.clone().into_iter())
    }

    fn clone_box(&self) -> Box<dyn BotRepo> {
        // ref should be shared but since hardcoded it should be ok
        Box::new(self.clone())
    }

    fn send_stream(&mut self, _bot: BotId, message: &str) -> BoxStream<Result<String, ()>> {
        let request = reqwest::Client::new()
        .post(format!("http://localhost:{}/v1/chat/completions", self.port))
        .json(&serde_json::json!({
            "model": "moly",
            "messages": [
                { "role": "system", "content": "Use positive language and offer helpful solutions to their problems." },
                { "role": "user", "content": message }
            ],
            "temperature": 0.7,
            "stream": true
        }));

        let stream = stream! {
            let response = request.send().await;

            let Ok(response) = response else {
                eprintln!("Error: {:?}", response);
                yield Err(());
                return;
            };

            for await value in response.bytes_stream() {
                let chunk = value.map_err(|e| {
                    eprintln!("Error: {}", e);
                    ()
                })?;

                if chunk.starts_with(b"data: [DONE]") {
                    yield Ok("".to_string());
                    return;
                }

                let completition: Value = serde_json::from_slice(&chunk[5..]).map_err(|e| {
                    eprintln!("Error: {}", e);
                    ()
                })?;

                dbg!(&completition);

                let completation: Completation = serde_json::from_value(completition).map_err(|e| {
                    eprintln!("Error: {}", e);
                    ()
                })?;

                let message = completation
                    .choices
                    .iter()
                    .map(|c| c.delta.content.clone())
                    .collect::<String>();

                yield Ok(message);
            }
        };

        stream.boxed()
    }

    fn send(&mut self, bot: BotId, message: &str) -> BoxFuture<Result<String, ()>> {
        let stream = self.send_stream(bot, message);

        let future = async move {
            let parts = stream.collect::<Vec<_>>().await;

            if parts.contains(&Err(())) {
                return Err(());
            }

            let message = parts.into_iter().filter_map(Result::ok).collect::<String>();
            Ok(message)
        };

        future.boxed()
    }
}
