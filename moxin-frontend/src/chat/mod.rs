pub mod chat_history;
pub mod chat_history;
pub mod chat_line_loading;
pub mod chat_line;
pub mod chat_panel;
pub mod chat_screen;
pub mod model_info;
pub mod model_selector_list;
pub mod model_selector;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    chat_history::live_design(cx);
    chat_line_loading::live_design(cx);
    chat_line::live_design(cx);
    chat_panel::live_design(cx);
    chat_screen::live_design(cx);
    model_info::live_design(cx);
    model_selector_list::live_design(cx);
    model_selector::live_design(cx);
}
