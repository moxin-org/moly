//! Framework-agnostic state management to implement a `Chat` component/widget/element.

use crate::protocol::*;
use std::sync::atomic::{AtomicU64, Ordering};

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
pub struct ChatState {
    pub messages: Vec<Message>,
    pub prompt_input_content: MessageContent,
    pub message_editor: Option<(usize, MessageContent)>,
    pub is_streaming: bool,
}

/// UI events that your framework of choice should feed into the [`ChatController`].
pub enum ChatUiEvent {
    PromptInputContentChange(MessageContent),
    MessageEditorContentChange(MessageContent),
    MessageEditorVisibilityChange(usize, bool),
    Send,
}

/// An update to the chat state to be applied.
pub type ChatStateMutation = Box<dyn FnMut(&mut ChatState) + Send>;

/// Unique identifier for a registered callback. Can be used to unregister it later.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CallbackId(u64);

impl CallbackId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        CallbackId(id)
    }
}

pub struct ChatController {
    state: ChatState,
    on_state_change_callbacks: Vec<(CallbackId, Box<dyn Fn(&ChatState) + Send>)>,
    on_state_mutation_callbacks: Vec<(
        CallbackId,
        Box<dyn Fn(&mut ChatStateMutation) -> ChatControl + Send>,
    )>,
    on_ui_event_callbacks: Vec<(CallbackId, Box<dyn Fn(&ChatUiEvent) -> ChatControl + Send>)>,
}

impl ChatController {
    pub fn on_state_change<F>(&mut self, callback: F) -> CallbackId
    where
        F: Fn(&ChatState) + Send + 'static,
    {
        let id = CallbackId::new();
        self.on_state_change_callbacks
            .push((id, Box::new(callback)));
        id
    }

    pub fn on_ui_event<F>(&mut self, callback: F) -> CallbackId
    where
        F: Fn(&ChatUiEvent) -> ChatControl + Send + 'static,
    {
        let id = CallbackId::new();
        self.on_ui_event_callbacks.push((id, Box::new(callback)));
        id
    }

    pub fn on_state_mutation<F>(&mut self, callback: F) -> CallbackId
    where
        F: Fn(&mut ChatStateMutation) -> ChatControl + Send + 'static,
    {
        let id = CallbackId::new();
        self.on_state_mutation_callbacks
            .push((id, Box::new(callback)));
        id
    }

    pub fn unregister_callback(&mut self, id: CallbackId) {
        self.on_state_change_callbacks
            .retain(|(callback_id, _)| *callback_id != id);
        self.on_ui_event_callbacks
            .retain(|(callback_id, _)| *callback_id != id);
        self.on_state_mutation_callbacks
            .retain(|(callback_id, _)| *callback_id != id);
    }

    pub fn state(&self) -> &ChatState {
        &self.state
    }

    pub fn dispatch_state_mutation<F>(&mut self, mutation: F)
    where
        F: FnMut(&mut ChatState) + Send + 'static,
    {
        let mut boxed_mutation: ChatStateMutation = Box::new(mutation);
        for (_, callback) in &self.on_state_mutation_callbacks {
            let control = callback(&mut boxed_mutation);
            match control {
                ChatControl::Continue => continue,
                ChatControl::Stop => return,
            }
        }

        self.perform_state_mutation(boxed_mutation);
    }

    pub fn perform_state_mutation<F>(&mut self, mut mutation: F)
    where
        F: FnMut(&mut ChatState) + Send + 'static,
    {
        mutation(&mut self.state);
        for (_, callback) in &self.on_state_change_callbacks {
            callback(&self.state);
        }
    }

    pub fn dispatch_ui_event(&mut self, event: ChatUiEvent) {
        for (_, callback) in &self.on_ui_event_callbacks {
            let control = callback(&event);
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
}

// dispatch_ui_event, perform_ui_event, dispatch_state_mutation, perform_state_mutation
// clipboard and fs interfaces?
