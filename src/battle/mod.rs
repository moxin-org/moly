pub mod agent_markdown;
pub mod battle_screen;
pub mod battle_service;
pub mod battle_sheet;
pub mod ending;
pub mod failure;
pub mod messages;
pub mod opening;
pub mod spinner;
pub mod styles;
pub mod vote;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    styles::live_design(cx);
    opening::live_design(cx);
    spinner::live_design(cx);
    vote::live_design(cx);
    agent_markdown::live_design(cx);
    messages::live_design(cx);
    battle_screen::live_design(cx);
    failure::live_design(cx);
    ending::live_design(cx);
}
