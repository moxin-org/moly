use futures::StreamExt;
use makepad_widgets::*;
use utils::asynchronous::spawn;

use crate::*;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::messages::*;
    use crate::prompt_input::*;

    pub Chat = {{Chat}} <RoundedView> {
        flow: Down,
        messages = <Messages> {}
        prompt = <PromptInput> {}
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct Chat {
    #[deref]
    deref: View,

    #[rust]
    pub bot_repo: Option<BotRepo>,

    // TODO: Can this be live?
    #[rust]
    pub bot_id: Option<BotId>,
}

impl Widget for Chat {
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

impl Chat {
    fn handle_submit(&mut self, cx: &mut Cx) {
        let messages = self.messages(id!(messages));
        let prompt = self.prompt_input(id!(prompt));

        let text = prompt.text();
        prompt.borrow_mut().unwrap().reset(); // from command text input

        // TODO: Less aggresive error handling for users.
        let bot_id = self.bot_id.clone().expect("no bot selected");

        let repo = self
            .bot_repo
            .as_ref()
            .expect("no bot repo provided")
            .clone();

        let context: Vec<Message> = {
            let mut messages = messages.borrow_mut().unwrap();

            messages.bot_repo = Some(repo.clone());

            messages.messages.push(Message {
                from: EntityId::User,
                body: text.clone(),
                is_writing: false,
            });

            messages.messages.push(Message {
                from: EntityId::Bot(bot_id.clone()),
                body: String::new(),
                is_writing: true,
            });

            messages
                .messages
                .iter()
                .filter(|m| !m.is_writing && m.from != EntityId::App)
                .cloned()
                .collect()
        };

        self.redraw(cx);

        let ui = self.ui_runner();

        spawn(async move {
            let mut client = repo.client();
            let mut message_stream = client.send_stream(bot_id, &context);

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

                log!(
                    "{}",
                    me.messages(id!(messages))
                        .borrow_mut()
                        .unwrap()
                        .messages
                        .last()
                        .unwrap()
                        .body
                );
            });
        });
    }
}
