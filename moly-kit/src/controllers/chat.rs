//! Framework-agnostic state management to implement a `Chat` component/widget/element.

use std::panic::Location;

use crate::{
    McpManagerClient, display_name_from_namespaced,
    protocol::*,
    utils::{
        asynchronous::{AbortOnDropHandle, PlatformSendStream, spawn_abort_on_drop},
        vec::VecMutation,
    },
};
use std::sync::{Arc, Mutex, Weak};

use futures::StreamExt;

mod plugin;
mod state;
mod task;

pub use plugin::*;
pub use state::*;
pub use task::*;

/// Private utility wrapper around a weak ref to a controller.
///
/// Motivation: Before spawning async tasks (or threads) you may be tempted to upgrade
/// the weak reference. If you do so, running futures will not be aborted when the controller
/// is not longer needed, because a strong reference will be kept alive by the
/// async task (or thread). This wrapper intends enforce upgrading only when needed.
#[derive(Default, Clone)]
struct ChatControllerAccessor {
    handle: Weak<Mutex<ChatController>>,
}

impl ChatControllerAccessor {
    fn lock_with<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut ChatController) -> R,
    {
        let handle = self.handle.upgrade()?;
        let mut controller = handle.lock().ok()?;
        Some(f(&mut controller))
    }

    fn new(handle: Weak<Mutex<ChatController>>) -> Self {
        Self { handle }
    }
}

/// State management abstraction specialized in handling the core complex chat logic.
///
/// This follow a controller-like interface to be part of an MVC-like architecture.
/// The controller receives different inputs, which causes work to happen inside, and
/// finally causes state to be changed and notified.
///
/// Inputs may be state mutations (which are simply closures that are executed immediately),
/// or tasks (more complex operations that may be async and cause multiple mutations over time).
///
/// The main objective of the controller is to implement default reusable behavior, but
/// this controller idea is extended with the concept of "plugins", which have various objetives
/// like getting notified of state changes to integrate with a view or to customize
/// the behavior of the controller itself.
pub struct ChatController {
    state: ChatState,
    /// A list of plugins defining custom behavior.
    plugins: Vec<(
        ChatControllerPluginRegistrationId,
        Box<dyn ChatControllerPlugin>,
    )>,
    /// Weak self-reference used by async tasks (or threads) spawned from
    /// inside the controller.
    accessor: ChatControllerAccessor,
    send_abort_on_drop: Option<AbortOnDropHandle>,
    load_bots_abort_on_drop: Option<AbortOnDropHandle>,
    execute_tools_abort_on_drop: Option<AbortOnDropHandle>,
    client: Option<Box<dyn BotClient>>,
    tool_manager: Option<McpManagerClient>,
}

impl ChatController {
    /// Creates a new reference-counted `ChatController`.
    ///
    /// This is the only public way to create a `ChatController`. A weak ref is
    /// internally used and passed to built-in async tasks (or threads) and you
    /// may find this useful when integrating the controller with your framework
    /// of choice.
    pub fn new_arc() -> Arc<Mutex<Self>> {
        Arc::new_cyclic(|weak| {
            Mutex::new(Self {
                state: ChatState::default(),
                plugins: Vec::new(),
                accessor: ChatControllerAccessor::new(weak.clone()),
                send_abort_on_drop: None,
                load_bots_abort_on_drop: None,
                execute_tools_abort_on_drop: None,
                client: None,
                tool_manager: None,
            })
        })
    }

    pub fn builder() -> ChatControllerBuilder {
        ChatControllerBuilder::new()
    }

    /// Registers a plugin to extend the controller behavior. Runs after all other plugins.
    pub fn append_plugin<P>(&mut self, plugin: P) -> ChatControllerPluginRegistrationId
    where
        P: ChatControllerPlugin + 'static,
    {
        let id = ChatControllerPluginRegistrationId::new();
        self.plugins.push((id, Box::new(plugin)));
        id
    }

    /// Registers a plugin to extend the controller behavior. Runs before all other plugins.
    pub fn prepend_plugin<P>(&mut self, plugin: P) -> ChatControllerPluginRegistrationId
    where
        P: ChatControllerPlugin + 'static,
    {
        let id = ChatControllerPluginRegistrationId::new();
        self.plugins.insert(0, (id, Box::new(plugin)));
        id
    }

    /// Unregisters a previously registered plugin.
    pub fn unregister_plugin(&mut self, id: ChatControllerPluginRegistrationId) {
        self.plugins.retain(|(plugin_id, _)| *plugin_id != id);
    }

    /// Read-only access to state.
    pub fn state(&self) -> &ChatState {
        &self.state
    }

    /// Dispatch mutations to state in a single transactional batch.
    ///
    /// `on_state_mutation` will be called between each mutation application. However,
    /// `on_state_ready` will only be called once, after all mutations grouped in
    /// this call have been applied.
    ///
    /// This means each reported mutation can be analyzed against the previous state
    /// snapshot, before the final state is reported.
    ///
    /// At [`on_state_mutation`], any [`ListMutation`] can use `log()` against
    /// the state snapshot without getting wrong indices.
    ///
    /// This also means that a "delete" may not have effect on the state reported to
    /// `on_state_ready` if a subsequent mutation in the same batch "re-inserts" the deleted
    /// item. This is important for plugin implementers to understand. In this example, if
    /// your plugin has side effects over "deleted" items, it's better to simply flag
    /// the delation on `on_state_mutation` and perform a full scan on the whole state
    /// that is commited to `on_state_ready`.
    ///
    /// As the "caller" of this method, you are responsible for grouping mutations
    /// that may alter the effect of each other.
    #[track_caller]
    pub fn dispatch_mutations(&mut self, mutations: Vec<ChatStateMutation>) {
        log::trace!("dispatch_mutation from {}", Location::caller());

        for mutation in mutations {
            for (_, plugin) in &mut self.plugins {
                plugin.on_state_mutation(&self.state, &mutation);
            }
            mutation.apply(&mut self.state);
        }

        for (_, plugin) in &mut self.plugins {
            plugin.on_state_ready(&self.state);
        }
    }

    /// Shorthand for dispatching a single mutation.
    ///
    /// Accepts any type that can be converted into a `ChatStateMutation`.
    ///
    /// See [`Self::dispatch_mutations`] for details about mutation grouping and plugin
    /// notifications.
    #[track_caller]
    pub fn dispatch_mutation(&mut self, mutation: impl Into<ChatStateMutation>) {
        self.dispatch_mutations(vec![mutation.into()]);
    }

    /// Get access to state to perform arbitrary unotified mutations.
    ///
    /// ## Danger
    ///
    /// Plugins will not get notified of this, and you may cause serious inconsistencies.
    ///
    /// This function only exists to perform quick modifications that are reverted
    /// almost immediately, leaving state as it was before.
    ///
    /// If you are using this, you should keep the lock to the controller until
    /// you undo what you did.
    #[track_caller]
    pub fn dangerous_state_mut(&mut self) -> &mut ChatState {
        log::trace!("dangerous_state_mut from {}", Location::caller());
        &mut self.state
    }

    pub fn dispatch_task(&mut self, task: ChatTask) {
        for (_, plugin) in &mut self.plugins {
            let control = plugin.on_task(&task);
            match control {
                ChatControl::Continue => continue,
                ChatControl::Stop => return,
            }
        }

        match task {
            ChatTask::Send(bot_id) => {
                self.handle_send(bot_id);
            }
            ChatTask::Stop => {
                self.clear_streaming_artifacts();
            }
            ChatTask::Load => {
                self.handle_load();
            }
            ChatTask::Execute(tool_calls, bot_id) => {
                self.handle_execute(tool_calls, bot_id);
            }
        }
    }

    fn handle_send(&mut self, bot_id: BotId) {
        // Clean previous streaming artifacts if any.
        self.clear_streaming_artifacts();

        let Some(mut client) = self.client.clone() else {
            self.dispatch_mutation(VecMutation::Push(Message::app_error(
                "No bot client configured",
            )));

            return;
        };

        // let Some(bot) = self.state.get_bot(&bot_id).cloned() else {
        //     self.dispatch_state_mutation(|state| {
        //         state.messages.push(Message::app_error("Bot not found"));
        //     });
        //     return;
        // };

        self.dispatch_mutation(VecMutation::Push(Message {
            from: EntityId::Bot(bot_id.clone()),
            content: MessageContent::default(),
            metadata: MessageMetadata {
                is_writing: true,
                ..Default::default()
            },
            ..Default::default()
        }));

        self.dispatch_mutation(ChatStateMutation::SetIsStreaming(true));

        let messages_context = self
            .state
            .messages
            .iter()
            .filter(|m| m.from != EntityId::App)
            .cloned()
            .collect::<Vec<_>>();

        let controller = self.accessor.clone();
        self.send_abort_on_drop = Some(spawn_abort_on_drop(async move {
            // The realtime check is hack to avoid showing a loading message for realtime assistants
            // TODO: we should base this on upgrade rather than capabilities
            // if !bot.capabilities.supports_realtime() {
            //     controller.lock_with(|c| {
            //         c.dispatch_state_mutation(|state| {
            //             state.messages.push(Message {
            //                 from: EntityId::Bot(bot_id.clone()),
            //                 metadata: MessageMetadata {
            //                     // TODO: Evaluate removing this from messages in favor of
            //                     // `is_streaming` in the controller.
            //                     is_writing: true,
            //                     ..Default::default()
            //                 },
            //                 ..Default::default()
            //             });
            //         })
            //     });
            // }

            let Some(tools) = controller.lock_with(|c| {
                c.tool_manager
                    .as_ref()
                    .map(|tm| tm.get_all_namespaced_tools())
                    .unwrap_or_default()
            }) else {
                return;
            };

            let message_stream = amortize(client.send(&bot_id, &messages_context, &tools));
            let mut message_stream = std::pin::pin!(message_stream);
            while let Some(result) = message_stream.next().await {
                let should_break = controller
                    .lock_with(|c| c.handle_message_content(result, &bot_id))
                    .unwrap_or(true);

                if should_break {
                    break;
                }
            }
            controller.lock_with(|c| c.clear_streaming_artifacts());
        }));
    }

    /// Aborts current streaming operation and cleans up artifacts.
    fn clear_streaming_artifacts(&mut self) {
        if self.send_abort_on_drop.is_none() {
            return;
        }

        self.send_abort_on_drop = None;

        self.dispatch_mutation(ChatStateMutation::SetIsStreaming(false));

        let mut updates_to_dispatch: Vec<ChatStateMutation> = Vec::new();
        // Indices to clean last, at the end of the other mutations.
        let mut indices_to_remove: Vec<usize> = Vec::new();

        // One pass to update dirty messages while generating a list of useless ones for later.
        for (index, message) in self.state.messages.iter().enumerate() {
            if message.metadata.is_writing {
                if message.content.is_empty() {
                    indices_to_remove.push(index);
                } else {
                    updates_to_dispatch.push(
                        VecMutation::update_with(&self.state.messages, index, |m| {
                            m.metadata.is_writing = false;
                        })
                        .into(),
                    );
                }
            }
        }

        self.dispatch_mutations(updates_to_dispatch);
        self.dispatch_mutation(VecMutation::RemoveMany::<Message>(indices_to_remove.into()));
    }

    /// Changes the client used by this controller when sending messages and laoding bots.
    ///
    /// NOTE: Calling this will reset the current bots and load status.
    pub fn set_client(&mut self, client: Option<Box<dyn BotClient>>) {
        self.client = client;
        self.dispatch_mutation(VecMutation::<Bot>::Clear);
        self.dispatch_mutation(ChatStateMutation::SetLoadStatus(Status::Idle));
    }

    fn handle_load(&mut self) {
        self.dispatch_mutation(ChatStateMutation::SetLoadStatus(Status::Working));

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.dispatch_mutation(ChatStateMutation::SetLoadStatus(Status::Error));
                self.dispatch_mutation(VecMutation::Push(Message::app_error(
                    "No bot client configured",
                )));
                return;
            }
        };

        let controller = self.accessor.clone();
        self.load_bots_abort_on_drop = Some(spawn_abort_on_drop(async move {
            let (bots, errors) = client.bots().await.into_value_and_errors();
            controller.lock_with(move |c| {
                if errors.is_empty() {
                    c.dispatch_mutation(ChatStateMutation::SetLoadStatus(Status::Success));
                } else {
                    c.dispatch_mutation(ChatStateMutation::SetLoadStatus(Status::Error));
                }

                c.dispatch_mutation(VecMutation::Set(bots.unwrap_or_default()));

                let messages = errors.into_iter().map(|e| Message::app_error(e)).collect();
                c.dispatch_mutation(VecMutation::Extend(messages));
            });
        }));
    }

    fn handle_message_content(
        &mut self,
        result: ClientResult<MessageContent>,
        bot_id: &BotId,
    ) -> bool {
        // For simplicity, lets handle this as an standard Result, ignoring content
        // if there are errors.
        match result.into_result() {
            Ok(mut content) => {
                // Take any pending upgrade from the client and abort if any.
                match content.upgrade.take() {
                    Some(upgrade) => {
                        let mut upgrade = Some(upgrade);
                        for (_, plugin) in &mut self.plugins {
                            upgrade = plugin.on_upgrade(upgrade.unwrap(), bot_id);
                            if upgrade.is_none() {
                                break;
                            }
                        }
                        return true;
                    }
                    None => {}
                }

                // TODO: Handle unexpected message.
                // TODO: Handle tools.

                self.dispatch_mutation(VecMutation::update_last_with(
                    &self.state.messages,
                    |message| {
                        message.update_content(|c| {
                            *c = content.clone();
                        });
                    },
                ));

                false
            }
            Err(errors) => {
                let messages = errors.into_iter().map(|e| Message::app_error(e)).collect();
                self.dispatch_mutation(VecMutation::Extend(messages));

                true
            }
        }
    }

    pub fn bot_client(&self) -> Option<&dyn BotClient> {
        self.client.as_deref()
    }

    pub fn bot_client_mut(&mut self) -> Option<&mut (dyn BotClient + 'static)> {
        self.client.as_deref_mut()
    }

    pub fn tool_manager(&self) -> Option<&McpManagerClient> {
        self.tool_manager.as_ref()
    }

    pub fn tool_manager_mut(&mut self) -> Option<&mut McpManagerClient> {
        self.tool_manager.as_mut()
    }

    pub fn set_tool_manager(&mut self, tool_manager: Option<McpManagerClient>) {
        self.tool_manager = tool_manager;
    }

    fn handle_execute(&mut self, tool_calls: Vec<ToolCall>, bot_id: Option<BotId>) {
        let Some(tool_manager) = self.tool_manager.clone() else {
            self.dispatch_mutation(VecMutation::Push(Message::app_error(
                "Tool execution failed: Tool manager not available",
            )));
            return;
        };

        let controller = self.accessor.clone();

        let loading_text = if tool_calls.len() == 1 {
            format!(
                "Executing tool '{}'...",
                display_name_from_namespaced(&tool_calls[0].name)
            )
        } else {
            format!("Executing {} tools...", tool_calls.len())
        };

        let loading_message = Message {
            from: EntityId::Tool,
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

        self.dispatch_mutation(VecMutation::Push(loading_message));
        self.dispatch_mutation(ChatStateMutation::SetIsStreaming(true));

        self.execute_tools_abort_on_drop = Some(spawn_abort_on_drop(async move {
            // Execute tool calls using MCP manager
            let tool_results = tool_manager.execute_tool_calls(tool_calls.clone()).await;

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
                        "🔧 Tool '{}' executed successfully:\n`{}`",
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
                        text.push_str(&format!("**{}** ❌: {}\n\n", display_name, result.content));
                    } else {
                        let summary = crate::utils::tool_execution::create_tool_output_summary(
                            tool_name,
                            &result.content,
                        );
                        text.push_str(&format!("**{}** ✅: `{}`\n\n", display_name, summary));
                    }
                }
                text
            };

            controller.lock_with(|c| {
                c.dispatch_mutation(ChatStateMutation::SetIsStreaming(false));
                c.dispatch_mutation(VecMutation::remove_many_from_filter(
                    &c.state.messages,
                    |_, m| !(m.metadata.is_writing && m.from == EntityId::Tool),
                ));
                c.dispatch_mutation(VecMutation::Push(Message {
                    from: EntityId::Tool, // Tool results use the tool role
                    content: MessageContent {
                        text: results_text,
                        tool_results: tool_results,
                        ..Default::default()
                    },
                    ..Default::default()
                }));

                if let Some(bot_id) = bot_id {
                    c.dispatch_task(ChatTask::Send(bot_id));
                }
            });
        }));
    }

    /// Shorthand for removing the client, tool manager, bots and resetting load status.
    // NOTE: This has been added to simplify the migration for Moly, replacing all
    // `store.bot_context = None` that were before, but is an obscure functionality.
    pub fn reset_connections(&mut self) {
        self.client = None;
        self.tool_manager = None;
        self.dispatch_mutation(VecMutation::<Bot>::Clear);
        self.dispatch_mutation(ChatStateMutation::SetLoadStatus(Status::Idle));
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

pub struct ChatControllerBuilder(Arc<Mutex<ChatController>>);

impl ChatControllerBuilder {
    pub fn new() -> Self {
        Self(ChatController::new_arc())
    }

    pub fn with_client<C>(self, client: C) -> Self
    where
        C: BotClient + 'static,
    {
        self.0.lock().unwrap().set_client(Some(Box::new(client)));
        self
    }

    pub fn with_plugin_append<P>(self, plugin: P) -> Self
    where
        P: ChatControllerPlugin + 'static,
    {
        self.0.lock().unwrap().append_plugin(plugin);
        self
    }

    pub fn with_plugin_prepend<P>(self, plugin: P) -> Self
    where
        P: ChatControllerPlugin + 'static,
    {
        self.0.lock().unwrap().prepend_plugin(plugin);
        self
    }

    pub fn with_tool_manager(self, tool_manager: McpManagerClient) -> Self {
        self.0.lock().unwrap().set_tool_manager(Some(tool_manager));
        self
    }

    pub fn build_arc(self) -> Arc<Mutex<ChatController>> {
        self.0
    }
}
