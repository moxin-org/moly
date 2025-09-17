//! Framework-agnostic state management to implement a `Chat` component/widget/element.

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
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Controls if remaining callbacks and default behavior should be executed.
///
/// Used when hooking into UI events and state mutations.
pub enum ChatControl {
    Continue,
    Stop,
}

/// State of the chat that you should reflect in your view component/widget/element.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChatState {
    /// The chat history sent as context to LLMs.
    pub messages: Vec<Message>,
    /// The content to send as a new user message.
    pub prompt_input_content: MessageContent,
    /// Indicates a message being edited.
    pub message_editor: Option<(usize, MessageContent)>,
    /// Indicates that the LLM is still streaming the response ("writing").
    pub is_streaming: bool,
    /// The bot to interact with when sending messages.
    pub bot_id: Option<BotId>,
}

impl ChatState {
    /// Show or hide the editor for a message.
    ///
    /// Limitation: Only one editor can be shown at a time. If you try to show another editor,
    /// the previous one will be hidden. If you try to hide an editor different from the one
    /// currently shown, nothing will happen.
    pub fn set_message_editor_visibility(&mut self, index: usize, visible: bool) {
        if index >= self.messages.len() {
            return;
        }

        if visible {
            let buffer = self.messages[index].content.clone();
            self.message_editor = Some((index, buffer));
        } else if self.message_editor.as_ref().map(|(index, _)| *index) == Some(index) {
            self.message_editor = None;
        }
    }
}

/// UI events that your framework of choice should feed into the [`ChatController`].
#[derive(Clone, Debug, PartialEq)]
pub enum ChatUiEvent {
    /// Triggered when the prompt input content changes (e.g. user typing,
    /// attaching files, etc).
    PromptInputContentChange(MessageContent),
    /// Triggered when a message editor content changes.
    MessageEditorContentChange(MessageContent),
    /// Triggered when a message editor is shown or hidden.
    MessageEditorVisibilityChange(usize, bool),
    /// Triggered when the user triggers sending the current chat history + prompt
    /// if any.
    Send,
}

/// Allows to hook between dispatched events and state mutations.
///
/// It's the basic building block for extending [`ChatController`] beyond its
/// default behavior.
pub trait ChatControllerPlugin: Send {
    /// Called when new state is available.
    ///
    /// Usually used to bind the controller to some view component/widget/element
    /// in your framework of choice.
    fn on_state_change(&mut self, _state: &ChatState) {}

    /// Called when a UI interaction occurs.
    ///
    /// Note: When a UI interaction is reported depends on how the UI components/widgets/elements
    /// that are used as your "view" are implemented.
    fn on_ui_event(&mut self, _event: &ChatUiEvent) -> ChatControl {
        ChatControl::Continue
    }

    /// Called with a state mutator to be applied over the current state.
    ///
    /// Useful for replicating state outside of the controller.
    fn on_state_mutation(
        &mut self,
        _mutation: &mut (dyn FnMut(&mut ChatState) + Send),
    ) -> ChatControl {
        ChatControl::Continue
    }

    // attachment handling?
}

/// Utility wrapper around a weak ref to a controller.
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
    client: Option<Box<dyn BotClient>>,
    cached_bots_result: Option<ClientResult<Vec<Bot>>>,
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
        let controller = Arc::new(Mutex::new(Self {
            state: ChatState::default(),
            plugins: Vec::new(),
            accessor: ChatControllerAccessor::default(),
            send_abort_on_drop: None,
            client: None,
            cached_bots_result: None,
            tool_manager: None,
        }));

        controller.lock().unwrap().accessor.handle = Arc::downgrade(&controller);
        controller
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
    /// Plugins will be called before the mutation is applied and can stop it.
    ///
    /// This function can return a value from the mutation closure. If a plugin
    /// interrupts the mutation, `None` is returned.
    ///
    /// The controller will only run this mutation once over the state. Plugins
    /// get access to this closure (without the return value) so they may do
    /// additional stuff with it (e.g. replicate the mutation outside of the controller).
    pub fn dispatch_state_mutation<F, R>(&mut self, mut mutation: F) -> Option<R>
    where
        F: (FnMut(&mut ChatState) -> R) + Send,
    {
        {
            for (_, plugin) in &mut self.plugins {
                let control = plugin.on_state_mutation(&mut |state| {
                    mutation(state);
                });

                match control {
                    ChatControl::Continue => continue,
                    ChatControl::Stop => return None,
                }
            }
        }

        Some(self.perform_state_mutation(mutation))
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

    /// Dispatches a UI event.
    ///
    /// Plugins will be called before the default behavior and can stop it.
    pub fn dispatch_ui_event(&mut self, event: ChatUiEvent) {
        for (_, plugin) in &mut self.plugins {
            let control = plugin.on_ui_event(&event);
            match control {
                ChatControl::Continue => continue,
                ChatControl::Stop => return,
            }
        }
        self.perform_ui_event(event);
    }

    /// Performs a UI event directly, bypassing plugins.
    pub fn perform_ui_event(&mut self, event: ChatUiEvent) {
        match event {
            ChatUiEvent::PromptInputContentChange(content) => {
                self.dispatch_state_mutation(move |state| {
                    state.prompt_input_content = content.clone();
                });
            }
            ChatUiEvent::MessageEditorContentChange(content) => {
                self.dispatch_state_mutation(move |state| {
                    if let Some((_, editor_content)) = &mut state.message_editor {
                        *editor_content = content.clone();
                    }
                });
            }
            ChatUiEvent::MessageEditorVisibilityChange(message_index, visible) => {
                self.dispatch_state_mutation(move |state| {
                    if visible {
                        if message_index < state.messages.len() {
                            let message = &state.messages[message_index];
                            state.message_editor = Some((message_index, message.content.clone()));
                        }
                    } else {
                        state.message_editor = None;
                    }
                });
            }
            ChatUiEvent::Send => {
                self.handle_send();
            }
        }
    }

    fn handle_send(&mut self) {
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

        let Some(bot_id) = self.state.bot_id.clone() else {
            self.dispatch_state_mutation(|state| {
                state.messages.push(Message::app_error("No bot selected"));
            });
            return;
        };

        // Do not proceed if None because it means it was cancelled by a plugin.
        let Some(()) = self.dispatch_state_mutation(|state| {
            let prompt_input_content = std::mem::take(&mut state.prompt_input_content);
            if !prompt_input_content.is_empty() {
                state.messages.push(Message {
                    from: EntityId::User,
                    content: prompt_input_content,
                    ..Default::default()
                });
            }
            state.is_streaming = true;
        }) else {
            return;
        };

        let messages_context = self
            .state
            .messages
            .iter()
            .filter(|m| m.from != EntityId::App)
            .cloned()
            .collect::<Vec<_>>();

        let controller = self.accessor.clone();
        self.send_abort_on_drop = Some(spawn_abort_on_drop(async move {
            let Some(bots) = controller.lock_with(|c| c.bots()) else {
                return;
            };

            let bots = bots.await;
            let Some(bots) = bots.into_value() else {
                return;
            };

            let bot = bots.into_iter().find(|b| b.id == bot_id);
            let Some(bot) = bot else {
                return;
            };

            // The realtime check is hack to avoid showing a loading message for realtime assistants
            // TODO: we should base this on upgrade rather than capabilities
            if !bot.capabilities.supports_realtime() {
                controller.lock_with(|c| {
                    c.dispatch_state_mutation(|state| {
                        state.messages.push(Message {
                            from: EntityId::Bot(state.bot_id.clone().unwrap()),
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

            let message_stream = amortize(client.send(&bot_id, &messages_context, &tools));
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

    /// Aborts the current send operation if any.
    pub fn abort_send(&mut self) {
        self.clear_streaming_artifacts();
    }

    /// Changes the client used by this controller when sending messages.
    pub fn set_client<C>(&mut self, client: C)
    where
        C: BotClient + 'static,
    {
        self.client = Some(Box::new(client));
        self.cached_bots_result = None;
    }

    /// Fetch the available bots from the underlying configured client.
    ///
    /// The response is cached the first time this resolves. If it was successful
    /// (at least partially), subsequent calls will return the cached value.
    // TODO: Syncronize to avoid race conditions on concurrent calls.
    fn bots(&mut self) -> BoxPlatformSendFuture<'static, ClientResult<Vec<Bot>>> {
        if let Some(cached) = self.cached_bots_result.as_ref() {
            if cached.has_value() {
                let cached = cached.clone();
                return Box::pin(async move { cached });
            }
        }

        let Some(client) = self.client.clone() else {
            return Box::pin(async {
                ClientError::new(
                    ClientErrorKind::Unknown,
                    "No bot client configured".to_string(),
                )
                .into()
            });
        };

        let controller = self.accessor.clone();
        Box::pin(async move {
            let result = client.bots().await;
            controller.lock_with({
                let result = result.clone();
                move |c| {
                    c.cached_bots_result = Some(result);
                }
            });
            result
        })
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

// dispatch_ui_event, perform_ui_event, dispatch_state_mutation, perform_state_mutation
// clipboard and fs interfaces?
