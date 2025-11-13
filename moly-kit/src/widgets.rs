//! Widgets provided by this crate. You can import this in your DSL.
//!
//! Note: Some widgets may depend on certain feature flags.

use makepad_widgets::*;

mod attachment_list;
mod attachment_view;
mod attachment_viewer_modal;
mod avatar;
mod chat_lines;
mod citation;
pub(crate) mod citation_list;
mod hook_view;
mod image_view;
mod message_loading;
mod message_markdown;
mod message_thinking_block;
mod slot;
mod standard_message_content;
mod theme_moly_kit_light;

pub mod messages;
pub use messages::*;

pub mod prompt_input;
pub use prompt_input::*;

pub mod moly_modal;
pub use moly_modal::*;

pub mod realtime;
pub use realtime::*;

cfg_if::cfg_if! {
    if #[cfg(any(feature = "async-rt", feature = "async-web"))] {
        pub mod chat;
        pub use chat::*;
    }
}

pub fn live_design(cx: &mut makepad_widgets::Cx) {
    theme_moly_kit_light::live_design(cx);
    // Link the MolyKit theme to the MolyKit-specific theme.
    // Currently we only have a light theme which we use as default.
    cx.link(live_id!(moly_kit_theme), live_id!(theme_moly_kit_light));

    hook_view::live_design(cx);
    image_view::live_design(cx);
    attachment_view::live_design(cx);
    moly_modal::live_design(cx);
    attachment_viewer_modal::live_design(cx);
    attachment_list::live_design(cx);
    citation::live_design(cx);
    citation_list::live_design(cx);
    makepad_code_editor::live_design(cx);
    message_markdown::live_design(cx);
    message_loading::live_design(cx);
    avatar::live_design(cx);
    slot::live_design(cx);
    standard_message_content::live_design(cx);
    chat_lines::live_design(cx);
    crate::deep_inquire::widgets::live_design(cx);
    messages::live_design(cx);
    prompt_input::live_design(cx);
    chat::live_design(cx);
    realtime::live_design(cx);
    message_thinking_block::live_design(cx);
}
