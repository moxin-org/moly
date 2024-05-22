use makepad_widgets::Cx;

pub mod external_link;
pub mod icon;
pub mod modal;
pub mod popup;
pub mod resource_imports;
pub mod styles;
pub mod utils;
pub mod widgets;

pub fn live_design(cx: &mut Cx) {
    styles::live_design(cx);
    resource_imports::live_design(cx);
    widgets::live_design(cx);
    icon::live_design(cx);
    modal::live_design(cx);
    external_link::live_design(cx);
    popup::live_design(cx);
}
