use std::future::Future;

use futures_core::Stream;
use makepad_widgets::*;
use moly_widgets::*;
use prompt_input::PromptInputWidgetExt;

use crate::clients::moly::MolyRepo;

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
                // remove the default label
                label = <Label> {}
                <View> {
                    width: Fill,
                    align: {x: 0.5, y: 0.5},
                    <Label> {
                        text: "moly-mini"
                        draw_text: {
                            color: #000
                        }
                    }
                }
                port = <TextInput> {
                    width: 100,
                    empty_message: "Port..."
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

    #[rust]
    bot_client: MolyRepo,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        let Event::Actions(actions) = event else {
            return;
        };

        if self.prompt_input(id!(prompt)).submitted(actions) {
            self.handle_submit();
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let messages = self.messages(id!(messages));

        messages.borrow_mut().unwrap().bot_client =
            Some(Box::new(crate::clients::moly::MolyRepo::default()));

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
        ];
    }
}

impl Ui {
    fn handle_submit(&self) {
        let rt = tokio::runtime::Handle::current();
        let mut client = self.bot_client.clone();
        let text = self.prompt_input(id!(prompt)).text();
        let ui = self.ui_runner();

        client.port = self.text_input(id!(port)).text().parse().unwrap_or(0);

        self.messages(id!(messages))
            .borrow_mut()
            .unwrap()
            .messages
            .push(Message {
                from: EntityId::User,
                body: text.clone(),
                is_writing: false,
            });

        rt.spawn(async move {
            let result = client
                .send(BotId::from("moly"), &text)
                .await
                .unwrap_or_else(|_| "An error occurred".to_string());

            ui.defer_with_redraw(|me, _cx, _scope| {
                me.messages(id!(messages))
                    .borrow_mut()
                    .unwrap()
                    .messages
                    .push(Message {
                        from: EntityId::Bot(BotId::from("moly")),
                        body: result,
                        is_writing: false,
                    });

                me.prompt_input(id!(prompt)).set_text("");
            });
        });
    }
}
