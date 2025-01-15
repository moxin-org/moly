use makepad_widgets::Cx;

pub mod messages;
pub mod protocol;

pub use messages::*;
pub use protocol::*;

pub fn live_design(cx: &mut Cx) {
    messages::live_design(cx);
}
