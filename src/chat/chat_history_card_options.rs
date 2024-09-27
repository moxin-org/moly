use crate::data::chats::chat::ChatID;
use makepad_widgets::*;
use super::chat_history_card::ChatHistoryCardAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import makepad_draw::shader::draw_color::DrawColor;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;

    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")
    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")

    ChatHistoryCardOptions = {{ChatHistoryCardOptions}} {
        width: Fit
        height: Fit
        flow: Overlay

        options_content = <RoundedView> {
            width: Fit,
            height: Fit,
            flow: Down,

            draw_bg: {
                color: #fff,
                border_width: 1.0,
                border_color: #D0D5DD,
                radius: 2.
            }

            edit_chat_name = <MolyButton> {
                width: Fit
                height: Fit
                padding: { top: 12, right: 12, bottom: 12, left: 12}

                draw_bg: {
                    border_width: 0,
                    radius: 0
                }

                icon_walk: {width: 12, height: 12}
                draw_icon: {
                    svg_file: (ICON_EDIT),
                    fn get_color(self) -> vec4 {
                        return #000;
                    }
                }

                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 9},
                    fn get_color(self) -> vec4 {
                        return #000;
                    }
                }

                text: "Edit Chat Name"
            }


            delete_chat = <MolyButton> {
                width: Fill
                height: Fit
                padding: { top: 12, right: 12, bottom: 12, left: 12}
                align: {x: 0.0, y: 0.5}

                draw_bg: {
                    border_width: 0,
                    radius: 0
                }

                icon_walk: {width: 12, height: 12}
                draw_icon: {
                    svg_file: (ICON_DELETE),
                    fn get_color(self) -> vec4 {
                        return #B42318;
                    }
                }

                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 9},
                    fn get_color(self) -> vec4 {
                        return #B42318;
                    }
                }

                text: "Delete Chat"
            }
        }
    }
}
#[derive(Live, LiveHook, Widget)]
pub struct ChatHistoryCardOptions {
    #[deref]
    view: View,
    #[rust]
    chat_id: ChatID,
}

impl Widget for ChatHistoryCardOptions {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatHistoryCardOptions {
    pub fn selected(&mut self, cx: &mut Cx, chat_id: ChatID) {
        self.chat_id = chat_id;
        self.redraw(cx);
    }
}

impl ChatHistoryCardOptionsRef {
    pub fn selected(&mut self, cx: &mut Cx, chat_id: ChatID) {
        let Some(mut inner) = self.borrow_mut() else { return };
        inner.selected(cx, chat_id);
    }
}

impl WidgetMatchEvent for ChatHistoryCardOptions {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        if self.button(id!(delete_chat)).clicked(actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatHistoryCardAction::MenuClosed(self.chat_id),
            );
            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatHistoryCardAction::DeleteChatOptionSelected(self.chat_id),
            );
        }

        if self.button(id!(edit_chat_name)).clicked(actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatHistoryCardAction::MenuClosed(self.chat_id),
            );
            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatHistoryCardAction::ActivateTitleEdition(self.chat_id),
            );
        }
    }
}
