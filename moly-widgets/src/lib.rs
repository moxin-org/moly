use makepad_widgets::Cx;

mod avatar;

mod message_loading;
mod message_markdown;
pub mod messages;
pub mod prompt_input;
pub mod protocol;
pub mod utils;

pub use messages::*;
pub use prompt_input::*;
pub use protocol::*;

#[cfg(any(feature = "async-rt", feature = "async-web"))]
pub mod chat;

#[cfg(any(feature = "async-rt", feature = "async-web"))]
pub use chat::*;

pub fn live_design(cx: &mut Cx) {
    makepad_code_editor::live_design(cx);
    message_markdown::live_design(cx);
    message_loading::live_design(cx);
    avatar::live_design(cx);
    messages::live_design(cx);
    prompt_input::live_design(cx);
    chat::live_design(cx);
}
