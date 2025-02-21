//! Widgets provided by this crate. You can import this in your DSL.
//!
//! Note: Some widgets may depend on certain feature flags.

mod avatar;
mod message_loading;
mod message_markdown;

#[cfg(any(feature = "async-rt", feature = "async-web"))]
pub mod chat;
pub mod messages;
pub mod prompt_input;

#[cfg(any(feature = "async-rt", feature = "async-web"))]
pub use chat::*;
pub use messages::*;
pub use prompt_input::*;

pub fn live_design(cx: &mut makepad_widgets::Cx) {
    makepad_code_editor::live_design(cx);
    message_markdown::live_design(cx);
    message_loading::live_design(cx);
    avatar::live_design(cx);
    messages::live_design(cx);
    prompt_input::live_design(cx);
    chat::live_design(cx);
}
