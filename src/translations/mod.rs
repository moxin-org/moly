use makepad_widgets::Cx;

mod en;
mod zh;
mod es;

pub fn live_design(cx: &mut Cx) {
    en::live_design(cx);
    zh::live_design(cx);
    es::live_design(cx);
}

