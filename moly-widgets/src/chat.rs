use futures::{stream::AbortHandle, StreamExt};
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

    #[rust]
    abort_handle: Option<AbortHandle>,
}

impl Widget for Chat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        let Event::Actions(actions) = event else {
            return;
        };

        if self.prompt_input_ref().read().submitted(actions) {
            self.handle_submit(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl Chat {
    pub fn prompt_input_ref(&self) -> PromptInputRef {
        self.prompt_input(id!(prompt))
    }

    pub fn messages_ref(&self) -> MessagesRef {
        self.messages(id!(messages))
    }

    fn handle_submit(&mut self, cx: &mut Cx) {
        let prompt = self.prompt_input_ref();

        if prompt.read().has_send_task() {
            self.handle_send(cx);
        } else if prompt.read().has_stop_task() {
            self.handle_stop(cx);
        }
    }

    fn handle_send(&mut self, cx: &mut Cx) {
        let prompt = self.prompt_input_ref();

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
            let mut messages = self.messages_ref();
            let mut messages = messages.write();

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

        self.prompt_input_ref().write().set_stop();
        self.redraw(cx);

        let ui = self.ui_runner();
        let future = async move {
            let mut client = repo.client();
            let mut message_stream = client.send_stream(&bot_id, &context);

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
                me.messages_ref()
                    .write()
                    .messages
                    .last_mut()
                    .expect("no message where to put delta")
                    .is_writing = false;
            });
        };

        let (future, abort_handle) = futures::future::abortable(future);

        self.abort_handle = Some(abort_handle);

        spawn(async move {
            future.await.unwrap_or_else(|_| log!("Aborted"));
            ui.defer_with_redraw(|me, _cx, _scope| {
                me.abort_handle = None;
                me.prompt_input_ref().write().set_send();
            });
        });
    }

    fn handle_stop(&mut self, _cx: &mut Cx) {
        self.abort_handle.take().map(|handle| handle.abort());
    }
}
