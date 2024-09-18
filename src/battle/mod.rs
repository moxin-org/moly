pub mod agent_selector;
pub mod battle_screen;
pub mod mae;
pub mod messages;
pub mod prompt;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    messages::live_design(cx);
    agent_selector::live_design(cx);
    prompt::live_design(cx);
    battle_screen::live_design(cx);
}
