pub mod mcp_screen;
pub mod mcp_servers;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    mcp_screen::live_design(cx);
    mcp_servers::live_design(cx);
}
