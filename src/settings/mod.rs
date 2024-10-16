mod battle_section;
pub mod settings_screen;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    battle_section::live_design(cx);
    settings_screen::live_design(cx);
}
