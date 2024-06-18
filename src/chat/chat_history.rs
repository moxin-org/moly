use crate::data::{chats::chat::ChatID, store::Store};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use makepad_widgets::*;
use moxin_protocol::data::File;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    import crate::chat::chat_panel::ChatAgentAvatar;

    ChatCard = {{ChatCard}} {
        content = <RoundedView> {
            flow: Down
            width: Fill
            height: Fit
            padding: 20
            spacing: 12

            cursor: Hand

            draw_bg: {
                color: #fff
                border_width: 1
            }

            title = <Label> {
                width: Fit,
                height: Fit,
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000,
                }
                text: ""
            }

            <View> {
                width: Fill
                height: Fit
                align: {y: 1}

                avatar = <ChatAgentAvatar> {
                    width: 30
                    height: 30

                    draw_bg: {
                        radius: 8
                    }
                    avatar_label = {
                        text: ""
                    }
                }

                filler = <View> {width: Fill}

                date = <Label> {
                    width: Fit,
                    height: Fit,
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 10},
                        color: #667085,
                    }
                    text: "5:29 PM, 5/12/24"
                }
            }


        }
    }

    ChatHistory = {{ChatHistory}} {
        flow: Down
        width: Fill
        height: Fill
        padding: 10

        list = <PortalList> {
            ChatCard = <ChatCard> {margin: {top: 20}}
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
                            .item(cx, item_id, live_id!(ChatCard))
                            .unwrap()
                            .as_chat_card();
                        let _ = item.set_chat_id(saved_chat_ids[item_id]);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }

        DrawStep::done()
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatCard {
    #[deref]
    view: View,
    #[rust]
    chat_id: ChatID,
}

impl Widget for ChatCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let chat = store
            .chats
            .saved_chats
            .iter()
            .find(|c| c.borrow().id == self.chat_id)
            .unwrap();

        if let Some(current_chat_id) = store.chats.get_current_chat_id() {
            let content_view = self.view(id!(content));

            if current_chat_id == self.chat_id {
                let active_border_color = vec3(0.082, 0.522, 0.604);
                content_view.apply_over(
                    cx,
                    live! {
                        draw_bg: {border_color: (active_border_color)}
                    },
                );
            } else {
                let border_color = vec3(0.918, 0.925, 0.941);
                content_view.apply_over(
                    cx,
                    live! {
                        draw_bg: {border_color: (border_color)}
                    },
                );
            }
        }

        let title_label = self.view.label(id!(title));
        title_label.set_text(chat.borrow_mut().get_title());

        let initial_letter = chat
            .borrow()
            .model_filename
            .chars()
            .next()
            .unwrap_or_default()
            .to_uppercase()
            .to_string();

        let avatar_label = self.view.label(id!(avatar.avatar_label));
        avatar_label.set_text(&initial_letter);

        let date_label = self.view.label(id!(date));

        // Format date.
        // TODO: Feels wrong to asume the id will always be the date, do smth about this.
        let naive_datetime = NaiveDateTime::from_timestamp_millis(chat.borrow().id as i64)
            .expect("Invalid timestamp");
        let datetime: DateTime<Local> = Local.from_utc_datetime(&naive_datetime);
        let formatted_date = datetime.format("%-I:%M %p, %-d/%m/%y").to_string();

        date_label.set_text(&formatted_date);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let widget_uid = self.widget_uid();

        if let Some(fe) = self.view(id!(content)).finger_down(actions) {
            if fe.tap_count == 1 {
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    ChatHistoryAction::ChatSelected(self.chat_id),
                );

                store.select_chat(self.chat_id);

                self.redraw(cx);
            }
        }
    }
}

impl ChatCard {
    pub fn set_chat_id(&mut self, id: ChatID) {
        self.chat_id = id;
    }
}

impl ChatCardRef {
    pub fn set_chat_id(&mut self, id: ChatID) -> Result<(), &'static str> {
        let Some(mut inner) = self.borrow_mut() else {
            return Err("Widget not found in the document");
        };

        inner.set_chat_id(id);
        Ok(())
    }
}

#[derive(Clone, DefaultNone, Eq, Hash, PartialEq, Debug)]
pub enum ChatHistoryAction {
    None,
    ChatSelected(ChatID),
}
