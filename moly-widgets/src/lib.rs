use makepad_widgets::Cx;

mod avatar;
mod message_markdown;
pub mod messages;
pub mod protocol;

pub use messages::*;
pub use protocol::*;

pub fn live_design(cx: &mut Cx) {
    makepad_code_editor::live_design(cx);
    message_markdown::live_design(cx);
    avatar::live_design(cx);

    messages::live_design(cx);
}
