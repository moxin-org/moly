use makepad_widgets::*;
use std::cell::{Ref, RefMut};
use std::sync::{Arc, Mutex};

use crate::controllers::chat::{
    ChatController, ChatControllerPlugin, ChatControllerPluginRegistrationId, ChatState,
    ChatStateMutation, ChatTask,
};
use crate::mcp::mcp_manager::display_name_from_namespaced;
use crate::utils::makepad::EventExt;
use crate::utils::vec::VecMutation;
use crate::widgets::moly_modal::MolyModalWidgetExt;
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

/// A batteries-included chat to to implement chatbots.
#[derive(Live, LiveHook, Widget)]
pub struct Chat {
    #[deref]
    deref: View,

    #[rust]
    chat_controller: Option<Arc<Mutex<ChatController>>>,

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
    plugin_id: Option<ChatControllerPluginRegistrationId>,
}

impl Widget for Chat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // for action in event.actions() {
        //     dbg!(action);
        // }

        // Pass down the BotContext if not the same.
        let self_chat_controller_ptr = self.chat_controller.as_ref().map(|c| Arc::as_ptr(c));
        let messages_chat_controller_ptr = self
            .messages_ref()
            .read()
            .chat_controller
            .as_ref()
            .map(|c| Arc::as_ptr(c));

        if self_chat_controller_ptr != messages_chat_controller_ptr {
            self.messages_ref().write().chat_controller = self.chat_controller.clone();
        }

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        self.handle_messages(cx, event);
        self.handle_prompt_input(cx, event);
        self.handle_realtime(cx);
        self.handle_modal_dismissal(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        println!(
            "Chat draw_walk ({})",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

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

    fn handle_realtime(&mut self, _cx: &mut Cx) {
        if self.realtime(id!(realtime)).connection_requested() {
            let bot_id = self.bot_id.clone().expect("no bot selected");

            self.chat_controller
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .dispatch_task(ChatTask::Send(bot_id));
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
                let chat_controller = self.chat_controller.clone().unwrap();

                // Get current messages and append the new conversation messages
                let mut all_messages = chat_controller.lock().unwrap().state().messages.clone();

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
                chat_controller
                    .lock()
                    .unwrap()
                    .dispatch_mutation(VecMutation::Set(all_messages));

                self.messages_ref().write().instant_scroll_to_bottom(cx);
            }
        }
    }

    fn handle_capabilities(&mut self, cx: &mut Cx) {
        let capabilities = self.chat_controller.as_ref().and_then(|controller| {
            self.bot_id.as_ref().and_then(|bot_id| {
                controller
                    .lock()
                    .unwrap()
                    .state()
                    .get_bot(bot_id)
                    .map(|bot| bot.capabilities.clone())
            })
        });

        self.prompt_input_ref()
            .write()
            .set_bot_capabilities(cx, capabilities);
    }

    fn handle_messages(&mut self, cx: &mut Cx, event: &Event) {
        for action in event.actions() {
            let Some(action) = action.as_widget_action() else {
                continue;
            };

            if action.widget_uid != self.messages_ref().widget_uid() {
                continue;
            }

            let chat_controller = self.chat_controller.clone().unwrap();

            match action.cast::<MessagesAction>() {
                MessagesAction::Delete(index) => chat_controller
                    .lock()
                    .unwrap()
                    .dispatch_mutation(VecMutation::<Message>::RemoveOne(index)),
                MessagesAction::Copy(index) => {
                    let lock = chat_controller.lock().unwrap();
                    let text = &lock.state().messages[index].content.text;
                    cx.copy_to_clipboard(text);
                }
                MessagesAction::EditSave(index) => {
                    let text = self
                        .messages_ref()
                        .read()
                        .current_editor_text()
                        .expect("no editor text");

                    self.messages_ref()
                        .write()
                        .set_message_editor_visibility(index, false);

                    let mut lock = chat_controller.lock().unwrap();

                    let mutation =
                        VecMutation::update_with(&lock.state().messages, index, |message| {
                            message.update_content(move |content| {
                                content.text = text;
                            });
                        });

                    lock.dispatch_mutation(mutation);
                }
                MessagesAction::EditRegenerate(index) => {
                    let mut messages =
                        chat_controller.lock().unwrap().state().messages[0..=index].to_vec();

                    let text = self
                        .messages_ref()
                        .read()
                        .current_editor_text()
                        .expect("no editor text");

                    self.messages_ref()
                        .write()
                        .set_message_editor_visibility(index, false);

                    messages[index].update_content(|content| {
                        content.text = text;
                    });

                    chat_controller
                        .lock()
                        .unwrap()
                        .dispatch_mutation(VecMutation::Set(messages));

                    let bot_id = self.bot_id.clone().expect("no bot selected");

                    chat_controller
                        .lock()
                        .unwrap()
                        .dispatch_task(ChatTask::Send(bot_id));
                }
                MessagesAction::ToolApprove(index) => {
                    let mut lock = chat_controller.lock().unwrap();

                    let mut updated_message = lock.state().messages[index].clone();

                    for tool_call in &mut updated_message.content.tool_calls {
                        tool_call.permission_status = ToolCallPermissionStatus::Approved;
                    }

                    lock.dispatch_mutation(VecMutation::Update(index, updated_message));

                    let tools = lock.state().messages[index].content.tool_calls.clone();
                    lock.dispatch_task(ChatTask::Execute(tools, self.bot_id.clone()));
                }
                MessagesAction::ToolDeny(index) => {
                    let mut lock = chat_controller.lock().unwrap();

                    let mut updated_message = lock.state().messages[index].clone();

                    updated_message.update_content(|content| {
                        for tool_call in &mut content.tool_calls {
                            tool_call.permission_status = ToolCallPermissionStatus::Denied;
                        }
                    });

                    lock.dispatch_mutation(VecMutation::Update(index, updated_message));

                    // Create synthetic tool results indicating denial to maintain conversation flow
                    let tool_results: Vec<ToolResult> = lock.state().messages[index]
                        .content
                        .tool_calls
                        .iter()
                        .map(|tc| {
                            let display_name = display_name_from_namespaced(&tc.name);
                            ToolResult {
                                tool_call_id: tc.id.clone(),
                                content: format!(
                                    "Tool execution was denied by the user. Tool '{}' was not executed.",
                                    display_name
                                ),
                                is_error: true,
                            }
                        })
                        .collect();

                    // Add tool result message with denial results
                    lock.dispatch_mutation(VecMutation::Push(Message {
                        from: EntityId::Tool,
                        content: MessageContent {
                            text: "🚫 Tool execution was denied by the user.".to_string(),
                            tool_results,
                            ..Default::default()
                        },
                        ..Default::default()
                    }));
                }
                MessagesAction::None => {}
            }
        }
    }

    fn handle_submit(&mut self, cx: &mut Cx) {
        let mut prompt = self.prompt_input_ref();
        let chat_controller = self.chat_controller.clone().unwrap();

        if prompt.read().has_send_task() {
            let bot_id = self.bot_id.clone().expect("no bot selected");

            let text = prompt.text();
            let attachments = prompt
                .read()
                .attachment_list_ref()
                .read()
                .attachments
                .clone();

            if !text.is_empty() || !attachments.is_empty() {
                chat_controller
                    .lock()
                    .unwrap()
                    .dispatch_mutation(VecMutation::Push(Message {
                        from: EntityId::User,
                        content: MessageContent {
                            text,
                            attachments,
                            ..Default::default()
                        },
                        ..Default::default()
                    }));
            }

            prompt.write().reset(cx);
            chat_controller
                .lock()
                .unwrap()
                .dispatch_task(ChatTask::Send(bot_id));
        } else if prompt.read().has_stop_task() {
            chat_controller
                .lock()
                .unwrap()
                .dispatch_task(ChatTask::Stop);
        }
    }

    fn handle_call(&mut self, _cx: &mut Cx) {
        // Use the standard send mechanism which will return the upgrade
        // The upgrade message will be processed in handle_message_delta
        self.chat_controller
            .as_mut()
            .unwrap()
            .lock()
            .unwrap()
            .dispatch_task(ChatTask::Send(
                self.bot_id.clone().expect("no bot selected"),
            ));
    }

    // fn handle_tool_calls(
    //     &mut self,
    //     _cx: &mut Cx,
    //     tool_calls: Vec<ToolCall>,
    //     loading_message_index: usize,
    // ) {
    //     let context = self
    //         .bot_context
    //         .as_ref()
    //         .expect("no BotContext provided")
    //         .clone();

    //     let ui = self.ui_runner();
    //     let future = async move {
    //         // Get the tool manager from context
    //         let Some(tool_manager) = context.tool_manager() else {
    //             ui.defer_with_redraw(move |me, cx, _| {
    //                 let error_message = Message {
    //                     from: EntityId::Tool,
    //                     content: MessageContent {
    //                         text: "Tool execution failed: Tool manager not available".to_string(),
    //                         ..Default::default()
    //                     },
    //                     metadata: MessageMetadata {
    //                         is_writing: false,
    //                         ..MessageMetadata::new()
    //                     },
    //                     ..Default::default()
    //                 };
    //                 me.dispatch(
    //                     cx,
    //                     &mut vec![ChatTask::UpdateMessage(
    //                         loading_message_index,
    //                         error_message,
    //                     )],
    //                 );
    //             });
    //             return;
    //         };

    //         // Execute tool calls using MCP manager
    //         let tool_results = tool_manager.execute_tool_calls(tool_calls.clone()).await;

    //         // Update the loading message with tool results and trigger a new send
    //         ui.defer_with_redraw(move |me, cx, _| {
    //             // Create formatted text for tool results
    //             let results_text = if tool_results.len() == 1 {
    //                 let result = &tool_results[0];
    //                 let tool_name = tool_calls
    //                     .iter()
    //                     .find(|tc| tc.id == result.tool_call_id)
    //                     .map(|tc| tc.name.as_str())
    //                     .unwrap_or("unknown");

    //                 let display_name = display_name_from_namespaced(tool_name);
    //                 if result.is_error {
    //                     format!("🔧 Tool '{}' failed:\n{}", display_name, result.content)
    //                 } else {
    //                     let summary = crate::utils::tool_execution::create_tool_output_summary(
    //                         tool_name,
    //                         &result.content,
    //                     );
    //                     format!(
    //                         "🔧 Tool '{}' executed successfully:\n`{}`",
    //                         display_name, summary
    //                     )
    //                 }
    //             } else {
    //                 let mut text = format!("🔧 Executed {} tools:\n\n", tool_results.len());
    //                 for result in &tool_results {
    //                     let tool_name = tool_calls
    //                         .iter()
    //                         .find(|tc| tc.id == result.tool_call_id)
    //                         .map(|tc| tc.name.as_str())
    //                         .unwrap_or("unknown");

    //                     let display_name = display_name_from_namespaced(tool_name);
    //                     if result.is_error {
    //                         text.push_str(&format!(
    //                             "**{}** ❌: {}\n\n",
    //                             display_name, result.content
    //                         ));
    //                     } else {
    //                         let summary = crate::utils::tool_execution::create_tool_output_summary(
    //                             tool_name,
    //                             &result.content,
    //                         );
    //                         text.push_str(&format!("**{}** ✅: `{}`\n\n", display_name, summary));
    //                     }
    //                 }
    //                 text
    //             };

    //             // Update the existing loading message with tool results
    //             let updated_message = Message {
    //                 from: EntityId::Tool, // Tool results use the tool role
    //                 content: MessageContent {
    //                     text: results_text,
    //                     tool_results,
    //                     ..Default::default()
    //                 },
    //                 metadata: MessageMetadata {
    //                     is_writing: false, // No longer loading
    //                     ..MessageMetadata::new()
    //                 },
    //                 ..Default::default()
    //             };

    //             me.dispatch(
    //                 cx,
    //                 &mut vec![
    //                     ChatTask::UpdateMessage(loading_message_index, updated_message),
    //                     ChatTask::Send, // Trigger a new send with the tool results
    //                 ],
    //             );
    //         });
    //     };

    //     spawn(future);
    // }

    // fn handle_task(&mut self, cx: &mut Cx, task: &ChatTask) {
    //     match task {
    //         // ChatTask::UpdateMessage(index, message) => {
    //         //     self.messages_ref().write_with(|m| {
    //         //         let new_message = message.clone();
    //         //         let old_message = m.messages.get_mut(*index).expect("no message at index");

    //         //         *old_message = new_message;
    //         //         m.set_message_editor_visibility(*index, false);
    //         //     });

    //         //     self.redraw(cx);
    //         // }
    //         // ChatTask::SetMessages(messages) => {
    //         //     self.messages_ref().write_with(|m| {
    //         //         m.messages = messages.clone();

    //         //         if let Some(index) = m.current_editor_index() {
    //         //             m.set_message_editor_visibility(index, false);
    //         //         }
    //         //     });

    //         //     self.redraw(cx);
    //         // }
    //         // ChatTask::ScrollToBottom(triggered_by_stream) => {
    //         //     self.messages_ref()
    //         //         .write()
    //         //         .scroll_to_bottom(cx, *triggered_by_stream);
    //         // }
    //         ChatTask::ApproveToolCalls(index) => {
    //             // Get the tool calls from the message and mark them as approved
    //             let mut message_updated = None;
    //             let tool_calls = self.messages_ref().write_with(|m| {
    //                 if let Some(message) = m.messages.get_mut(*index) {
    //                     message.update_content(|content| {
    //                         for tool_call in &mut content.tool_calls {
    //                             tool_call.permission_status = ToolCallPermissionStatus::Approved;
    //                         }
    //                     });
    //                     message_updated = Some(message.clone());
    //                     message.content.tool_calls.clone()
    //                 } else {
    //                     Vec::new()
    //                 }
    //             });

    //             if let Some(message) = message_updated {
    //                 self.dispatch(cx, &mut vec![ChatTask::UpdateMessage(*index, message)]);
    //             }

    //             if !tool_calls.is_empty() {
    //                 // Add immediate system message with loading state
    //                 let next_index = self.messages_ref().read().messages.len();
    //                 let loading_text = if tool_calls.len() == 1 {
    //                     format!(
    //                         "Executing tool '{}'...",
    //                         display_name_from_namespaced(&tool_calls[0].name)
    //                     )
    //                 } else {
    //                     format!("Executing {} tools...", tool_calls.len())
    //                 };

    //                 let loading_message = Message {
    //                     from: EntityId::Tool,
    //                     content: MessageContent {
    //                         text: loading_text,
    //                         ..Default::default()
    //                     },
    //                     metadata: MessageMetadata {
    //                         is_writing: true,
    //                         ..MessageMetadata::new()
    //                     },
    //                     ..Default::default()
    //                 };

    //                 self.dispatch(
    //                     cx,
    //                     &mut vec![ChatTask::InsertMessage(next_index, loading_message)],
    //                 );

    //                 self.handle_tool_calls(cx, tool_calls, next_index);
    //             } else {
    //                 ::log::error!("No tool calls found at index: {}", index);
    //             }

    //             self.redraw(cx);
    //         }
    //         ChatTask::DenyToolCalls(index) => {
    //             // Get the tool calls from the message and mark them as denied
    //             let mut message_updated = None;
    //             let tool_calls = self.messages_ref().write_with(|m| {
    //                 if let Some(message) = m.messages.get_mut(*index) {
    //                     message.update_content(|content| {
    //                         for tool_call in &mut content.tool_calls {
    //                             tool_call.permission_status = ToolCallPermissionStatus::Denied;
    //                         }
    //                     });
    //                     message_updated = Some(message.clone());
    //                     message.content.tool_calls.clone()
    //                 } else {
    //                     Vec::new()
    //                 }
    //             });

    //             if let Some(message) = message_updated {
    //                 self.dispatch(cx, &mut vec![ChatTask::UpdateMessage(*index, message)]);
    //             }

    //             if !tool_calls.is_empty() {
    //                 // Create synthetic tool results indicating denial to maintain conversation flow
    //                 let tool_results: Vec<ToolResult> = tool_calls.iter().map(|tc| {
    //                     let display_name = display_name_from_namespaced(&tc.name);
    //                     ToolResult {
    //                         tool_call_id: tc.id.clone(),
    //                         content: format!("Tool execution was denied by the user. Tool '{}' was not executed.", display_name),
    //                         is_error: true,
    //                     }
    //                 }).collect();

    //                 let next_index = self.messages_ref().read().messages.len();

    //                 // Add tool result message with denial results
    //                 let tool_message = Message {
    //                     from: EntityId::Tool,
    //                     content: MessageContent {
    //                         text: "🚫 Tool execution was denied by the user.".to_string(),
    //                         tool_results,
    //                         ..Default::default()
    //                     },
    //                     ..Default::default()
    //                 };

    //                 // Continue the conversation with the denial results
    //                 self.dispatch(
    //                     cx,
    //                     &mut vec![
    //                         ChatTask::InsertMessage(next_index, tool_message),
    //                         ChatTask::Send,
    //                     ],
    //                 );
    //             }

    //             self.redraw(cx);
    //         }
    //     }
    // }

    /// Handles a message delta from the bot.
    ///
    /// Returns true if the message delta was handled successfully.
    // fn handle_message_delta(&mut self, cx: &mut Cx, result: ClientResult<MessageContent>) -> bool {
    //     let messages = self.messages_ref();

    //     // For simplicity, lets handle this as an standard Result, ignoring delta
    //     // if there are errors.
    //     match result.into_result() {
    //         Ok(content) => {
    //             // Check if this is a realtime upgrade message
    //             if let Some(Upgrade::Realtime(channel)) = &content.upgrade {
    //                 // Clean up any loading state since we're opening the modal instead
    //                 self.clean_streaming_artifacts();

    //                 // Set up the realtime channel in the UI
    //                 let mut realtime = self.realtime(id!(realtime));
    //                 realtime.set_bot_entity_id(
    //                     cx,
    //                     EntityId::Bot(self.bot_id.clone().unwrap_or_default()),
    //                 );
    //                 realtime.set_realtime_channel(channel.clone());
    //                 realtime.set_bot_context(self.bot_context.clone());

    //                 let modal = self.moly_modal(id!(audio_modal));
    //                 modal.open(cx);

    //                 // Skip the rest, do not add a message to the chat
    //                 return true;
    //             }

    //             // Let's abort if we don't have where to put the delta.
    //             let Some(mut message) = messages.read().messages.last().cloned() else {
    //                 return true;
    //             };

    //             // Let's abort if we see we are putting delta in the wrong message.
    //             if let Some(expected_message) = self.expected_message.as_ref() {
    //                 if message.from != expected_message.from
    //                     || message.content != expected_message.content
    //                     || message.metadata.is_writing != expected_message.metadata.is_writing
    //                     || message.metadata.created_at != expected_message.metadata.created_at
    //                 {
    //                     log!("Unexpected message to put delta in. Stopping.");
    //                     return true;
    //                 }
    //             }

    //             message.set_content(content);

    //             let index = messages.read().messages.len() - 1;
    //             let mut tasks = vec![ChatTask::UpdateMessage(index, message.clone())];

    //             // Stick the chat to the bottom if the user didn't manually scroll.
    //             if !self.user_scrolled_during_stream {
    //                 tasks.push(ChatTask::ScrollToBottom(true));
    //             }

    //             self.dispatch(cx, &mut tasks);

    //             let Some(ChatTask::UpdateMessage(_, updated_message)) = tasks.into_iter().next()
    //             else {
    //                 // Let's abort if the tasks were modified in an unexpected way.
    //                 return true;
    //             };

    //             if !updated_message.content.tool_calls.is_empty() {
    //                 // Mark message as not writing since tool calls are complete
    //                 let mut final_message = updated_message.clone();
    //                 final_message.metadata.is_writing = false;
    //                 self.dispatch(cx, &mut vec![ChatTask::UpdateMessage(index, final_message)]);
    //                 // TODO: We might want to dispatch a ChatTask::RequestToolPermission(index) here
    //                 // Check if dangerous mode is enabled to auto-approve tool calls
    //                 let dangerous_mode_enabled = self
    //                     .bot_context
    //                     .as_ref()
    //                     .map(|ctx| {
    //                         ctx.tool_manager()
    //                             .map(|tm| tm.get_dangerous_mode_enabled())
    //                             .unwrap_or(false)
    //                     })
    //                     .unwrap_or(false);

    //                 if dangerous_mode_enabled {
    //                     // Auto-approve tool calls in dangerous mode
    //                     self.dispatch(cx, &mut vec![ChatTask::ApproveToolCalls(index)]);
    //                 }

    //                 // Signal to stop the current stream since we're switching to tool execution
    //                 return true;
    //             }

    //             self.expected_message = Some(updated_message);

    //             false
    //         }
    //         Err(errors) => {
    //             let mut tasks = errors
    //                 .into_iter()
    //                 .enumerate()
    //                 .map(|(i, e)| {
    //                     ChatTask::InsertMessage(
    //                         messages.read().messages.len() + i,
    //                         Message {
    //                             from: EntityId::App,
    //                             content: MessageContent {
    //                                 text: e.to_string(),
    //                                 ..Default::default()
    //                             },
    //                             ..Default::default()
    //                         },
    //                     )
    //                 })
    //                 .collect::<Vec<_>>();

    //             // Stick the chat to the bottom if the user didn't manually scroll.
    //             if !self.user_scrolled_during_stream {
    //                 tasks.push(ChatTask::ScrollToBottom(true));
    //             }

    //             self.dispatch(cx, &mut tasks);
    //             true
    //         }
    //     }
    // }

    /// Returns true if the chat is currently streaming.
    pub fn is_streaming(&self) -> bool {
        self.chat_controller
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .state()
            .is_streaming
    }

    pub fn set_bot_id(&mut self, cx: &mut Cx, bot_id: Option<BotId>) {
        self.bot_id = bot_id;
        self.handle_capabilities(cx);
    }

    pub fn bot_id(&self) -> Option<&BotId> {
        self.bot_id.as_ref()
    }

    pub fn set_chat_controller(
        &mut self,
        _cx: &mut Cx,
        chat_controller: Option<Arc<Mutex<ChatController>>>,
    ) {
        self.unlink_current_controller();

        self.chat_controller = chat_controller;

        if let Some(controller) = self.chat_controller.as_ref() {
            let mut guard = controller.lock().unwrap();

            let plugin = Plugin {
                ui: self.ui_runner(),
                relevant_mutations: vec![],
            };
            self.plugin_id = Some(guard.append_plugin(plugin));

            // maybe not good idea to trigger a load implicitly
            // if guard.state().load_status == Status::Idle {
            //     guard.dispatch_task(ChatTask::Load);
            // }
        }

        // TODO: Probably doesn't make sense.
        // self.handle_capabilities(cx);
    }

    pub fn chat_controller(&self) -> Option<&Arc<Mutex<ChatController>>> {
        self.chat_controller.as_ref()
    }

    fn unlink_current_controller(&mut self) {
        if let Some(plugin_id) = self.plugin_id {
            if let Some(controller) = self.chat_controller.as_ref() {
                controller.lock().unwrap().remove_plugin(plugin_id);
            }
        }

        self.chat_controller = None;
        self.plugin_id = None;
    }

    fn on_relevant_chat_controller_mutation(&mut self, cx: &mut Cx, mutation: ChatStateMutation) {
        match mutation {
            ChatStateMutation::SetIsStreaming(true) => {
                self.handle_streaming_start(cx);
            }
            ChatStateMutation::SetIsStreaming(false) => {
                self.handle_streaming_end(cx);
            }
            _ => {}
        }
    }

    fn handle_streaming_start(&mut self, cx: &mut Cx) {
        self.prompt_input_ref().write().set_stop();
        self.messages_ref().write().animated_scroll_to_bottom(cx);
        self.redraw(cx);
    }

    fn handle_streaming_end(&mut self, cx: &mut Cx) {
        self.prompt_input_ref().write().set_send();
        self.redraw(cx);
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

impl Drop for Chat {
    fn drop(&mut self) {
        self.unlink_current_controller();
    }
}

struct Plugin {
    ui: UiRunner<Chat>,
    relevant_mutations: Vec<ChatStateMutation>,
}

impl ChatControllerPlugin for Plugin {
    fn on_state_ready(&mut self, _state: &ChatState) {
        for mutation in self.relevant_mutations.drain(..) {
            self.ui.defer_with_redraw(move |me, cx, _| {
                me.on_relevant_chat_controller_mutation(cx, mutation);
            });
        }

        // Always redraw on state change.
        self.ui.defer_with_redraw(move |_, _, _| {});
    }

    fn on_state_mutation(&mut self, _state: &ChatState, mutation: &ChatStateMutation) {
        let is_relevant = match mutation {
            ChatStateMutation::SetIsStreaming(_) => true,
            _ => false,
        };

        if is_relevant {
            self.relevant_mutations.push(mutation.clone());
        }
    }

    fn on_upgrade(&mut self, upgrade: Upgrade, bot_id: &BotId) -> Option<Upgrade> {
        match upgrade {
            Upgrade::Realtime(channel) => {
                let entity_id = EntityId::Bot(bot_id.clone());
                self.ui.defer(move |me, cx, _| {
                    me.handle_streaming_end(cx);

                    // Set up the realtime channel in the UI
                    let mut realtime = me.realtime(id!(realtime));
                    realtime.set_bot_entity_id(cx, entity_id);
                    realtime.set_realtime_channel(channel.clone());

                    let modal = me.moly_modal(id!(audio_modal));
                    modal.open(cx);
                });
                None
            }
            #[allow(unreachable_patterns)]
            upgrade => Some(upgrade),
        }
    }
}
