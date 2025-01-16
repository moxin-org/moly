use std::future::Future;

use futures_core::Stream;
use makepad_widgets::*;
use moly_widgets::*;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use moly_widgets::messages::Messages;
    use moly_widgets::prompt_input::PromptInput;

    pub Ui = {{Ui}} <Window> {
        align: {x: 0.5, y: 0.5}
        pass: { clear_color: #fff }

        caption_bar = {
            caption_label = {
                label = <Label> {
                    text: "moly-mini"
                    draw_text: {
                        color: #000
                    }
                }
            }

            visible: true,
        }

        body = <View> {
            flow: Down,
            padding: 12,
            messages = <Messages> {}
            prompt = <PromptInput> {}
        }
    }
);

#[derive(Live, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let messages = self.messages(id!(messages));
        messages.borrow_mut().unwrap().messages = vec![
            Message {
                from: EntityId::User,
                body: "Hello, world!".to_string(),
                is_writing: false,
            },
            Message {
                from: EntityId::Bot(BotId::from("bot")),
                body: "Hello, bot!".to_string(),
                is_writing: false,
            },
            Message {
                from: EntityId::Bot(BotId::from("bot")),
                body: "".to_string(),
                is_writing: true,
            },
        ];

        messages.borrow_mut().unwrap().bot_client = Some(Box::new(DummyBotClient {
            bots: vec![DummyBot {
                avatar: Picture::Grapheme("D".to_string()),
            }],
        }));
    }
}

#[derive(Clone)]
struct DummyBot {
    avatar: Picture,
}

impl Bot for DummyBot {
    fn id(&self) -> BotId {
        BotId::from("bot")
    }

    fn name(&self) -> &str {
        "Dummy Bot"
    }

    fn avatar(&self) -> &Picture {
        &self.avatar
    }
}

#[derive(Clone)]
struct DummyBotClient {
    bots: Vec<DummyBot>,
}

impl BotClient for DummyBotClient {
    fn get_bot(&self, id: BotId) -> Option<&dyn Bot> {
        self.bots
            .iter()
            .find(|bot| bot.id() == id)
            .map(|bot| bot as &dyn Bot)
    }

    fn get_bot_mut(&mut self, id: BotId) -> Option<&mut dyn Bot> {
        self.bots
            .iter_mut()
            .find(|bot| bot.id() == id)
            .map(|bot| bot as &mut dyn Bot)
    }

    fn send(&mut self, bot: BotId, message: &str) -> Box<dyn Future<Output = Result<String, ()>>> {
        Box::new(async move { Ok("Hello, world!".to_string()) })
    }

    fn send_stream(
        &mut self,
        bot: BotId,
        message: &str,
    ) -> Box<dyn Stream<Item = Result<String, ()>>> {
        Box::new(futures::stream::iter(vec![Ok("Hello, world!".to_string())]))
    }

    fn stop(&mut self, bot: BotId) {
        unimplemented!()
    }

    fn bots(&self) -> Box<dyn Iterator<Item = &dyn Bot> + '_> {
        Box::new(self.bots.iter().map(|bot| bot as &dyn Bot))
    }

    fn clone_box(&self) -> Box<dyn BotClient> {
        Box::new(self.clone())
    }
}
