pub mod settings_screen;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    settings_screen::live_design(cx);
}
