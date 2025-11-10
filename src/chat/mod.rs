pub mod chat_history;
pub mod chat_history_card;
pub mod chat_history_card_options;
pub mod chat_history_panel;
pub mod chat_params;
pub mod chat_screen;
pub mod chat_screen_mobile;
pub mod chat_view;
pub mod chats_deck;
pub mod delete_chat_modal;
pub mod entity_button;
pub mod model_info;
pub mod model_selector;
pub mod model_selector_item;
pub mod model_selector_list;
pub mod model_selector_loading;
pub mod moly_bot_grouping;
pub mod moly_bot_filter;
pub mod prompt_input;
pub mod shared;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    entity_button::live_design(cx);
    prompt_input::live_design(cx);
    chat_history_card::live_design(cx);
    chat_history::live_design(cx);
    chat_history_panel::live_design(cx);
    chat_params::live_design(cx);
    chat_view::live_design(cx);
    chats_deck::live_design(cx);
    chat_screen::live_design(cx);
    chat_screen_mobile::live_design(cx);
    model_info::live_design(cx);
    model_selector_list::live_design(cx);
    model_selector_item::live_design(cx);
    model_selector::live_design(cx);
    model_selector_loading::live_design(cx);
    shared::live_design(cx);
    delete_chat_modal::live_design(cx);
    chat_history_card_options::live_design(cx);
}
