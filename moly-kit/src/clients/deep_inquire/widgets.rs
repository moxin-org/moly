pub(super) mod deep_inquire_content;
pub(super) mod stages;

pub(crate) fn live_design(cx: &mut makepad_widgets::Cx) {
    stages::live_design(cx);
    deep_inquire_content::live_design(cx);
}
