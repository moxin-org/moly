pub mod settings_screen;
pub mod connection_settings;
pub mod delete_server_modal;
pub mod configure_connection_modal;
use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    connection_settings::live_design(cx);
    settings_screen::live_design(cx);
    delete_server_modal::live_design(cx);
    configure_connection_modal::live_design(cx);
}
