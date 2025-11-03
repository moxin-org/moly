use async_stream::stream;
use moly_kit::protocol::*;
use moly_kit::utils::asynchronous::{BoxPlatformSendFuture, BoxPlatformSendStream};
use std::collections::VecDeque;
use std::fmt::Write;

const HELP: &str = r#"Available commands:
- say <text>: The bot will respond with <text>.
- hello: The bot will respond with "world".
- ping: The bot will respond with "pong".
- wait: The bot will wait for some seconds and then respond with "done waiting".
- error: The bot will respond with a single error.
- errors: The bot will respond with multiple errors.
- never: The bot will never respond.
- inspect <index>: The bot will respond with the debug representation of the message at the given index.
- stream_lines <n>: The bot will progressively respond with n lines.
- help: Show this help message.
- \<anythig else\>: The bot will respond with "Not a command: \<anything else\>".
- \<empty message\>: The bot will respond with the debug representation of the last message."#;

pub struct TesterClient;

impl BotClient for TesterClient {
    fn bots(&self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        let future = futures::future::ready(ClientResult::new_ok(vec![Bot {
            id: BotId::new("tester", "tester"),
            name: "tester".to_string(),
            avatar: Picture::Grapheme("T".into()),
            capabilities: BotCapabilities::new(),
        }]));

        Box::pin(future)
    }

    fn send(
        &mut self,
        _bot_id: &BotId,
        messages: &[Message],
        _tools: &[Tool],
    ) -> BoxPlatformSendStream<'static, ClientResult<MessageContent>> {
        let messages = messages.to_vec();

        let stream = stream! {
            let last_message = messages
                .last()
                .expect("didn't receive any messages")
                .clone();

            let mut input = last_message
                .content
                .text
                .split_whitespace()
                .map(|b| b.to_lowercase())
                .collect::<VecDeque<_>>();

            match input.pop_front().as_deref() {
                Some("say") => {
                    let body = input.make_contiguous().join(" ");
                    yield ClientResult::new_ok(MessageContent {
                        text: body.into(),
                        ..Default::default()
                    });
                }
                Some("error") => {
                    yield ClientResult::new_err(
                        ClientError::new(
                            ClientErrorKind::Unknown,
                            "User requested a single error".into(),
                        )
                        .into(),
                    );
                },
                Some("errors") => {
                    yield ClientResult::new_err(vec![
                        ClientError::new(
                            ClientErrorKind::Unknown,
                            "User requested multiple errors".into(),
                        )
                        .into(),
                        ClientError::new(ClientErrorKind::Unknown, "This is another error".into())
                            .into(),
                    ]);
                },
                Some("hello") => {
                    yield ClientResult::new_ok(MessageContent {
                        text: "world".into(),
                        ..Default::default()
                    });
                },
                Some("ping") => {
                    yield ClientResult::new_ok(MessageContent {
                        text: "pong".into(),
                        ..Default::default()
                    });
                },
                Some("never") => {
                    futures::future::pending::<()>().await;
                    unreachable!();
                }
                Some("wait") => {
                    moly_kit::utils::asynchronous::sleep(std::time::Duration::from_secs(2)).await;
                    yield ClientResult::new_ok(MessageContent {
                        text: "done waiting".into(),
                        ..Default::default()
                    });
                }
                Some("help") => {
                    yield ClientResult::new_ok(MessageContent {
                        text: HELP.into(),
                        ..Default::default()
                    });
                },
                Some("inspect") => match input.pop_front().and_then(|s| s.parse::<usize>().ok()) {
                    Some(index) => {
                        let code = match messages.get(index) {
                            Some(message) => format!("{message:#?}"),
                            None => format!("None"),
                        };

                        yield ClientResult::new_ok(MessageContent {
                            text: format!("Message at index {index}:\n```\n{code}\n```").into(),
                            ..Default::default()
                        });
                    }
                    None => {
                        yield ClientResult::new_ok(MessageContent {
                            text: "Expected a message index after 'inspect' command".into(),
                            ..Default::default()
                        });
                    },
                },
                Some("stream_lines") => {
                    match input.pop_front().and_then(|s| s.parse::<usize>().ok()) {
                        Some(n) => {
                            let mut content = MessageContent::default();
                            for i in 1..=n {
                                moly_kit::utils::asynchronous::sleep(std::time::Duration::from_millis(500)).await;
                                write!(content.text, "This is line {i}\n\n").unwrap();
                                yield ClientResult::new_ok(content.clone());
                            }
                        }
                        None => {
                            yield ClientResult::new_ok(MessageContent {
                                text: "Expected a number after 'stream_lines' command".into(),
                                ..Default::default()
                            });
                        },
                    }
                }
                Some(_text) => {
                    let text = &last_message.content.text;
                    yield ClientResult::new_ok(MessageContent {
                        text: format!("Not a command: `{text}`"),
                        ..Default::default()
                    });
                }
                None => {
                    yield ClientResult::new_ok(MessageContent {
                        text: format!("Empty text at last message:\n```\n{last_message:#?}\n```")
                            .into(),
                        ..Default::default()
                    });
                },
            }
        };

        Box::pin(stream)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(TesterClient)
    }
}
