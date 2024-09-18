pub mod battle_screen;
pub mod half;
pub mod prompt;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    half::live_design(cx);
    prompt::live_design(cx);
    battle_screen::live_design(cx);
}
