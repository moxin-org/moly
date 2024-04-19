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
        empty_message: ""

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

        <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            padding: {top: 12, bottom: 12},
            align: {x: 0.5, y: 0.0},

            input = <EditTextInput> {
                read_only: true,
            }

            edit_buttons = <View> {
                visible: false,
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

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ChatLineState {
    #[default]
    Editable,
    NotEditable,
    OnEdit,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatLine {
    #[deref]
    view: View,

    #[rust]
    message_id: usize,

    #[rust]
    edition_state: ChatLineState,

    #[rust]
    actions_enabled: bool,
}

impl Widget for ChatLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatLine {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        match self.edition_state {
            ChatLineState::Editable => self.handle_editable_actions(cx, actions, scope),
            ChatLineState::OnEdit => self.handle_on_edit_actions(cx, actions, scope),
            ChatLineState::NotEditable => {}
        }
    }
}

impl ChatLine {
    pub fn set_edit_mode(&mut self, cx: &mut Cx, enabled: bool) {
        //self.edit_mode = enabled;
        self.edition_state = if enabled {
            ChatLineState::OnEdit
        } else {
            ChatLineState::Editable
        };

        self.view(id!(actions_section.actions)).set_visible(false);
        self.view(id!(edit_buttons)).set_visible(enabled);
        self.text_input(id!(input))
            .apply_over(cx, live! {read_only: (!enabled)});

        self.redraw(cx);
    }

    pub fn handle_editable_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if let Some(action) = actions.find_widget_action(self.view.widget_uid()) {
            match action.cast() {
                ViewAction::FingerHoverIn(_) => {
                    self.view(id!(actions_section.actions)).set_visible(true);
                    self.redraw(cx);
                }
                ViewAction::FingerHoverOut(_) => {
                    self.view(id!(actions_section.actions)).set_visible(false);
                    self.redraw(cx);
                }
                _ => {}
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
    }

    pub fn handle_on_edit_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if let Some(fe) = self.view(id!(save)).finger_up(&actions) {
            if fe.was_tap() {
                let updated_message = self.text_input(id!(input)).text();

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
}

impl ChatLineRef {
    pub fn set_role(&mut self, text: &str) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.label(id!(role)).set_text(text);
    }

    pub fn set_avatar_text(&mut self, text: &str) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.label(id!(avatar_label)).set_text(text);
    }

    pub fn set_message_text(&mut self, text: &str) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        match inner.edition_state {
            ChatLineState::Editable | ChatLineState::NotEditable => {
                inner.text_input(id!(input)).set_text(text.trim());
            }
            ChatLineState::OnEdit => {}
        }
    }

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

        if enabled {
            if inner.edition_state == ChatLineState::NotEditable {
                inner.edition_state = ChatLineState::Editable;
            }
        } else {
            inner.edition_state = ChatLineState::NotEditable;
            inner.view(id!(actions_section.actions)).set_visible(false);
        }
    }
}
