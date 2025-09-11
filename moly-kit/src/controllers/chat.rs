//! Framework-agnostic state management to implement a `Chat` component/widget/element.

use crate::protocol::*;
use std::sync::{
    Arc, Mutex,
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
#[derive(Clone, Debug)]
pub struct ChatState {
    pub messages: Vec<Message>,
    pub prompt_input_content: MessageContent,
    pub message_editor: Option<(usize, MessageContent)>,
    pub is_streaming: bool,
}

/// UI events that your framework of choice should feed into the [`ChatController`].
#[derive(Clone, Debug, PartialEq)]
pub enum ChatUiEvent {
    PromptInputContentChange(MessageContent),
    MessageEditorContentChange(MessageContent),
    MessageEditorVisibilityChange(usize, bool),
    Send,
}

/// An update to the chat state to be applied.
pub type ChatStateMutation = Box<dyn FnMut(&mut ChatState) + Send>;

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
    fn on_state_mutation(&mut self, _mutation: &mut ChatStateMutation) -> ChatControl {
        ChatControl::Continue
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

struct ChatControllerInner {
    state: ChatState,
    plugins: Vec<(
        ChatControllerPluginRegistrationId,
        Arc<Mutex<Box<dyn ChatControllerPlugin>>>,
    )>,
}

#[derive(Clone)]
pub struct ChatController {
    inner: Arc<Mutex<ChatControllerInner>>,
}

impl ChatController {
    pub fn register_plugin<P>(&mut self, plugin: P) -> ChatControllerPluginRegistrationId
    where
        P: ChatControllerPlugin + 'static,
    {
        let id = ChatControllerPluginRegistrationId::new();
        self.lock()
            .plugins
            .push((id, Arc::new(Mutex::new(Box::new(plugin)))));
        id
    }

    pub fn unregister_plugin(&mut self, id: ChatControllerPluginRegistrationId) {
        self.lock()
            .plugins
            .retain(|(plugin_id, _)| *plugin_id != id);
    }

    // pub fn state(&self) -> ChatState {
    //     // TODO: Expensive.
    //     self.inner().state.clone()
    // }

    pub fn dispatch_state_mutation<F>(&mut self, mutation: F)
    where
        F: FnMut(&mut ChatState) + Send + 'static,
    {
        let mut boxed_mutation: ChatStateMutation = Box::new(mutation);

        {
            let mut inner = self.lock();
            for (_, plugin) in &mut inner.plugins {
                let control = plugin
                    .lock()
                    .unwrap()
                    .on_state_mutation(&mut boxed_mutation);

                match control {
                    ChatControl::Continue => continue,
                    ChatControl::Stop => return,
                }
            }
        }

        self.perform_state_mutation(boxed_mutation);
    }

    pub fn perform_state_mutation<F>(&mut self, mut mutation: F)
    where
        F: FnMut(&mut ChatState) + Send + 'static,
    {
        let mut inner = self.lock();
        mutation(&mut inner.state);
        for (_, plugin) in inner.plugins.iter().cloned() {
            plugin.lock().unwrap().on_state_change(&inner.state);
        }
    }

    pub fn dispatch_ui_event(&mut self, event: ChatUiEvent) {
        for (_, plugin) in &mut self.lock().plugins {
            let control = plugin.lock().unwrap().on_ui_event(&event);
            match control {
                ChatControl::Continue => continue,
                ChatControl::Stop => return,
            }
        }
        self.perform_ui_event(event);
    }

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
            ChatUiEvent::Send => {}
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ChatControllerInner> {
        self.inner.lock().unwrap()
    }
}

// dispatch_ui_event, perform_ui_event, dispatch_state_mutation, perform_state_mutation
// clipboard and fs interfaces?
