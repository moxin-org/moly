pub mod settings_screen;
pub mod mofa_settings;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    mofa_settings::live_design(cx);
    settings_screen::live_design(cx);
}
