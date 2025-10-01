//! Framework-agnostic state management to implement a `Chat` component/widget/element.

use std::panic::Location;

use crate::{
    McpManagerClient,
    protocol::*,
    utils::asynchronous::{AbortOnDropHandle, PlatformSendStream, spawn_abort_on_drop},
};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicU64, Ordering},
};

use futures::StreamExt;

/// Controls if remaining callbacks and default behavior should be executed.
pub enum ChatControl {
    Continue,
    Stop,
}

/// Represents a generic status in which an operation can be.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Status {
    #[default]
    Idle,
    Working,
    Error,
    Success,
}

impl Status {
    pub fn is_idle(&self) -> bool {
        matches!(self, Status::Idle)
    }

    pub fn is_working(&self) -> bool {
        matches!(self, Status::Working)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Status::Error)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Status::Success)
    }
}

/// State of the chat that you should reflect in your view component/widget/element.
// TODO: Makes sense? #[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChatState {
    /// The chat history sent as context to LLMs.
    pub messages: Vec<Message>,
    /// Indicates that the LLM is still streaming the response ("writing").
    pub is_streaming: bool,
    /// The bots that were loaded from the configured client.
    pub bots: Vec<Bot>,
    pub load_status: Status,
}

impl ChatState {
    pub fn get_bot(&self, bot_id: &BotId) -> Option<&Bot> {
        self.bots.iter().find(|b| &b.id == bot_id)
    }
}

/// Represents complex (mostly async) operations that may cause multiple mutations
/// over time.
#[derive(Clone, Debug, PartialEq)]
pub enum ChatTask {
    /// Causes the whole list of messages to be sent to the specified bot and starts
    /// the streaming response work in the background.
    Send(BotId),
    /// Interrupts the streaming started by `Send`.
    Stop,
    /// Should be triggered to start fetching async data (e.g. bots).
    ///
    /// Eventually, the state will contain the list of bots or errors as messages.
    Load,
}

/// Allows to hook between dispatched events of any kind.
///
/// It's the fundamental building block for extending [`ChatController`] beyond
/// its default behavior and integrating it with other technologies.
pub trait ChatControllerPlugin: Send {
    /// Called when new state is available.
    ///
    /// Usually used to bind the controller to some view component/widget/element
    /// in your framework of choice.
    fn on_state_change(&mut self, _state: &ChatState) {}

    fn on_task(&mut self, _event: &ChatTask) -> ChatControl {
        ChatControl::Continue
    }

    /// Called with a state mutator to be applied over the current state.
    ///
    /// Useful for replicating state outside of the controller.
    fn on_state_mutation(&mut self, _mutation: &mut (dyn FnMut(&mut ChatState) + Send)) {}

    // attachment handling?
}

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

/// Unique identifier for a registered plugin. Can be used to unregister it later.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChatControllerPluginRegistrationId(u64);

impl ChatControllerPluginRegistrationId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
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
                client: None,
                tool_manager: None,
            })
        })
    }

    pub fn builder() -> ChatControllerBuilder {
        ChatControllerBuilder::new()
    }

    /// Registers a plugin to extend the controller behavior.
    pub fn register_plugin<P>(&mut self, plugin: P) -> ChatControllerPluginRegistrationId
    where
        P: ChatControllerPlugin + 'static,
    {
        let id = ChatControllerPluginRegistrationId::new();
        self.plugins.push((id, Box::new(plugin)));
        id
    }

    /// Unregisters a previously registered plugin.
    pub fn unregister_plugin(&mut self, id: ChatControllerPluginRegistrationId) {
        self.plugins.retain(|(plugin_id, _)| *plugin_id != id);
    }

    /// Read-only access to state. Use `dispatch_state_mutation` to change it.
    ///
    /// If you need to bypass plugins, you can use `perform_state_mutation` instead.
    pub fn state(&self) -> &ChatState {
        &self.state
    }

    /// Dispatches a state mutation to be applied.
    ///
    /// Plugins will be called before the mutation is applied.
    ///
    /// This function can return a value from the mutation closure.
    ///
    /// The controller will only run this mutation once over the state. Plugins
    /// get access to this closure (without the return value) so they may do
    /// additional stuff with it (e.g. replicate the mutation outside of the controller).
    #[track_caller]
    pub fn dispatch_state_mutation<F, R>(&mut self, mut mutation: F) -> R
    where
        F: (FnMut(&mut ChatState) -> R) + Send,
    {
        log::trace!("dispatch_state_mutation from {}", Location::caller());

        for (_, plugin) in &mut self.plugins {
            plugin.on_state_mutation(&mut |state| {
                mutation(state);
            });
        }

        self.perform_state_mutation(mutation)
    }

    /// Applies a state mutation directly, bypassing plugins.
    ///
    /// This function can return a value from the mutation closure.
    pub fn perform_state_mutation<F, R>(&mut self, mut mutation: F) -> R
    where
        F: (FnMut(&mut ChatState) -> R) + Send,
    {
        let out = mutation(&mut self.state);

        for (_, plugin) in &mut self.plugins {
            plugin.on_state_change(&self.state);
        }

        out
    }

    pub fn dispatch_task(&mut self, event: ChatTask) {
        for (_, plugin) in &mut self.plugins {
            let control = plugin.on_task(&event);
            match control {
                ChatControl::Continue => continue,
                ChatControl::Stop => return,
            }
        }
        self.perform_task(event);
    }

    pub fn perform_task(&mut self, event: ChatTask) {
        match event {
            ChatTask::Send(bot_id) => {
                self.handle_send(bot_id);
            }
            ChatTask::Stop => {
                self.clear_streaming_artifacts();
            }
            ChatTask::Load => {
                self.handle_load();
            }
        }
    }

    fn handle_send(&mut self, bot_id: BotId) {
        // Clean previous streaming artifacts if any.
        self.clear_streaming_artifacts();

        let Some(mut client) = self.client.clone() else {
            self.dispatch_state_mutation(|state| {
                state
                    .messages
                    .push(Message::app_error("No bot client configured"));
            });
            return;
        };

        let Some(bot) = self.state.get_bot(&bot_id).cloned() else {
            self.dispatch_state_mutation(|state| {
                state.messages.push(Message::app_error("Bot not found"));
            });
            return;
        };

        self.dispatch_state_mutation(|state| {
            state.is_streaming = true;
        });

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
            if !bot.capabilities.supports_realtime() {
                controller.lock_with(|c| {
                    c.dispatch_state_mutation(|state| {
                        state.messages.push(Message {
                            from: EntityId::Bot(bot_id.clone()),
                            metadata: MessageMetadata {
                                // TODO: Evaluate removing this from messages in favor of
                                // `is_streaming` in the controller.
                                is_writing: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    })
                });
            }

            let Some(tools) = controller.lock_with(|c| {
                c.tool_manager
                    .as_ref()
                    .map(|tm| tm.get_all_namespaced_tools())
                    .unwrap_or_default()
            }) else {
                return;
            };

            let message_stream = amortize(client.send(&bot.id, &messages_context, &tools));
            let mut message_stream = std::pin::pin!(message_stream);
            while let Some(result) = message_stream.next().await {
                let should_break = controller
                    .lock_with(|c| c.handle_message_content(result))
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
        self.dispatch_state_mutation(|state| {
            state.messages.retain_mut(|m| {
                m.metadata.is_writing = false;
                !m.content.is_empty()
            });
        });
    }

    /// Changes the client used by this controller when sending messages.
    pub fn set_client<C>(&mut self, client: C)
    where
        C: BotClient + 'static,
    {
        self.client = Some(Box::new(client));
        self.state.bots.clear();
    }

    fn handle_load(&mut self) {
        self.dispatch_state_mutation(|state| {
            state.load_status = Status::Working;
        });

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.dispatch_state_mutation(|state| {
                    state.load_status = Status::Error;
                    state
                        .messages
                        .push(Message::app_error("No bot client configured"));
                });
                return;
            }
        };

        let controller = self.accessor.clone();
        self.send_abort_on_drop = Some(spawn_abort_on_drop(async move {
            let (bots, errors) = client.bots().await.into_value_and_errors();
            controller.lock_with(move |c| {
                c.dispatch_state_mutation(move |state| {
                    if errors.is_empty() {
                        state.load_status = Status::Success;
                    } else {
                        state.load_status = Status::Error;
                    }

                    state.bots = bots.clone().unwrap_or_default();
                    for error in &errors {
                        state.messages.push(Message::app_error(error));
                    }
                });
            });
        }));
    }

    fn handle_message_content(&mut self, result: ClientResult<MessageContent>) -> bool {
        // For simplicity, lets handle this as an standard Result, ignoring content
        // if there are errors.
        match result.into_result() {
            Ok(content) => {
                // Check if this is a realtime upgrade message
                if let Some(Upgrade::Realtime(_channel)) = &content.upgrade {
                    todo!();
                }

                // TODO: Handle unexpected message.
                // TODO: Handle tools.

                self.dispatch_state_mutation(|state| {
                    state.messages.last_mut().unwrap().content = content.clone();
                });

                false
            }
            Err(errors) => {
                self.dispatch_state_mutation(|state| {
                    let messages_append = errors.iter().map(|e| Message::app_error(e));
                    state.messages.extend(messages_append);
                });

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
        self.0.lock().unwrap().set_client(client);
        self
    }

    pub fn with_plugin<P>(self, plugin: P) -> Self
    where
        P: ChatControllerPlugin + 'static,
    {
        self.0.lock().unwrap().register_plugin(plugin);
        self
    }

    pub fn build_arc(self) -> Arc<Mutex<ChatController>> {
        self.0
    }
}

// dispatch_ui_event, perform_ui_event, dispatch_state_mutation, perform_state_mutation
// clipboard and fs interfaces?
