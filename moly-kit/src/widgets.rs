//! Widgets provided by this crate. You can import this in your DSL.
//!
//! Note: Some widgets may depend on certain feature flags.

mod avatar;
mod chat_lines;
mod citation;
pub(crate) mod citation_list;
mod message_loading;
mod message_markdown;
mod message_thinking_block;
mod slot;
mod standard_message_content;

pub mod messages;
pub use messages::*;

pub mod prompt_input;
pub use prompt_input::*;

cfg_if::cfg_if! {
    if #[cfg(any(feature = "async-rt", feature = "async-web"))] {
        pub mod chat;
        pub use chat::*;
    }
}

pub fn live_design(cx: &mut makepad_widgets::Cx) {
    citation::live_design(cx);
    citation_list::live_design(cx);
    makepad_code_editor::live_design(cx);
    message_markdown::live_design(cx);
    message_loading::live_design(cx);
    avatar::live_design(cx);
    slot::live_design(cx);
    standard_message_content::live_design(cx);
    chat_lines::live_design(cx);
    messages::live_design(cx);
    prompt_input::live_design(cx);
    chat::live_design(cx);
    message_thinking_block::live_design(cx);
}
