use futures::{stream::AbortHandle, StreamExt};
use makepad_widgets::*;
use std::cell::{Ref, RefMut};
use utils::asynchronous::spawn;

use crate::utils::events::EventExt;
use crate::utils::ui_runner::DeferWithRedrawAsync;
use crate::*;

live_design!(
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;
    use link::shaders::*;

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
    /// 
    /// The boolean indicates if the scroll was triggered by a stream or not.
    ScrollToBottom(bool),
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

    /// Used to control we are putting message deltas into the right message during
    /// streaming.
    // Note: If messages had unique identifiers, we wouldn't need to keep a copy of
    // the message as a workaround.
    #[rust]
    expected_message: Option<Message>,

    #[rust]
    hook_before: Option<Box<dyn FnMut(&mut Vec<ChatTask>, &mut Chat, &mut Cx)>>,

    #[rust]
    hook_after: Option<Box<dyn FnMut(&[ChatTask], &mut Chat, &mut Cx)>>,

    #[rust]
    is_hooking: bool,

    /// Wether the user has scrolled during the stream of the current message.
    #[rust]
    user_scrolled_during_stream: bool,
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
        self.handle_scrolling();
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

    fn handle_scrolling(&mut self) {
        // If we are waiting for a message, update wether the user has scrolled during the stream.
        if self.expected_message.is_some() {
            self.user_scrolled_during_stream = self.messages_ref().read().user_scrolled();
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
                        message.content.text = m.current_editor_text().expect("no editor text");
                        ChatTask::UpdateMessage(index, message).into()
                    });

                    self.dispatch(cx, &mut tasks);
                }
                MessagesAction::EditRegenerate(index) => {
                    let mut tasks = self.messages_ref().read_with(|m| {
                        let mut messages = m.messages[0..=index].to_vec();

                        let index = m.current_editor_index().expect("no editor index");
                        let text = m.current_editor_text().expect("no editor text");

                        messages[index].content.text = text;

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
                        content: MessageContent {
                            text,
                            ..Default::default()
                        },
                        is_writing: false,
                    },
                ));
            }

            composition.extend([ChatTask::Send, ChatTask::ClearPrompt]);

            self.dispatch(cx, &mut composition);
        } else if prompt.read().has_stop_task() {
            self.dispatch(cx, &mut ChatTask::Stop.into());
        }
    }

    fn handle_send_task(&mut self, cx: &mut Cx) {
        // Let's start clean before starting a new stream.
        self.abort();

        // TODO: See `bot_id` TODO.
        let bot_id = self.bot_id.clone().expect("no bot selected");

        let repo = self
            .bot_repo
            .as_ref()
            .expect("no bot repo provided")
            .clone();

        // First check if the bot exists in the repository
        if repo.get_bot(&bot_id).is_none() {
            // Bot not found, add error message
            let next_index = self.messages_ref().read().messages.len();
            let error_message = format!("App error: Bot not found. The bot might have been disabled or removed. Bot ID: {}", bot_id);
            
            let message = Message {
                from: EntityId::App,
                content: MessageContent {
                    text: error_message,
                    ..Default::default()
                },
                is_writing: false,
            };
            
            self.dispatch(cx, &mut vec![ChatTask::InsertMessage(next_index, message)]);
            return;
        }

        let context: Vec<Message> = self.messages_ref().write_with(|messages| {
            messages.bot_repo = Some(repo.clone());

            messages.messages.push(Message {
                from: EntityId::Bot(bot_id.clone()),
                is_writing: true,
                ..Default::default()
            });

            messages
                .messages
                .iter()
                .filter(|m| !m.is_writing && m.from != EntityId::App)
                .cloned()
                .collect()
        });

        self.dispatch(cx, &mut ChatTask::ScrollToBottom(false).into());
        self.prompt_input_ref().write().set_stop();
        self.redraw(cx);

        let ui = self.ui_runner();
        let future = async move {
            let mut client = repo.client();
            let bot = match repo.get_bot(&bot_id) {
                Some(bot) => bot,
                None => {
                    // This should never happen as we check above, but handle it gracefully anyway
                    let bot_id_clone = bot_id.clone(); // Clone the bot_id for the closure
                    ui.defer_with_redraw(move |me, cx, _| {
                        let error_message = format!("App error: Bot not found during stream initialization. Bot ID: {}", bot_id_clone);
                        let next_index = me.messages_ref().read().messages.len();
                        let message = Message {
                            from: EntityId::App,
                            content: MessageContent {
                                text: error_message,
                                ..Default::default()
                            },
                            is_writing: false,
                        };
                        me.dispatch(cx, &mut vec![ChatTask::InsertMessage(next_index, message)]);
                    });
                    return;
                }
            };

            let mut message_stream = client.send_stream(&bot, &context);
            while let Some(result) = message_stream.next().await {
                // In theory, with the synchroneous defer, if stream messages come
                // faster than deferred closures are executed, and one closure causes
                // an abort, the other already deferred closures will still be executed
                // and may cause race conditions.
                //
                // In practice, this never happened in my tests. But better safe than
                // sorry. The async variant let this async context wait before processing
                // the next delta. And also allows to stop naturally from here as well
                // thanks to its ability to send a value back.
                let should_break = ui
                    .defer_with_redraw_async(move |me, cx, _| me.handle_message_delta(cx, result))
                    .await;

                if should_break.unwrap_or(true) {
                    break;
                }
            }
        };

        let (future, abort_handle) = futures::future::abortable(future);
        self.abort_handle = Some(abort_handle);

        spawn(async move {
            // The wrapper Future is only error if aborted.
            //
            // Cleanup caused by signaling stuff like `Chat::abort` should be done synchronously
            // so one can abort and immediately start a new stream without race conditions.
            //
            // Only cleanup after natural termination of the stream should be here.
            if future.await.is_ok() {
                ui.defer_with_redraw(|me, _, _| me.clean_streaming_artifacts());
            }
        });
    }

    fn handle_stop_task(&mut self, cx: &mut Cx) {
        self.abort();
        self.redraw(cx);
    }

    /// Immediately remove resources/data related to the current streaming and signal
    /// the stream to stop as soon as possible.
    fn abort(&mut self) {
        self.abort_handle.take().map(|handle| handle.abort());
        self.clean_streaming_artifacts();
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
                    let text = &m.messages[*index].content.text;
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
                self.handle_send_task(cx);
            }
            ChatTask::Stop => {
                self.handle_stop_task(cx);
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
            ChatTask::ScrollToBottom(triggered_by_stream) => {
                self.messages_ref().write().scroll_to_bottom(cx, *triggered_by_stream);
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

    /// Remove data related to current streaming, leaving everything ready for a new one.
    ///
    /// Called as soon as possible after streaming completes naturally or immediately when
    /// calling [Chat::abort].
    fn clean_streaming_artifacts(&mut self) {
        self.abort_handle = None;
        self.expected_message = None;
        self.user_scrolled_during_stream = false;
        self.messages_ref().write().reset_scroll_state();
        self.prompt_input_ref().write().set_send();
        self.messages_ref().write().messages.retain_mut(|m| {
            m.is_writing = false;
            !m.content.is_empty()
        });
    }

    fn handle_message_delta(&mut self, cx: &mut Cx, result: ClientResult<MessageContent>) -> bool {
        let messages = self.messages_ref();

        // For simplicity, lets handle this as an standard Result, ignoring delta
        // if there are errors.
        match result.into_result() {
            Ok(content) => {
                // Let's abort if we don't have where to put the delta.
                let Some(mut message) = messages.read().messages.last().cloned() else {
                    return true;
                };

                // Let's abort if we see we are putting delta in the wrong message.
                if let Some(expected_message) = self.expected_message.as_ref() {
                    if message != *expected_message {
                        return true;
                    }
                }

                message.content = content;

                let index = messages.read().messages.len() - 1;
                let mut tasks = vec![ChatTask::UpdateMessage(index, message)];

                // Stick the chat to the bottom if the user didn't manually scroll.
                if !self.user_scrolled_during_stream {
                    tasks.push(ChatTask::ScrollToBottom(true));
                }

                self.dispatch(cx, &mut tasks);

                let Some(ChatTask::UpdateMessage(_, message)) = tasks.into_iter().next() else {
                    // Let's abort if the tasks were modified in an unexpected way.
                    return true;
                };

                self.expected_message = Some(message);

                false
            }
            Err(errors) => {
                let mut tasks = errors
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| {
                        ChatTask::InsertMessage(
                            messages.read().messages.len() + i,
                            Message {
                                from: EntityId::App,
                                content: MessageContent {
                                    text: e.to_string(),
                                    ..Default::default()
                                },
                                is_writing: false,
                            },
                        )
                    })
                    .collect::<Vec<_>>();

                // Stick the chat to the bottom if the user didn't manually scroll.
                if !self.user_scrolled_during_stream {
                    tasks.push(ChatTask::ScrollToBottom(true));
                }

                self.dispatch(cx, &mut tasks);

                true
            }
        }
    }

    /// Returns true if the chat is currently streaming.
    pub fn is_streaming(&self) -> bool {
        if let Some(message) = self.messages_ref().read().messages.last() {
            message.is_writing
        } else {
            false
        }
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
