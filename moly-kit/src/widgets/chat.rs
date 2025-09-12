use futures::{StreamExt, stream::AbortHandle};
use makepad_widgets::*;
use std::cell::{Ref, RefMut};
use utils::asynchronous::spawn;

use crate::utils::asynchronous::PlatformSendStream;
use crate::utils::makepad::EventExt;
use crate::utils::ui_runner::DeferWithRedrawAsync;
use crate::widgets::moly_modal::MolyModalWidgetExt;

use crate::mcp::mcp_manager::display_name_from_namespaced;
use crate::*;

live_design!(
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;
    use link::shaders::*;

    use crate::widgets::messages::*;
    use crate::widgets::prompt_input::*;
    use crate::widgets::moly_modal::*;
    use crate::widgets::realtime::*;

    pub Chat = {{Chat}} <RoundedView> {
        flow: Down,
        messages = <Messages> {}
        prompt = <PromptInput> {}

        <View> {
            width: Fill, height: Fit
            flow: Overlay

            audio_modal = <MolyModal> {
                dismiss_on_focus_lost: false
                content: <RealtimeContent> {}
            }
        }
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
#[derive(Debug, Clone, PartialEq)]
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

    /// When received back, it will approve and execute the tool calls in the message at the given index.
    ApproveToolCalls(usize),

    /// When received back, it will deny the tool calls in the message at the given index.
    DenyToolCalls(usize),
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

    /// The [BotContext] used by this chat to hold bots and interact with them.
    #[rust]
    bot_context: Option<BotContext>,

    /// The id of the bot the chat will message when sending.
    // TODO: Can this be live?
    // TODO: Default to the first bot in [BotContext] if `None`.
    #[rust]
    bot_id: Option<BotId>,

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

    /// Tasks queued during dispatch to avoid nested dispatch calls
    #[rust]
    pending_tasks: Vec<ChatTask>,
}

impl Widget for Chat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Pass down the BotContext if not the same.
        self.messages_ref().write_with(|m| {
            if m.bot_context != self.bot_context {
                m.bot_context = self.bot_context.clone();
            }
        });

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);
        self.handle_messages(cx, event);
        self.handle_prompt_input(cx, event);
        self.handle_realtime(cx);
        self.handle_modal_dismissal(cx, event);
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

        if self.prompt_input_ref().read().call_pressed(event.actions()) {
            self.handle_call(cx);
        }
    }

    fn handle_realtime(&mut self, cx: &mut Cx) {
        if self.realtime(id!(realtime)).connection_requested() {
            self.dispatch(cx, &mut ChatTask::Send.into());
        }
    }

    fn handle_modal_dismissal(&mut self, cx: &mut Cx, event: &Event) {
        // Check if the modal should be dismissed
        for action in event.actions() {
            if let RealtimeModalAction::DismissModal = action.cast() {
                self.moly_modal(id!(audio_modal)).close(cx);
            }
        }

        // Check if the audio modal was dismissed
        if self.moly_modal(id!(audio_modal)).dismissed(event.actions()) {
            // Collect conversation messages from the realtime widget before resetting
            let mut conversation_messages =
                self.realtime(id!(realtime)).take_conversation_messages();

            // Reset realtime widget state for cleanup
            self.realtime(id!(realtime)).reset_state(cx);

            // Add conversation messages to chat history preserving order
            if !conversation_messages.is_empty() {
                // Get current messages and append the new conversation messages
                let mut all_messages = self.messages_ref().read().messages.clone();

                // Add a system message before and after the conversation, informing
                // that a voice call happened.
                let system_message = Message {
                    from: EntityId::App,
                    content: MessageContent {
                        text: "Voice call started.".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                conversation_messages.insert(0, system_message);

                let system_message = Message {
                    from: EntityId::App,
                    content: MessageContent {
                        text: "Voice call ended.".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                conversation_messages.push(system_message);

                all_messages.extend(conversation_messages);
                self.dispatch(
                    cx,
                    &mut vec![
                        ChatTask::SetMessages(all_messages),
                        ChatTask::ScrollToBottom(true),
                    ],
                );
            }
        }
    }

    fn handle_capabilities(&mut self, cx: &mut Cx) {
        if let (Some(bot_context), Some(bot_id)) = (&self.bot_context, &self.bot_id) {
            if let Some(bot) = bot_context.get_bot(bot_id) {
                self.prompt_input_ref()
                    .write()
                    .set_bot_capabilities(cx, Some(bot.capabilities.clone()));
            } else if self.bot_id.is_none() {
                self.prompt_input_ref()
                    .write()
                    .set_bot_capabilities(cx, None);
            }
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
                        message.update_content(|content| {
                            content.text = m.current_editor_text().expect("no editor text");
                        });
                        ChatTask::UpdateMessage(index, message).into()
                    });

                    self.dispatch(cx, &mut tasks);
                }
                MessagesAction::EditRegenerate(index) => {
                    let mut tasks = self.messages_ref().read_with(|m| {
                        let mut messages = m.messages[0..=index].to_vec();

                        let index = m.current_editor_index().expect("no editor index");
                        let text = m.current_editor_text().expect("no editor text");

                        messages[index].update_content(|content| {
                            content.text = text;
                        });

                        vec![ChatTask::SetMessages(messages), ChatTask::Send]
                    });

                    self.dispatch(cx, &mut tasks);
                }
                MessagesAction::ToolApprove(index) => {
                    self.dispatch(cx, &mut ChatTask::ApproveToolCalls(index).into());
                }
                MessagesAction::ToolDeny(index) => {
                    self.dispatch(cx, &mut ChatTask::DenyToolCalls(index).into());
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
            let attachments = prompt
                .read()
                .attachment_list_ref()
                .read()
                .attachments
                .clone();
            let mut composition = Vec::new();

            if !text.is_empty() || !attachments.is_empty() {
                composition.push(ChatTask::InsertMessage(
                    next_index,
                    Message {
                        from: EntityId::User,
                        content: MessageContent {
                            text,
                            attachments,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ));
            }

            composition.extend([ChatTask::Send, ChatTask::ClearPrompt]);

            self.dispatch(cx, &mut composition);
        } else if prompt.read().has_stop_task() {
            self.dispatch(cx, &mut ChatTask::Stop.into());
        }
    }

    fn handle_call(&mut self, cx: &mut Cx) {
        // Use the standard send mechanism which will return the upgrade
        // The upgrade message will be processed in handle_message_delta
        self.dispatch(cx, &mut ChatTask::Send.into());
    }

    fn handle_tool_calls(
        &mut self,
        _cx: &mut Cx,
        tool_calls: Vec<ToolCall>,
        loading_message_index: usize,
    ) {
        let context = self
            .bot_context
            .as_ref()
            .expect("no BotContext provided")
            .clone();

        let ui = self.ui_runner();
        let future = async move {
            // Get the tool manager from context
            let Some(tool_manager) = context.tool_manager() else {
                ui.defer_with_redraw(move |me, cx, _| {
                    let error_message = Message {
                        from: EntityId::System,
                        content: MessageContent {
                            text: "Tool execution failed: Tool manager not available".to_string(),
                            ..Default::default()
                        },
                        metadata: MessageMetadata {
                            is_writing: false,
                            ..MessageMetadata::new()
                        },
                        ..Default::default()
                    };
                    me.dispatch(
                        cx,
                        &mut vec![ChatTask::UpdateMessage(
                            loading_message_index,
                            error_message,
                        )],
                    );
                });
                return;
            };

            // Execute tool calls using MCP manager
            let tool_results = tool_manager.execute_tool_calls(tool_calls.clone()).await;

            // Update the loading message with tool results and trigger a new send
            ui.defer_with_redraw(move |me, cx, _| {
                // Create formatted text for tool results
                let results_text = if tool_results.len() == 1 {
                    let result = &tool_results[0];
                    let tool_name = tool_calls
                        .iter()
                        .find(|tc| tc.id == result.tool_call_id)
                        .map(|tc| tc.name.as_str())
                        .unwrap_or("unknown");

                    let display_name = display_name_from_namespaced(tool_name);
                    if result.is_error {
                        format!("🔧 Tool '{}' failed:\n{}", display_name, result.content)
                    } else {
                        let summary = crate::utils::tool_execution::create_tool_output_summary(
                            tool_name,
                            &result.content,
                        );
                        format!(
                            "🔧 Tool '{}' executed successfully:\n{}",
                            display_name, summary
                        )
                    }
                } else {
                    let mut text = format!("🔧 Executed {} tools:\n\n", tool_results.len());
                    for result in &tool_results {
                        let tool_name = tool_calls
                            .iter()
                            .find(|tc| tc.id == result.tool_call_id)
                            .map(|tc| tc.name.as_str())
                            .unwrap_or("unknown");

                        let display_name = display_name_from_namespaced(tool_name);
                        if result.is_error {
                            text.push_str(&format!(
                                "**{}** ❌: {}\n\n",
                                display_name, result.content
                            ));
                        } else {
                            let summary = crate::utils::tool_execution::create_tool_output_summary(
                                tool_name,
                                &result.content,
                            );
                            text.push_str(&format!("**{}** ✅: {}\n\n", display_name, summary));
                        }
                    }
                    text
                };

                // Update the existing loading message with tool results
                let updated_message = Message {
                    from: EntityId::System, // Tool results are system messages
                    content: MessageContent {
                        text: results_text,
                        tool_results,
                        ..Default::default()
                    },
                    metadata: MessageMetadata {
                        is_writing: false, // No longer loading
                        ..MessageMetadata::new()
                    },
                    ..Default::default()
                };

                me.dispatch(
                    cx,
                    &mut vec![
                        ChatTask::UpdateMessage(loading_message_index, updated_message),
                        ChatTask::Send, // Trigger a new send with the tool results
                    ],
                );
            });
        };

        spawn(future);
    }

    fn handle_send_task(&mut self, cx: &mut Cx) {
        // Let's start clean before starting a new stream.
        self.abort();

        // TODO: See `bot_id` TODO.
        let bot_id = self.bot_id.clone().expect("no bot selected");

        let context = self
            .bot_context
            .as_ref()
            .expect("no BotContext provided")
            .clone();

        // First check if the bot exists in the BotContext.
        if context.get_bot(&bot_id).is_none() {
            // Bot not found, add error message
            let next_index = self.messages_ref().read().messages.len();
            let error_message = format!(
                "App error: Bot not found. The bot might have been disabled or removed. Bot ID: {}",
                bot_id
            );

            let message = Message {
                from: EntityId::App,
                content: MessageContent {
                    text: error_message,
                    ..Default::default()
                },
                ..Default::default()
            };

            self.dispatch(cx, &mut vec![ChatTask::InsertMessage(next_index, message)]);
            return;
        }

        let messages_history_context: Vec<Message> = self.messages_ref().write_with(|messages| {
            messages.bot_context = Some(context.clone());

            messages
                .messages
                .iter()
                .filter(|m| m.metadata.is_idle() && m.from != EntityId::App)
                .cloned()
                .collect()
        });

        // The realtime check is hack to avoid showing a loading message for realtime assistants
        // TODO: we should base this on upgrade rather than capabilities
        let bot = context.get_bot(&bot_id).unwrap(); // We already checked it exists above
        if !bot.capabilities.supports_realtime() {
            let loading_message = Message {
                from: EntityId::Bot(bot_id.clone()),
                metadata: MessageMetadata {
                    is_writing: true,
                    ..MessageMetadata::new()
                },
                ..Default::default()
            };

            let next_index = self.messages_ref().read().messages.len();
            self.dispatch(
                cx,
                &mut vec![ChatTask::InsertMessage(next_index, loading_message)],
            );
        }

        self.dispatch(cx, &mut vec![ChatTask::ScrollToBottom(false)]);
        self.prompt_input_ref().write().set_stop();
        self.redraw(cx);

        let ui = self.ui_runner();
        let future = async move {
            let mut client = context.client();
            let bot = match context.get_bot(&bot_id) {
                Some(bot) => bot,
                None => {
                    // This should never happen as we check above, but handle it gracefully anyway
                    let bot_id_clone = bot_id.clone(); // Clone the bot_id for the closure
                    ui.defer_with_redraw(move |me, cx, _| {
                        let error_message = format!(
                            "App error: Bot not found during stream initialization. Bot ID: {}",
                            bot_id_clone
                        );
                        let next_index = me.messages_ref().read().messages.len();
                        let message = Message {
                            from: EntityId::App,
                            content: MessageContent {
                                text: error_message,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        me.dispatch(cx, &mut vec![ChatTask::InsertMessage(next_index, message)]);
                    });
                    return;
                }
            };

            let tools = if let Some(tool_manager) = context.tool_manager() {
                tool_manager.get_all_namespaced_tools()
            } else {
                Vec::new()
            };

            let message_stream = amortize(client.send(&bot.id, &messages_history_context, &tools));
            let mut message_stream = std::pin::pin!(message_stream);
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
        // Prevent nested dispatch - queue tasks if we're already dispatching
        if self.is_hooking {
            self.pending_tasks.extend(tasks.iter().cloned());
            return;
        }

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

        // Process any pending tasks that were queued during execution
        if !self.pending_tasks.is_empty() {
            let mut pending = std::mem::take(&mut self.pending_tasks);
            self.dispatch(cx, &mut pending);
        }
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
                    let new_message = message.clone();
                    let old_message = m.messages.get_mut(*index).expect("no message at index");

                    *old_message = new_message;
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
                self.prompt_input_ref().write().reset(cx);
            }
            ChatTask::ScrollToBottom(triggered_by_stream) => {
                self.messages_ref()
                    .write()
                    .scroll_to_bottom(cx, *triggered_by_stream);
            }
            ChatTask::ApproveToolCalls(index) => {
                // Get the tool calls from the message and mark them as approved
                let mut message_updated = None;
                let tool_calls = self.messages_ref().write_with(|m| {
                    if let Some(message) = m.messages.get_mut(*index) {
                        message.update_content(|content| {
                            for tool_call in &mut content.tool_calls {
                                tool_call.permission_status = ToolCallPermissionStatus::Approved;
                            }
                        });
                        message_updated = Some(message.clone());
                        message.content.tool_calls.clone()
                    } else {
                        Vec::new()
                    }
                });

                if let Some(message) = message_updated {
                    self.dispatch(cx, &mut vec![ChatTask::UpdateMessage(*index, message)]);
                }

                if !tool_calls.is_empty() {
                    // Add immediate system message with loading state
                    let next_index = self.messages_ref().read().messages.len();
                    let loading_text = if tool_calls.len() == 1 {
                        format!(
                            "Executing tool '{}'...",
                            display_name_from_namespaced(&tool_calls[0].name)
                        )
                    } else {
                        format!("Executing {} tools...", tool_calls.len())
                    };

                    let loading_message = Message {
                        from: EntityId::System,
                        content: MessageContent {
                            text: loading_text,
                            ..Default::default()
                        },
                        metadata: MessageMetadata {
                            is_writing: true,
                            ..MessageMetadata::new()
                        },
                        ..Default::default()
                    };

                    self.dispatch(
                        cx,
                        &mut vec![ChatTask::InsertMessage(next_index, loading_message)],
                    );

                    self.handle_tool_calls(cx, tool_calls, next_index);
                } else {
                    ::log::error!("No tool calls found at index: {}", index);
                }

                self.redraw(cx);
            }
            ChatTask::DenyToolCalls(index) => {
                // Get the tool calls from the message and mark them as denied
                let mut message_updated = None;
                let tool_calls = self.messages_ref().write_with(|m| {
                    if let Some(message) = m.messages.get_mut(*index) {
                        message.update_content(|content| {
                            for tool_call in &mut content.tool_calls {
                                tool_call.permission_status = ToolCallPermissionStatus::Denied;
                            }
                        });
                        message_updated = Some(message.clone());
                        message.content.tool_calls.clone()
                    } else {
                        Vec::new()
                    }
                });

                if let Some(message) = message_updated {
                    self.dispatch(cx, &mut vec![ChatTask::UpdateMessage(*index, message)]);
                }

                if !tool_calls.is_empty() {
                    // Create synthetic tool results indicating denial to maintain conversation flow
                    let tool_results: Vec<ToolResult> = tool_calls.iter().map(|tc| {
                        let display_name = display_name_from_namespaced(&tc.name);
                        ToolResult {
                            tool_call_id: tc.id.clone(),
                            content: format!("Tool execution was denied by the user. Tool '{}' was not executed.", display_name),
                            is_error: true,
                        }
                    }).collect();

                    let next_index = self.messages_ref().read().messages.len();

                    // Add tool result message with denial results
                    let tool_message = Message {
                        from: EntityId::System,
                        content: MessageContent {
                            text: "🚫 Tool execution was denied by the user.".to_string(),
                            tool_results,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    // Continue the conversation with the denial results
                    self.dispatch(
                        cx,
                        &mut vec![
                            ChatTask::InsertMessage(next_index, tool_message),
                            ChatTask::Send,
                        ],
                    );
                }

                self.redraw(cx);
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
            m.metadata.is_writing = false;
            !m.content.is_empty()
        });
    }

    /// Handles a message delta from the bot.
    ///
    /// Returns true if the message delta was handled successfully.
    fn handle_message_delta(&mut self, cx: &mut Cx, result: ClientResult<MessageContent>) -> bool {
        let messages = self.messages_ref();

        // For simplicity, lets handle this as an standard Result, ignoring delta
        // if there are errors.
        match result.into_result() {
            Ok(content) => {
                // Check if this is a realtime upgrade message
                if let Some(Upgrade::Realtime(channel)) = &content.upgrade {
                    // Clean up any loading state since we're opening the modal instead
                    self.clean_streaming_artifacts();

                    // Set up the realtime channel in the UI
                    let mut realtime = self.realtime(id!(realtime));
                    realtime.set_realtime_channel(channel.clone());
                    realtime.set_bot_entity_id(
                        cx,
                        EntityId::Bot(self.bot_id.clone().unwrap_or_default()),
                    );
                    realtime.set_bot_context(self.bot_context.clone());

                    let modal = self.moly_modal(id!(audio_modal));
                    modal.open(cx);

                    // Skip the rest, do not add a message to the chat
                    return true;
                }

                // Let's abort if we don't have where to put the delta.
                let Some(mut message) = messages.read().messages.last().cloned() else {
                    return true;
                };

                // Let's abort if we see we are putting delta in the wrong message.
                if let Some(expected_message) = self.expected_message.as_ref() {
                    if message.from != expected_message.from
                        || message.content != expected_message.content
                        || message.metadata.is_writing != expected_message.metadata.is_writing
                        || message.metadata.created_at != expected_message.metadata.created_at
                    {
                        log!("Unexpected message to put delta in. Stopping.");
                        return true;
                    }
                }

                message.set_content(content);

                let index = messages.read().messages.len() - 1;
                let mut tasks = vec![ChatTask::UpdateMessage(index, message.clone())];

                // Stick the chat to the bottom if the user didn't manually scroll.
                if !self.user_scrolled_during_stream {
                    tasks.push(ChatTask::ScrollToBottom(true));
                }

                self.dispatch(cx, &mut tasks);

                let Some(ChatTask::UpdateMessage(_, updated_message)) = tasks.into_iter().next()
                else {
                    // Let's abort if the tasks were modified in an unexpected way.
                    return true;
                };

                if !updated_message.content.tool_calls.is_empty() {
                    // Mark message as not writing since tool calls are complete
                    let mut final_message = updated_message.clone();
                    final_message.metadata.is_writing = false;
                    self.dispatch(cx, &mut vec![ChatTask::UpdateMessage(index, final_message)]);
                    // TODO: We might want to dispatch a ChatTask::RequestToolPermission(index) here

                    // Signal to stop the current stream since we're switching to permission request
                    return true;
                }

                self.expected_message = Some(updated_message);

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
                                ..Default::default()
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
            message.metadata.is_writing
        } else {
            false
        }
    }

    pub fn set_bot_id(&mut self, cx: &mut Cx, bot_id: Option<BotId>) {
        self.bot_id = bot_id;
        self.handle_capabilities(cx);
    }

    pub fn bot_id(&self) -> Option<&BotId> {
        self.bot_id.as_ref()
    }

    pub fn set_bot_context(&mut self, cx: &mut Cx, bot_context: Option<BotContext>) {
        self.bot_context = bot_context;
        self.handle_capabilities(cx);
    }

    pub fn bot_context(&self) -> Option<&BotContext> {
        self.bot_context.as_ref()
    }
}

// TODO: Since `ChatRef` is generated by a macro, I can't document this to give
// these functions better visibility from the module view.
impl ChatRef {
    /// Immutable access to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<'_, Chat> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [Chat].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<'_, Chat> {
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

/// Util that wraps the stream of `send()` and gives you a stream less agresive to
/// the receiver UI regardless of the streaming chunk size.
fn amortize(
    input: impl PlatformSendStream<Item = ClientResult<MessageContent>> + 'static,
) -> impl PlatformSendStream<Item = ClientResult<MessageContent>> + 'static {
    // Use utils
    use crate::utils::string::AmortizedString;
    use async_stream::stream;

    // Stream state
    let mut amortized_text = AmortizedString::default();
    let mut amortized_reasoning = AmortizedString::default();

    // Stream compute
    stream! {
        // Our wrapper stream "activates" when something comes from the underlying stream.
        for await result in input {
            // Transparently yield the result on error and then stop.
            if result.has_errors() {
                yield result;
                return;
            }

            // Modified content that we will be yielding.
            let mut content = result.into_value().unwrap();

            // Feed the whole string into the string amortizer.
            // Put back what has been already amortized from previous iterations.
            let text = std::mem::take(&mut content.text);
            amortized_text.update(text);
            content.text = amortized_text.current().to_string();

            // Same for reasoning.
            let reasoning = std::mem::take(&mut content.reasoning);
            amortized_reasoning.update(reasoning);
            content.reasoning = amortized_reasoning.current().to_string();

            // Prioritize yielding amortized reasoning updates first.
            for reasoning in &mut amortized_reasoning {
                content.reasoning = reasoning;
                yield ClientResult::new_ok(content.clone());
            }

            // Finially, begin yielding amortized text updates.
            // This will also include the amortized reasoning until now because we
            // fed it back into the content.
            for text in &mut amortized_text {
                content.text = text;
                yield ClientResult::new_ok(content.clone());
            }
        }
    }
}
