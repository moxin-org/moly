pub mod add_provider_modal;
pub mod delete_server_modal;
pub mod moly_server_screen;
pub mod provider_view;
pub mod providers;
pub mod providers_screen;
use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    providers_screen::live_design(cx);
    moly_server_screen::live_design(cx);
    delete_server_modal::live_design(cx);
    provider_view::live_design(cx);
    providers::live_design(cx);
    add_provider_modal::live_design(cx);
}
