use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import crate::shared::styles::*;

    ICON_COPY = dep("crate://self/resources/icons/copy.svg")
    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")

    ChatLineEditButton = <RoundedView> {
        width: 56,
        height: 31,
        align: {x: 0.5, y: 0.5}
        spacing: 6,

        cursor: Hand,

        draw_bg: { color: #099250 }

        button_label = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
        }
    }

    SaveButton = <ChatLineEditButton> {
        button_label = {
            text: "Save"
        }
    }

    CancelButton = <ChatLineEditButton> {
        draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }

        button_label = {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return #000;
                }
            }
            text: "Cancel"
        }
    }

    EditTextInput = <TextInput> {
        width: Fill,
        height: Fit,
        padding: 0,

        draw_bg: {
            color: #fff
        }
        draw_text: {
            text_style:<REGULAR_FONT>{font_size: 10},
            word: Wrap,

            instance prompt_enabled: 0.0
            fn get_color(self) -> vec4 {
                return #000;
            }
        }

        // TODO find a way to override colors
        draw_cursor: {
            instance focus: 0.0
            uniform border_radius: 0.5
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    0.,
                    0.,
                    self.rect_size.x,
                    self.rect_size.y,
                    self.border_radius
                )
                sdf.fill(mix(#fff, #000, self.focus));
                return sdf.result
            }
        }

        // TODO find a way to override colors
        draw_select: {
            instance hover: 0.0
            instance focus: 0.0
            uniform border_radius: 2.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    0.,
                    0.,
                    self.rect_size.x,
                    self.rect_size.y,
                    self.border_radius
                )
                sdf.fill(mix(#eee, #ddd, self.focus)); // Pad color
                return sdf.result
            }
        }
    }

    ChatLineBody = <View> {
        width: Fill,
        height: Fit,
        spacing: 5,
        flow: Down,

        <View> {
            height: 20,
            align: {x: 0.0, y: 0.5},

            role = <Label> {
                width: Fit,
                height: Fit,
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000
                }
            }
        }

        chat_message = <View> {
            width: Fill,
            height: Fit,
            chat_label = <Label> {
                width: Fill,
                height: Fit,
                padding: {top: 12, bottom: 12},

                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 10},
                    color: #000,
                    word: Wrap,
                }
            }
        }

        chat_edit = <View> {
            visible: false,
            width: Fill,
            height: Fit,
            flow: Down,
            padding: {top: 12, bottom: 12},
            align: {x: 0.5, y: 0.0},

            input = <EditTextInput> {
            }

            <View> {
                width: Fit,
                height: Fit,
                margin: {top: 10},
                spacing: 6,
                save = <SaveButton> {}
                cancel = <CancelButton> {}
            }
        }
    }

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

            body_section =  <ChatLineBody> {}

            actions_section = <View> {
                width: Fill,
                height: 16,
                actions = <View> {
                    width: Fill,
                    height: Fit,
                    visible: false,
                    spacing: 6,

                    copy_button = <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_COPY) }
                    }
                    edit_button = <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_EDIT) }
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
    Edit(usize, String),
    Copy(usize),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatLine {
    #[deref]
    view: View,

    #[rust]
    message_id: usize,

    #[rust]
    actions_enabled: bool,
}

impl Widget for ChatLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        if let Some(action) = actions.find_widget_action(self.view.widget_uid()) {
            if self.actions_enabled {
                if let ViewAction::FingerHoverIn(_) = action.cast() {
                    self.view(id!(actions_section.actions)).set_visible(true);
                    self.redraw(cx);
                }
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

        if self.button(id!(edit_button)).clicked(&actions) {
            self.set_edit_mode(cx, true);
        }

        if let Some(fe) = self.view(id!(save)).finger_up(&actions) {
            if fe.was_tap() {
                let updated_message = self.text_input(id!(chat_edit.input)).text();

                let widget_id = self.view.widget_uid();
                cx.widget_action(
                    widget_id,
                    &scope.path,
                    ChatLineAction::Edit(self.message_id, updated_message),
                );

                self.set_edit_mode(cx, false);
            }
        }

        if let Some(fe) = self.view(id!(cancel)).finger_up(&actions) {
            if fe.was_tap() {
                self.set_edit_mode(cx, false);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatLine {
    pub fn set_edit_mode(&mut self, cx: &mut Cx, enabled: bool) {
        self.view(id!(chat_message)).set_visible(!enabled);
        self.view(id!(chat_edit)).set_visible(enabled);

        if enabled {
            let message = self.label(id!(chat_message.chat_label)).text();
            self.text_input(id!(chat_edit.input))
                .set_text(message.as_str());
        }

        self.redraw(cx);
    }
}

impl ChatLineRef {
    pub fn set_message_id(&mut self, message_id: usize) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.message_id = message_id;
    }

    pub fn set_actions_enabled(&mut self, enabled: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.actions_enabled = enabled;
    }
}
