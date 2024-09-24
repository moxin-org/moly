pub mod agent_markdown;
pub mod agent_selector;
pub mod battle_screen;
pub mod battle_service;
pub mod battle_sheet;
pub mod mae;
pub mod messages;
pub mod no_messages;
pub mod prompt;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    agent_markdown::live_design(cx);
    no_messages::live_design(cx);
    messages::live_design(cx);
    agent_selector::live_design(cx);
    prompt::live_design(cx);
    battle_screen::live_design(cx);
}
