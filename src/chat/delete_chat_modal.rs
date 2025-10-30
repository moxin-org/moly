use makepad_widgets::*;

use crate::data::{chats::chat::ChatID, store::Store};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::MolyButton;
    use crate::shared::resource_imports::*;

    pub DeleteChatModal = {{DeleteChatModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 600
            height: Fit
            padding: {top: 44, right: 30 bottom: 30 left: 50}
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                border_radius: 3
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Right

                padding: {top: 8, bottom: 20}

                title = <View> {
                    width: Fit,
                    height: Fit,

                    <Label> {
                        text: "Delete Chat"
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 13},
                            color: #000
                        }
                    }
                }

                filler_x = <View> {width: Fill, height: Fit}

                close_button = <MolyButton> {
                    width: Fit,
                    height: Fit,

                    margin: {top: -8}

                    draw_icon: {
                        svg_file: (ICON_CLOSE),
                        fn get_color(self) -> vec4 {
                            return #000;
                        }
                    }
                    icon_walk: {width: 12, height: 12}
                }
            }

            body = <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 40,

                delete_prompt = <Label> {
                    width: Fill
                    draw_text: {
                        text_style: <REGULAR_FONT>{
                            font_size: 10,
                        },
                        color: #000
                        wrap: Word
                    }
                }

                actions = <View> {
                    width: Fill, height: Fit
                    flow: Right,
                    align: {x: 1.0, y: 0.5}
                    spacing: 20

                    cancel_button = <MolyButton> {
                        width: Fit,
                        height: Fit,
                        padding: {top: 10, bottom: 10, left: 14, right: 14}

                        draw_bg: {
                            instance border_radius: 2.0,
                            border_color_1: #D0D5DD,
                            border_size: 1.2,
                            color: #fff,
                        }

                        text: "Cancel"
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 10},
                            color: #x0
                        }
                    }

                    delete_button = <MolyButton> {
                        width: Fit,
                        height: Fit,
                        padding: {top: 10, bottom: 10, left: 14, right: 14}

                        draw_bg: {
                            instance border_radius: 2.0,
                            color: #D92D20,
                        }

                        text: "Delete"
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 10},
                            color: #fff
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DeleteChatModal {
    #[deref]
    view: View,
    #[rust]
    chat_id: ChatID,
}

#[derive(Clone, Debug, DefaultNone)]
pub enum DeleteChatModalAction {
    None,
    CloseButtonClicked,
    ChatDeleted,
    Cancelled,
}

impl Widget for DeleteChatModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let chat_title = scope
            .data
            .get::<Store>()
            .unwrap()
            .chats
            .saved_chats
            .iter()
            .map(|chat| chat.borrow())
            .find(|chat| chat.id == self.chat_id)
            .unwrap()
            .get_title()
            .to_string();

        let prompt_text = format!(
            "Are you sure you want to delete {}?\nThis action cannot be undone.",
            chat_title
        );
        self.label(ids!(wrapper.body.delete_prompt))
            .set_text(cx, &prompt_text);

        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetMatchEvent for DeleteChatModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.button(ids!(close_button)).clicked(actions) {
            cx.action(DeleteChatModalAction::CloseButtonClicked);
        }

        if self.button(ids!(delete_button)).clicked(actions) {
            let store = scope.data.get_mut::<Store>().unwrap();
            store.delete_chat(self.chat_id);
            cx.action(DeleteChatModalAction::ChatDeleted);
            cx.redraw_all();
        }

        if self.button(ids!(cancel_button)).clicked(actions) {
            cx.action(DeleteChatModalAction::Cancelled);
        }
    }
}

impl DeleteChatModalRef {
    pub fn set_chat_id(&mut self, chat_id: ChatID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.chat_id = chat_id;
        }
    }
}
