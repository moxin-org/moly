//! Framework-agnostic state management to implement a `Chat` component/widget/element.

use crate::{protocol::*, utils::asynchronous::AbortOnDropHandle};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicU64, Ordering},
};

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
        F: (FnMut(&mut ChatState) -> R) + Send + 'static,
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
        F: (FnMut(&mut ChatState) -> R) + Send + 'static,
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
        self.abort();

        let Some(client) = self.client.as_mut() else {
            self.dispatch_state_mutation(|state| {
                state
                    .messages
                    .push(Message::app_error("No bot client configured"));
            });
            return;
        };

        let Some(bot_id) = self.state.bot_id.as_ref() else {
            self.dispatch_state_mutation(|state| {
                state.messages.push(Message::app_error("No bot selected"));
            });
            return;
        };

        // Do not proceed if None because it means it was cancelled by a plugin.
        let Some(()) = self.dispatch_state_mutation(|state| {
            state.is_streaming = true;
            state.messages.push(Message {
                from: EntityId::User,
                content: std::mem::take(&mut state.prompt_input_content),
                ..Default::default()
            });
        }) else {
            return;
        };

        let controller = self.accessor.clone();
    }

    /// Aborts any ongoing send operation.
    fn abort(&mut self) {
        if let Some(mut handle) = self.send_abort_on_drop.take() {
            handle.abort();
        }
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
    async fn bots(&mut self) -> ClientResult<&[Bot]> {
        if !self
            .cached_bots_result
            .as_ref()
            .map(|r| r.has_value())
            .unwrap_or(false)
        {
            let Some(client) = self.client.as_mut() else {
                return ClientError::new(
                    ClientErrorKind::Unknown,
                    "No bot client configured".to_string(),
                )
                .into();
            };

            self.cached_bots_result = Some(client.bots().await);
        }

        self.cached_bots_result
            .as_ref()
            .unwrap()
            .map_value(|bots| bots.as_slice())
    }
}

// dispatch_ui_event, perform_ui_event, dispatch_state_mutation, perform_state_mutation
// clipboard and fs interfaces?
