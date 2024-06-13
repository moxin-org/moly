use makepad_widgets::*;
use crate::data::store::Store;
use super::chat_history_card::ChatHistoryCardWidgetRefExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::MoxinButton;
    import makepad_draw::shader::std::*;

    import crate::chat::chat_panel::ChatAgentAvatar;
    import crate::chat::chat_history_card::ChatHistoryCard;

    ICON_NEW_CHAT = dep("crate://self/resources/icons/new_chat.svg")
    ICON_CLOSE_PANEL = dep("crate://self/resources/icons/close_left_panel.svg")
    ICON_OPEN_PANEL = dep("crate://self/resources/icons/open_left_panel.svg")

    ChatHistoryActions = <View> {
        spacing: 10,
        height: Fit

        close_panel_button = <MoxinButton> {
            width: Fit,
            height: Fit,
            icon_walk: {margin: { left: -3, top: -1 }, width: 22, height: 22},
            draw_icon: {
                svg_file: (ICON_CLOSE_PANEL),
                fn get_color(self) -> vec4 {
                    return #475467;
                }
            }
        }

        new_chat_button = <MoxinButton> {
            width: Fit,
            height: Fit,
            icon_walk: {width: 20, height: 20},
            draw_icon: {
                svg_file: (ICON_NEW_CHAT),
                fn get_color(self) -> vec4 {
                    return #475467;
                }
            }
        }
    }

    ChatHistory = {{ChatHistory}} {
        flow: Down
        width: Fill
        height: Fill
        padding: 10

        <ChatHistoryActions> {}

        list = <PortalList> {
            ChatHistoryCard = <ChatHistoryCard> {margin: {top: 20}}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistory {
    #[deref]
    view: View,
}

impl Widget for ChatHistory {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let chats = &scope.data.get::<Store>().unwrap().chats;

        let mut saved_chat_ids = chats
            .saved_chats
            .iter()
            .map(|c| c.borrow().id)
            .collect::<Vec<_>>();

        // Reverse sort chat ids.
        saved_chat_ids.sort_by(|a, b| b.cmp(a));

        let chats_count = chats.saved_chats.len();

        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, chats_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < chats_count {
                        let mut item = list
                            .item(cx, item_id, live_id!(ChatHistoryCard))
                            .unwrap()
                            .as_chat_history_card();
                        let _ = item.set_chat_id(saved_chat_ids[item_id]);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatHistory {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if self.button(id!(new_chat_button)).clicked(&actions) {
            store.chats.create_empty_chat();
            self.redraw(cx);
        }
    }
}
