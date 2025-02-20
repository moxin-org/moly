use futures::{stream::AbortHandle, StreamExt};
use makepad_widgets::*;
use std::cell::{Ref, RefMut};
use std::sync::RwLock;
use utils::asynchronous::spawn;

use crate::utils::events::EventExt;
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

/// Private action that carries a [ChatHook] for a chat widget.
#[derive(Debug)]
struct ChatAction {
    hook: RwLock<ChatHook>,
    // A widget action is more strict than an action as it needs to implement `ActionDefaultRef`,
    // so this will keep things simple here inside.
    widget_uid: WidgetUid,
}

/// Encapsulates a [ChatTask] that can me modified before being executed by the chat widget.
#[derive(Debug)]
pub struct ChatHook {
    executed: bool,
    task: Option<ChatTask>,
}

impl ChatHook {
    pub fn abort(&mut self) {
        self.task = None;
    }

    pub fn task(&self) -> &ChatTask {
        self.task
            .as_ref()
            .expect("the task in this hook has been aborted")
    }

    pub fn task_mut(&mut self) -> &mut ChatTask {
        self.task
            .as_mut()
            .expect("the task in this hook has been aborted")
    }
}

/// A task that was or will be performed by the [Chat] widget depending on if you
/// read it before the chat widget receives it back or not.
///
/// The payload in this task will be used to perform the task itself. If you have
/// access to its wrapper [ChatHook], you can modify the task before it is executed.
///
/// See [Chat::tasks] and [Chat::hook] for more information.
#[derive(Debug)]
pub enum ChatTask {
    /// Sending the current chat for completition.
    Send,

    /// Stoping the current completition.
    Stop,

    /// Copying the message at the given index to the clipboard.
    CopyMessage(usize),

    /// Rewriting the message history.
    SetMessages(Vec<Message>),

    /// Inserting a new message at the given index.
    InsertMessage(usize, EntityId, String),

    /// Deleting the message at the given index.
    DeleteMessage(usize),

    /// Editing the message at the given index with the given text.
    UpdateMessage(usize, String),

    /// A task that is not it's own but a sequence of other tasks.
    ///
    /// Ex: "Edit & Regenerate" button is actually deleting part of the chat history,
    /// editing the message, and sending the chat history again.
    ///
    /// Ex: "Send" on its own only sends the current chat history. It's composed with
    /// an insert to send a new message.
    Compose(Vec<ChatTask>),
}

/// An intermidiate type that can read [ChatTask] from an [Event].
///
/// Avoids retaining [Chat] self reference during closure execution.
pub struct ChatTaskReader<'e> {
    event: &'e Event,
    widget_uid: WidgetUid,
}

impl<'e> ChatTaskReader<'e> {
    /// Construct a new [ChatTaskReader]. Prefer using [Chat::tasks] instead.
    fn new(widget_uid: WidgetUid, event: &'e Event) -> Self {
        Self { widget_uid, event }
    }

    /// Read the tasks from the event.
    pub fn read(&self, mut reader: impl FnMut(&ChatTask)) {
        for action in chat_actions(self.widget_uid, self.event) {
            let hook = action.hook.read().expect("the task is being hooked");
            let Some(task) = &hook.task else {
                return;
            };
            reader(&task);
        }
    }
}

/// An intermidiate type that can read/write [ChatHook] in an [Event].
///
/// Avoids retaining [Chat] self reference during closure execution.
pub struct ChatHookWriter<'e> {
    event: &'e Event,
    widget_uid: WidgetUid,
}

impl<'e> ChatHookWriter<'e> {
    /// Construct a new [ChatHookWriter]. Prefer using [Chat::hook] instead.
    fn new(widget_uid: WidgetUid, event: &'e Event) -> Self {
        Self { widget_uid, event }
    }

    /// Get write access to hooks in this event.
    pub fn write(&self, mut hook_fn: impl FnMut(&mut ChatHook)) {
        for action in chat_actions(self.widget_uid, self.event) {
            {
                // let's use `read` first to avoid panicking before other checks
                let hook = action.hook.read().expect("the task is being hooked");

                if hook.task.is_none() {
                    return;
                }

                if hook.executed {
                    panic!(
                        "Hooking into a chat task that has already been executed. \
                        Changes to the task would not have effect so this is invalid. \
                        If you are trying to read the task without changing it, use `tasks` instead. \
                        If you are trying to change the task, you should do it before `Chat`'s `handle_event`."
                    );
                }
            }

            let mut hook = action.hook.write().expect("the task is being hooked");
            hook_fn(&mut *hook);
        }
    }
}

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
        // Pass down the bot repo if not the same.
        self.messages_ref().write_with(|m| {
            if m.bot_repo != self.bot_repo {
                m.bot_repo = self.bot_repo.clone();
            }
        });

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);
        self.handle_tasks(cx, event);
        self.handle_messages(cx, event);
        self.handle_prompt_input(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl Chat {
    /// Getter to the underlying [[PromptInputRef]] independent of its id.
    pub fn prompt_input_ref(&self) -> PromptInputRef {
        self.prompt_input(id!(prompt))
    }

    /// Getter to the underlying [[MessagesRef]] independent of its id.
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
                    self.dispatch(cx, ChatTask::DeleteMessage(index));
                }
                MessagesAction::Copy(index) => {
                    self.dispatch(cx, ChatTask::CopyMessage(index));
                }
                MessagesAction::EditSave(index) => {
                    self.messages_ref().read_with(|m| {
                        let text = m.current_editor_text().expect("no editor text");
                        self.dispatch(cx, ChatTask::UpdateMessage(index, text));
                    });
                }
                MessagesAction::EditRegenerate(index) => {
                    self.messages_ref().read_with(|m| {
                        let mut messages = m.messages[0..=index].to_vec();

                        let index = m.current_editor_index().expect("no editor index");
                        let text = m.current_editor_text().expect("no editor text");

                        messages[index].body = text;
                        self.dispatch(
                            cx,
                            ChatTask::Compose(vec![
                                ChatTask::SetMessages(messages),
                                ChatTask::Send,
                            ]),
                        );
                    });
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
                    EntityId::User,
                    text.clone(),
                ));
            }

            composition.push(ChatTask::Send);

            self.dispatch(cx, ChatTask::Compose(composition));
        } else if prompt.read().has_stop_task() {
            self.dispatch(cx, ChatTask::Stop);
        }
    }

    fn perform_send(&mut self, cx: &mut Cx) {
        self.prompt_input_ref().write().reset(cx); // `reset` comes from command text input.

        // TODO: Less aggresive error handling for users.
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
            });

            messages.scroll_to_bottom();

            messages
                .messages
                .iter()
                .filter(|m| !m.is_writing && m.from != EntityId::App)
                .cloned()
                .collect()
        });

        self.prompt_input_ref().write().set_stop();
        self.redraw(cx);

        let ui = self.ui_runner();
        let future = async move {
            let mut client = repo.client();
            let mut message_stream = client.send_stream(&bot_id, &context);

            while let Some(delta) = message_stream.next().await {
                let delta = delta.unwrap_or_else(|_| "An error occurred".to_string());

                ui.defer_with_redraw(move |me, _cx, _scope| {
                    me.messages_ref().write_with(|messages| {
                        messages
                            .messages
                            .last_mut()
                            .expect("no message where to put delta")
                            .body
                            .push_str(&delta);

                        if messages.is_at_bottom() {
                            messages.scroll_to_bottom();
                        }
                    });
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

    /// Dispatch a task to be executed by the [Chat] widget.
    ///
    /// You can still hook into the task before it's executed for real, see [Chat::hook].
    pub fn dispatch(&self, cx: &mut Cx, task: ChatTask) {
        let action = ChatAction {
            hook: RwLock::new(ChatHook {
                executed: false,
                task: Some(task),
            }),
            widget_uid: self.widget_uid(),
        };

        cx.action(action);
    }

    pub fn tasks<'e>(&self, event: &'e Event) -> ChatTaskReader<'e> {
        ChatTaskReader::new(self.widget_uid(), event)
    }

    pub fn hook<'e>(&self, event: &'e Event) -> ChatHookWriter<'e> {
        ChatHookWriter::new(self.widget_uid(), event)
    }

    fn handle_tasks(&mut self, cx: &mut Cx, event: &Event) {
        self.hook(event).write(|hook| {
            hook.executed = true;
            if let Some(task) = hook.task.as_mut() {
                match task {
                    ChatTask::Compose(tasks) => {
                        for task in tasks {
                            self.handle_primitive_task(cx, task);
                        }
                    }
                    task => self.handle_primitive_task(cx, task),
                }
            }
        });
    }

    fn handle_primitive_task(&mut self, cx: &mut Cx, task: &ChatTask) {
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
            ChatTask::InsertMessage(index, entity, text) => {
                self.messages_ref().write().messages.insert(
                    *index,
                    Message {
                        from: entity.clone(),
                        body: text.clone(),
                        is_writing: false,
                    },
                );
                self.redraw(cx);
            }
            ChatTask::Send => {
                self.perform_send(cx);
            }
            ChatTask::Stop => {
                self.perform_stop(cx);
            }
            ChatTask::UpdateMessage(index, text) => {
                self.messages_ref().write_with(|m| {
                    m.messages[*index].body = text.clone();
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
            ChatTask::Compose(_) => {
                panic!("Developer error: Compose tasks should have been expanded before being handled.");
            }
        }
    }
}

fn chat_actions<'e>(
    widget_uid: WidgetUid,
    event: &'e Event,
) -> impl Iterator<Item = &'e ChatAction> + 'e {
    event
        .actions()
        .iter()
        .filter_map(|a| a.downcast_ref::<ChatAction>())
        .filter(move |a| a.widget_uid == widget_uid)
}

impl ChatRef {
    /// Immutable access to the underlying [[Chat]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<Chat> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [[Chat]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<Chat> {
        self.borrow_mut().unwrap()
    }

    /// Immutable reader to the underlying [[Chat]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read_with<R>(&self, f: impl FnOnce(&Chat) -> R) -> R {
        f(&*self.read())
    }

    /// Mutable writer to the underlying [[Chat]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write_with<R>(&mut self, f: impl FnOnce(&mut Chat) -> R) -> R {
        f(&mut *self.write())
    }
}
