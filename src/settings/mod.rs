pub mod moly_server_settings;
pub mod providers_screen;
pub mod delete_server_modal;
pub mod add_provider_modal;
pub mod provider_view;
pub mod providers;
use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    moly_server_settings::live_design(cx);
    providers_screen::live_design(cx);
    delete_server_modal::live_design(cx);
    provider_view::live_design(cx);
    providers::live_design(cx);
    add_provider_modal::live_design(cx);
}
