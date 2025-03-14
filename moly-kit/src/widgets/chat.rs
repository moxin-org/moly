use futures::{stream::AbortHandle, StreamExt};
use makepad_widgets::*;
use std::cell::{Ref, RefMut};
use utils::asynchronous::spawn;

use crate::utils::events::EventExt;
use crate::*;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::widgets::messages::*;
    use crate::widgets::prompt_input::*;

    pub Chat = {{Chat}} <RoundedView> {
        flow: Down,
        messages = <Messages> {}
        prompt = <PromptInput> {}
    }
);

/// A task of interest that was or will be performed by the [Chat] widget.
///
/// You can get notified when a group of tasks were already executed by using [Chat::set_hook_after].
///
/// You can also "hook" into the group of tasks before it's executed with [Chat::set_hook_before].
/// This allows you to modify their payloads (which are used by the task when executed), add and remove
/// tasks from the group, abort the group (by clearing the tasks vector), etc.
// TODO: Using indexes for many operations like `UpdateMessage` is not ideal. In the future
// messages may need to have a unique identifier.
#[derive(Debug)]
pub enum ChatTask {
    /// When received back, it will send the whole chat context to the bot.
    Send,

    /// When received back, it will cancel the response stream from the bot.
    Stop,

    /// When received back, it will copy the message at the given index to the clipboard.
    CopyMessage(usize),

    /// When received back, it will re-write the message history with the given messages.
    SetMessages(Vec<Message>),

    /// When received back, it will insert a message at the given index.
    InsertMessage(usize, Message),

    /// When received back, it will delete the message at the given index.
    DeleteMessage(usize),

    /// When received back, it will update the message at the given index.
    UpdateMessage(usize, Message),

    /// When received back, it will clear the prompt input.
    ClearPrompt,

    /// When received back, the chat will scroll to the bottom.
    ScrollToBottom,
}

impl From<ChatTask> for Vec<ChatTask> {
    fn from(task: ChatTask) -> Self {
        vec![task]
    }
}

/// A batteries-included chat to to implement chatbots.
#[derive(Live, LiveHook, Widget)]
pub struct Chat {
    #[deref]
    deref: View,

    /// The bot repository used by this chat to hold bots and interact with them.
    #[rust]
    pub bot_repo: Option<BotRepo>,

    /// The id of the bot the chat will message when sending.
    // TODO: Can this be live?
    // TODO: Default to the first bot in the repo if `None`.
    #[rust]
    pub bot_id: Option<BotId>,

    /// Toggles response streaming on or off. Default is on.
    // TODO: Implement this.
    #[live(true)]
    pub stream: bool,

    #[rust]
    abort_handle: Option<AbortHandle>,

    #[rust]
    hook_before: Option<Box<dyn FnMut(&mut Vec<ChatTask>, &mut Chat, &mut Cx)>>,

    #[rust]
    hook_after: Option<Box<dyn FnMut(&[ChatTask], &mut Chat, &mut Cx)>>,

    #[rust]
    is_hooking: bool,
}

impl Widget for Chat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Pass down the bot repo if not the same.
        self.messages_ref().write_with(|m| {
            if m.bot_repo != self.bot_repo {
                m.bot_repo = self.bot_repo.clone();
            }
        });

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);
        self.handle_messages(cx, event);
        self.handle_prompt_input(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl Chat {
    /// Getter to the underlying [PromptInputRef] independent of its id.
    pub fn prompt_input_ref(&self) -> PromptInputRef {
        self.prompt_input(id!(prompt))
    }

    /// Getter to the underlying [MessagesRef] independent of its id.
    pub fn messages_ref(&self) -> MessagesRef {
        self.messages(id!(messages))
    }

    fn handle_prompt_input(&mut self, cx: &mut Cx, event: &Event) {
        if self.prompt_input_ref().read().submitted(event.actions()) {
            self.handle_submit(cx);
        }
    }

    fn handle_messages(&mut self, cx: &mut Cx, event: &Event) {
        for action in event.actions() {
            let Some(action) = action.as_widget_action() else {
                continue;
            };

            if action.widget_uid != self.messages_ref().widget_uid() {
                continue;
            }

            match action.cast::<MessagesAction>() {
                MessagesAction::Delete(index) => {
                    self.dispatch(cx, &mut ChatTask::DeleteMessage(index).into());
                }
                MessagesAction::Copy(index) => {
                    self.dispatch(cx, &mut ChatTask::CopyMessage(index).into());
                }
                MessagesAction::EditSave(index) => {
                    let mut tasks = self.messages_ref().read_with(|m| {
                        let mut message = m.messages[index].clone();
                        message.body = m.current_editor_text().expect("no editor text");
                        ChatTask::UpdateMessage(index, message).into()
                    });

                    self.dispatch(cx, &mut tasks);
                }
                MessagesAction::EditRegenerate(index) => {
                    let mut tasks = self.messages_ref().read_with(|m| {
                        let mut messages = m.messages[0..=index].to_vec();

                        let index = m.current_editor_index().expect("no editor index");
                        let text = m.current_editor_text().expect("no editor text");

                        messages[index].body = text;
                        vec![ChatTask::SetMessages(messages), ChatTask::Send]
                    });

                    self.dispatch(cx, &mut tasks);
                }
                MessagesAction::None => {}
            }
        }
    }

    fn handle_submit(&mut self, cx: &mut Cx) {
        let prompt = self.prompt_input_ref();

        if prompt.read().has_send_task() {
            let next_index = self.messages_ref().read().messages.len();
            let text = prompt.text();
            let mut composition = Vec::new();

            if !text.is_empty() {
                composition.push(ChatTask::InsertMessage(
                    next_index,
                    Message {
                        from: EntityId::User,
                        body: text.clone(),
                        is_writing: false,
                        citations: vec![],
                    },
                ));
            }

            composition.extend([ChatTask::Send, ChatTask::ClearPrompt]);

            self.dispatch(cx, &mut composition);
        } else if prompt.read().has_stop_task() {
            self.dispatch(cx, &mut ChatTask::Stop.into());
        }
    }

    fn perform_send(&mut self, cx: &mut Cx) {
        // TODO: See `bot_id` TODO.
        let bot_id = self.bot_id.clone().expect("no bot selected");

        let repo = self
            .bot_repo
            .as_ref()
            .expect("no bot repo provided")
            .clone();

        let context: Vec<Message> = self.messages_ref().write_with(|messages| {
            messages.bot_repo = Some(repo.clone());

            messages.messages.push(Message {
                from: EntityId::Bot(bot_id.clone()),
                body: String::new(),
                is_writing: true,
                citations: vec![],
            });

            messages
                .messages
                .iter()
                .filter(|m| !m.is_writing && m.from != EntityId::App)
                .cloned()
                .collect()
        });

        self.dispatch(cx, &mut ChatTask::ScrollToBottom.into());
        self.prompt_input_ref().write().set_stop();
        self.redraw(cx);

        let ui = self.ui_runner();
        let future = async move {
            let mut client = repo.client();
            let mut message_stream = client.send_stream(&bot_id, &context);

            while let Some(delta) = message_stream.next().await {
                let delta = match delta {
                    Ok(delta) => delta,
                    Err(_) => MessageDelta {
                        body: "An error occurred".to_string(),
                        ..Default::default()
                    },
                };

                ui.defer_with_redraw(move |me, cx, _scope| {
                    let (index, message, is_at_bottom) = me.messages_ref().read_with(|messages| {
                        let mut message = messages
                            .messages
                            .last()
                            .expect("no message where to put delta")
                            .clone();

                        message.body.push_str(&delta.body);
                        // TODO: Maybe this is a good case for a sorted set, like `BTreeSet`.
                        for citation in delta.citations {
                            if !message.citations.contains(&citation) {
                                message.citations.push(citation);
                            }
                        }

                        (
                            messages.messages.len() - 1,
                            message,
                            messages.is_at_bottom(),
                        )
                    });

                    me.dispatch(cx, &mut ChatTask::UpdateMessage(index, message).into());

                    if is_at_bottom {
                        me.dispatch(cx, &mut ChatTask::ScrollToBottom.into());
                    }
                });
            }

            ui.defer_with_redraw(|me, _cx, _scope| {
                me.messages_ref().write_with(|messages| {
                    messages
                        .messages
                        .last_mut()
                        .expect("no message where to put delta")
                        .is_writing = false;
                });
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

    fn perform_stop(&mut self, _cx: &mut Cx) {
        self.abort_handle.take().map(|handle| handle.abort());
    }

    /// Dispatch a set of tasks to be executed by the [Chat] widget as a single hookable
    /// unit of work.
    ///
    /// You can still hook into these tasks before they are executed if you set a hook with
    /// [Chat::set_hook_before].
    ///
    /// Warning: Like other operation over makepad's [WidgetRef], this function may panic if you hold
    /// borrows to widgets inside [Chat], for example [Messages] or [PromptInput]. Be aware when using
    /// `read_with` and `write_with` methods.
    // TODO: Mitigate interior mutability issues with many tricks or improving makepad.
    pub fn dispatch(&mut self, cx: &mut Cx, tasks: &mut Vec<ChatTask>) {
        self.is_hooking = true;

        if let Some(mut hook) = self.hook_before.take() {
            hook(tasks, self, cx);
            self.hook_before = Some(hook);
        }

        for task in tasks.iter() {
            self.handle_task(cx, task);
        }

        if let Some(mut hook) = self.hook_after.take() {
            hook(tasks, self, cx);
            self.hook_after = Some(hook);
        }

        self.is_hooking = false;
    }

    /// Performs a set of tasks in the [Chat] widget immediately.
    ///
    /// This is not hookable.
    pub fn perform(&mut self, cx: &mut Cx, tasks: &[ChatTask]) {
        for task in tasks {
            self.handle_task(cx, &task);
        }
    }

    fn handle_task(&mut self, cx: &mut Cx, task: &ChatTask) {
        match task {
            ChatTask::CopyMessage(index) => {
                self.messages_ref().read_with(|m| {
                    let text = m.messages[*index].body.as_str();
                    cx.copy_to_clipboard(text);
                });
            }
            ChatTask::DeleteMessage(index) => {
                self.messages_ref().write().messages.remove(*index);
                self.redraw(cx);
            }
            ChatTask::InsertMessage(index, message) => {
                self.messages_ref()
                    .write()
                    .messages
                    .insert(*index, message.clone());
                self.redraw(cx);
            }
            ChatTask::Send => {
                self.perform_send(cx);
            }
            ChatTask::Stop => {
                self.perform_stop(cx);
            }
            ChatTask::UpdateMessage(index, message) => {
                self.messages_ref().write_with(|m| {
                    m.messages[*index] = message.clone();
                    m.set_message_editor_visibility(*index, false);
                });

                self.redraw(cx);
            }
            ChatTask::SetMessages(messages) => {
                self.messages_ref().write_with(|m| {
                    m.messages = messages.clone();

                    if let Some(index) = m.current_editor_index() {
                        m.set_message_editor_visibility(index, false);
                    }
                });

                self.redraw(cx);
            }
            ChatTask::ClearPrompt => {
                self.prompt_input_ref().write().reset(cx); // `reset` comes from command text input.
            }
            ChatTask::ScrollToBottom => {
                self.messages_ref().write().scroll_to_bottom(cx);
            }
        }
    }

    /// Sets a hook to be executed before a group of tasks is executed.
    ///
    /// You get mutable access to the group of tasks, so you can modify what is
    /// about to happen. See [ChatTask] for more details about this.
    ///
    /// If you just want to get notified when something already happened, see [Chat::set_hook_after].
    pub fn set_hook_before(
        &mut self,
        hook: impl FnMut(&mut Vec<ChatTask>, &mut Chat, &mut Cx) + 'static,
    ) {
        if self.is_hooking {
            panic!("Cannot set a hook while hooking");
        }

        self.hook_before = Some(Box::new(hook));
    }

    /// Sets a hook to be executed after a group of tasks is executed.
    ///
    /// You get immutable access to the group of tasks, so you can inspect what happened.
    pub fn set_hook_after(&mut self, hook: impl FnMut(&[ChatTask], &mut Chat, &mut Cx) + 'static) {
        if self.is_hooking {
            panic!("Cannot set a hook while hooking");
        }

        self.hook_after = Some(Box::new(hook));
    }
}

// TODO: Since `ChatRef` is generated by a macro, I can't document this to give
// these functions better visibility from the module view.
impl ChatRef {
    /// Immutable access to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<Chat> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<Chat> {
        self.borrow_mut().unwrap()
    }

    /// Immutable reader to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read_with<R>(&self, f: impl FnOnce(&Chat) -> R) -> R {
        f(&*self.read())
    }

    /// Mutable writer to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write_with<R>(&mut self, f: impl FnOnce(&mut Chat) -> R) -> R {
        f(&mut *self.write())
    }
}
