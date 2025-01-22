use futures::StreamExt;
use makepad_widgets::*;
use moly_widgets::*;
use prompt_input::PromptInputWidgetExt;

use crate::{clients::moly::MolyRepo, utils::spawn};

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
            self.handle_submit(cx);
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
    }
}

impl Ui {
    fn handle_submit(&mut self, cx: &mut Cx) {
        let prompt = self.prompt_input(id!(prompt));
        let text = prompt.text();
        prompt.borrow_mut().unwrap().reset(); // from command text input

        let bot_id = BotId::from("moly");

        self.messages(id!(messages))
            .borrow_mut()
            .unwrap()
            .messages
            .push(Message {
                from: EntityId::User,
                body: text.clone(),
                is_writing: false,
            });

        self.messages(id!(messages))
            .borrow_mut()
            .unwrap()
            .messages
            .push(Message {
                from: EntityId::Bot(bot_id),
                body: String::new(),
                is_writing: true,
            });

        self.redraw(cx);

        let mut client = self.bot_client.clone();
        let ui = self.ui_runner();

        spawn(async move {
            let mut message_stream = client.send_stream(BotId::from("moly"), &text);

            while let Some(delta) = message_stream.next().await {
                let delta = delta.unwrap_or_else(|_| "An error occurred".to_string());

                ui.defer_with_redraw(move |me, _cx, _scope| {
                    me.messages(id!(messages))
                        .borrow_mut()
                        .unwrap()
                        .messages
                        .last_mut()
                        .expect("no message where to put delta")
                        .body
                        .push_str(&delta);
                });
            }

            ui.defer_with_redraw(|me, _cx, _scope| {
                me.messages(id!(messages))
                    .borrow_mut()
                    .unwrap()
                    .messages
                    .last_mut()
                    .expect("no message where to put delta")
                    .is_writing = false;
            });
        });
    }
}
