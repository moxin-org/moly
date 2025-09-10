//! Framework-agnostic state management to implement a `Chat` component/widget/element.

pub enum ChatControl {
    Continue,
    Stop,
}

pub struct ChatState;

pub enum ChatUiEvent {
    PromptInputTextChange(String),
}

pub struct ChatStateMutation;

pub struct ChatController;

// dispatch_ui_event, perform_ui_event, dispatch_state_mutation, perform_state_mutation
// clipboard and fs interfaces?
