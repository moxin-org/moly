use makepad_widgets::*;

use crate::data::chat::ChatMessage;
use crate::data::store::Store;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import crate::shared::styles::*;

    ICON_COPY = dep("crate://self/resources/icons/copy.svg")
    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")

    ChatLineActionButton = <Button> {
        draw_icon: {
            fn get_color(self) -> vec4 {
                return #BDBDBD;
            }
        }
        padding: 0,
        icon_walk: {width: 14, height: 14}
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                return sdf.result
            }
        }
        text: ""
    }

    ChatLine = {{ChatLine}} {
        margin: {top: 10, bottom: 3},
        width: Fill,
        height: Fit,

        cursor: Default,

        avatar_section = <View> {
            width: Fit,
            height: Fit,
            margin: {left: 20, right: 20},
        }

        main_section = <View> {
            width: Fill,
            height: Fit,

            flow: Down,
            spacing: 8,

            body_section = <View> {}

            actions_section = <View> {
                width: Fill,
                height: 16,
                actions = <View> {
                    width: Fill,
                    height: Fit,
                    visible: false,
                    spacing: 6,

                    <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_EDIT) }
                    }
                    <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_COPY) }
                    }
                    delete_button = <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_DELETE) }
                    }
                }
            }
        }

    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatLineAction {
    Delete(usize),
    Edit(usize),
    Copy(usize),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatLine {
    #[deref]
    view: View,

    #[rust]
    message_id: usize,
}

impl Widget for ChatLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        if let Some(action) = actions.find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerHoverIn(_) = action.cast() {
                self.view(id!(actions_section.actions)).set_visible(true);
                self.redraw(cx);
            }
            if let ViewAction::FingerHoverOut(_) = action.cast() {
                self.view(id!(actions_section.actions)).set_visible(false);
                self.redraw(cx);
            }
        }

        if self.button(id!(delete_button)).clicked(&actions) {
            let widget_id = self.view.widget_uid();
            cx.widget_action(
                widget_id,
                &scope.path,
                ChatLineAction::Delete(self.message_id),
            );
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatLineRef {
    pub fn set_message_id(&mut self, message_id: usize) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.message_id = message_id;
    }
}
