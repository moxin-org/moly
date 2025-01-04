pub mod settings_screen;
pub mod mofa_settings;
pub mod delete_server_modal;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    mofa_settings::live_design(cx);
    settings_screen::live_design(cx);
    delete_server_modal::live_design(cx);
}
