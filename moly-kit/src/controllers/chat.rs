use std::sync::Arc;

use crate::{
    protocol::*,
    utils::asynchronous::{AbortOnDropHandle, abort_on_drop, spawn},
};

struct Editor;

trait ChatMutation {}

/// When doing custom event handling, allows you to control if default behavior
/// should still be performed or not.
pub enum ChatEventControl {
    /// Default behavior should be performed.
    Continue,
    /// Default behavior should not be performed.
    Stop,
}

/// Direct UI events fed into this controller.
pub(crate) enum ChatEvent {
    /// Prompt input send button clicked.
    PromptInputSend,
    /// Text in the prompt input changed.
    PromptInputTextChange(String),
}

pub struct ChatState {
    pub messages: Vec<Message>,
    // is_streaming: bool,
    // prompt_input_text: String,
    current_editor: Option<Editor>,
    streaming_abort_on_drop: Option<AbortOnDropHandle>,
}

impl ChatState {
    /// Check if the chat is currently streaming/writing/loading a message.
    pub fn is_streaming(&self) -> bool {
        self.streaming_abort_on_drop
            .as_ref()
            .map(|handle| !handle.was_manually_aborted())
            .unwrap_or(false)
    }
}

pub struct ChatController {
    state: ChatState,
}

impl ChatController {
    /// Executed when an UI event is fed into this controller.
    fn on_event() {}
    /// Executed when a mutation will be applied to state.
    ///
    /// Useful for triggering redraw in UI libraries and for state replication.
    fn on_mutation() {}
    /// Called after a mutation removed something related to a external blob.
    fn on_blob_leak() {}
    /// Feeds an UI event into this controller.
    fn event(e: ChatEvent) {}
    /// Applies a mutation to the state and causes state to be emitted.
    fn mutation(m: impl ChatMutation) {}
}

/*
ahhhhhh this approach will take a lot, i need something faster....



*/
