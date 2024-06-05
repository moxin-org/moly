pub mod delete_model_modal;
pub mod downloaded_files_table;
pub mod downloaded_files_row;
pub mod model_info_modal;
pub mod my_models_screen;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    my_models_screen::live_design(cx);
    downloaded_files_table::live_design(cx);
    downloaded_files_row::live_design(cx);
    delete_model_modal::live_design(cx);
    model_info_modal::live_design(cx);
}
