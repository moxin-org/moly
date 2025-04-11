use makepad_widgets::Cx;

pub mod actions;
pub mod desktop_buttons;
pub mod download_notification_popup;
pub mod external_link;
pub mod list;
pub mod meta;
pub mod modal;
pub mod moly_server_popup;
pub mod popup_notification;
pub mod resource_imports;
pub mod styles;
pub mod tooltip;
pub mod utils;
pub mod widgets;

pub fn live_design(cx: &mut Cx) {
    meta::live_design(cx);
    list::live_design(cx);
    styles::live_design(cx);
    resource_imports::live_design(cx);
    widgets::live_design(cx);
    modal::live_design(cx);
    popup_notification::live_design(cx);
    external_link::live_design(cx);
    download_notification_popup::live_design(cx);
    tooltip::live_design(cx);
    desktop_buttons::live_design(cx);
    moly_server_popup::live_design(cx);
}
