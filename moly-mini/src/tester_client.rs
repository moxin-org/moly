use moly_kit::protocol::*;
use std::collections::VecDeque;

pub struct TesterClient;

impl BotClient for TesterClient {
    fn bots(&self) -> MolyFuture<'static, ClientResult<Vec<Bot>>> {
        let future = futures::future::ready(ClientResult::new_ok(vec![Bot {
            id: BotId::new("tester", "tester"),
            name: "tester".to_string(),
            avatar: Picture::Grapheme("T".into()),
        }]));

        moly_future(future)
    }

    fn send(
        &mut self,
        _bot: &Bot,
        messages: &[Message],
    ) -> MolyStream<'static, ClientResult<MessageContent>> {
        let mut input = messages
            .last()
            .expect("didn't receive any messages")
            .content
            .text
            .split_whitespace()
            .map(|b| b.to_lowercase())
            .collect::<VecDeque<_>>();

        let stream = futures::stream::once(async move {
            match input.pop_front().as_deref() {
                Some("say") => {
                    let body = input.make_contiguous().join(" ");
                    ClientResult::new_ok(MessageContent {
                        text: body.into(),
                        ..Default::default()
                    })
                }
                Some("error") => ClientResult::new_err(
                    ClientError::new(
                        ClientErrorKind::Unknown,
                        "User requested a single error".into(),
                    )
                    .into(),
                ),
                Some("errors") => ClientResult::new_err(vec![
                    ClientError::new(
                        ClientErrorKind::Unknown,
                        "User requested multiple errors".into(),
                    )
                    .into(),
                    ClientError::new(ClientErrorKind::Unknown, "This is another error".into())
                        .into(),
                ]),
                Some("hello") => ClientResult::new_ok(MessageContent {
                    text: "world".into(),
                    ..Default::default()
                }),
                Some("ping") => ClientResult::new_ok(MessageContent {
                    text: "pong".into(),
                    ..Default::default()
                }),
                _ => ClientResult::new_ok(MessageContent {
                    text: "Yeah...".into(),
                    ..Default::default()
                }),
            }
        });

        moly_stream(stream)
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(TesterClient)
    }
}
