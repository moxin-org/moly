use makepad_widgets::*;
use std::cell::{Ref, RefMut};

#[allow(unused)]
use crate::{
    Attachment,
    protocol::{BotCapabilities, BotCapability},
    utils::makepad::EventExt,
    widgets::attachment_list::{AttachmentListRef, AttachmentListWidgetExt},
};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;
    use link::shaders::*;

    use crate::widgets::attachment_list::*;

    pub PromptInput = {{PromptInput}} <CommandTextInput> {
        send_icon: dep("crate://self/resources/send.svg"),
        stop_icon: dep("crate://self/resources/stop.svg"),

        persistent = {
            padding: {top: 8, bottom: 6, left: 4, right: 10}
            draw_bg: {
                color: #fff,
                border_radius: 10.0,
                border_color: #D0D5DD,
                border_size: 1.0,
            }
            center = {
                left = {
                    attach = <Button> {
                        visible: false
                        text: "",
                        draw_text: {
                            text_style: <THEME_FONT_ICONS> {
                                font_size: 16.
                            }
                            color: #000,
                            color_hover: #000,
                            color_focus: #000
                            color_down: #000
                        }
                        draw_bg: {
                            color_down: #0000
                            border_radius: 7.
                            border_size: 0.
                        }
                    }
                }
                text_input = {
                    empty_text: "Start typing...",
                    draw_bg: {
                        fn pixel(self) -> vec4 {
                            return vec4(0.);
                        }
                    }
                    draw_text: {
                        color: #000
                        color_hover: #000
                        color_focus: #000
                        color_empty: #98A2B3
                        color_empty_focus: #98A2B3
                        text_style: {font_size: 11}
                    }
                    draw_selection: {
                        color: #d9e7e9
                        color_hover: #d9e7e9
                        color_focus: #d9e7e9
                    }
                    draw_cursor: {
                        fn pixel(self) -> vec4 {
                            return #bbb;
                        }
                    }
                }
                right = {
                    align: {x: 0.5, y: 0.5}
                    spacing: 5
                    audio = <Button> {
                        visible: false
                        text: ""
                        draw_text: {
                            text_style: <THEME_FONT_ICONS> {
                                font_size: 16.
                            }
                            color: #000,
                            color_hover: #000,
                            color_focus: #000
                            color_down: #000
                        }
                        draw_bg: {
                            color_down: #0000
                            border_radius: 7.
                            border_size: 0.
                        }
                    }
                    submit = <Button> {
                        width: 28,
                        height: 28,
                        padding: {right: 2},
                        margin: {bottom: 2},

                        draw_icon: {
                            color: #fff
                        }

                        draw_bg: {
                            fn get_color(self) -> vec4 {
                                if self.enabled == 0.0 {
                                    return #D0D5DD;
                                }

                                return #000;
                            }

                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                let center = self.rect_size * 0.5;
                                let radius = min(self.rect_size.x, self.rect_size.y) * 0.5;

                                sdf.circle(center.x, center.y, radius);
                                sdf.fill_keep(self.get_color());

                                return sdf.result
                            }
                        }
                        icon_walk: {
                            width: 12,
                            height: 12
                            margin: {top: 0, left: 2},
                        }
                    }
                }
            }
            bottom = {
                attachments = <DenseAttachmentList> {
                    wrapper = {
                        margin: { top: 6 }
                    }
                }
            }
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
pub enum Task {
    #[default]
    Send,
    Stop,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub enum Interactivity {
    #[default]
    Enabled,
    Disabled,
}

/// A prepared text input for conversation with bots.
///
/// This is mostly a dummy widget. Prefer using and adapting [crate::widgets::chat::Chat] instead.
#[derive(Live, Widget)]
pub struct PromptInput {
    #[deref]
    deref: CommandTextInput,

    /// Icon used by this widget when the task is set to [Task::Send].
    #[live]
    pub send_icon: LiveValue,

    /// Icon used by this widget when the task is set to [Task::Stop].
    #[live]
    pub stop_icon: LiveValue,

    /// If this widget should provoke sending a message or stopping the current response.
    #[rust]
    pub task: Task,

    /// If this widget should be interactive or not.
    #[rust]
    pub interactivity: Interactivity,

    /// Capabilities of the currently selected bot
    #[rust]
    pub bot_capabilities: Option<BotCapabilities>,
}

impl LiveHook for PromptInput {
    #[allow(unused)]
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.update_button_visibility(cx);
    }
}

impl Widget for PromptInput {
    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.deref.set_text(cx, v);
    }

    fn text(&self) -> String {
        self.deref.text()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.ui_runner().handle(cx, event, scope, self);

        if self.button(id!(attach)).clicked(event.actions()) {
            let ui = self.ui_runner();
            Attachment::pick_multiple(move |result| match result {
                Ok(attachments) => {
                    ui.defer_with_redraw(move |me, _, _| {
                        let mut list = me.attachment_list_ref();
                        list.write().attachments.extend(attachments);
                        list.write().on_tap(move |list, index| {
                            list.attachments.remove(index);
                        });
                    });
                }
                Err(_) => {}
            });
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let button = self.button(id!(submit));

        match self.task {
            Task::Send => {
                button.apply_over(
                    cx,
                    live! {
                        draw_icon: {
                            svg_file: (self.send_icon),
                        }
                    },
                );
            }
            Task::Stop => {
                button.apply_over(
                    cx,
                    live! {
                        draw_icon: {
                            svg_file: (self.stop_icon),
                        }
                    },
                );
            }
        }

        match self.interactivity {
            Interactivity::Enabled => {
                button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            enabled: 1.0
                        }
                    },
                );
                button.set_enabled(cx, true);
            }
            Interactivity::Disabled => {
                button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            enabled: 0.0
                        }
                    },
                );
                button.set_enabled(cx, false);
            }
        }

        self.deref.draw_walk(cx, scope, walk)
    }
}

impl PromptInput {
    /// Reset this prompt input erasing text, removing attachments, etc.
    ///
    /// Shadows the [`CommandTextInput::reset`] method.
    pub fn reset(&mut self, cx: &mut Cx) {
        self.deref.reset(cx);
        self.attachment_list_ref().write().attachments.clear();
    }

    /// Check if the submit button or the return key was pressed.
    ///
    /// Note: To know what the button submission means, check [Self::task] or
    /// the utility methods.
    pub fn submitted(&self, actions: &Actions) -> bool {
        let submit = self.button(id!(submit));
        let input = self.text_input_ref();
        (submit.clicked(actions) || input.returned(actions).is_some())
            && self.interactivity == Interactivity::Enabled
    }

    pub fn call_pressed(&self, actions: &Actions) -> bool {
        self.button(id!(audio)).clicked(actions)
    }

    /// Shorthand to check if [Self::task] is set to [Task::Send].
    pub fn has_send_task(&self) -> bool {
        self.task == Task::Send
    }

    /// Shorthand to check if [Self::task] is set to [Task::Stop].
    pub fn has_stop_task(&self) -> bool {
        self.task == Task::Stop
    }

    /// Allows submission.
    pub fn enable(&mut self) {
        self.interactivity = Interactivity::Enabled;
    }

    /// Disallows submission.
    pub fn disable(&mut self) {
        self.interactivity = Interactivity::Disabled;
    }

    /// Shorthand to set [Self::task] to [Task::Send].
    pub fn set_send(&mut self) {
        self.task = Task::Send;
    }

    /// Shorthand to set [Self::task] to [Task::Stop].
    pub fn set_stop(&mut self) {
        self.task = Task::Stop;
    }

    pub(crate) fn attachment_list_ref(&self) -> AttachmentListRef {
        self.attachment_list(id!(attachments))
    }

    /// Set the capabilities of the currently selected bot
    pub fn set_bot_capabilities(&mut self, cx: &mut Cx, capabilities: Option<BotCapabilities>) {
        self.bot_capabilities = capabilities;
        self.update_button_visibility(cx);
    }

    /// Update button visibility based on bot capabilities
    fn update_button_visibility(&mut self, cx: &mut Cx) {
        let supports_attachments = self
            .bot_capabilities
            .as_ref()
            .map(|caps| caps.supports_attachments())
            .unwrap_or(false);

        let supports_realtime = self
            .bot_capabilities
            .as_ref()
            .map(|caps| caps.supports_realtime())
            .unwrap_or(false);

        // Show attach button only if bot supports attachments AND we're on a supported platform
        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_arch = "wasm32"
        ))]
        self.button(id!(attach))
            .set_visible(cx, supports_attachments);

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_arch = "wasm32"
        )))]
        self.button(id!(attach)).set_visible(cx, false);

        // Show audio/call button only if bot supports realtime, we're on a supported platform
        // and realtime feature is enabled
        #[cfg(not(target_arch = "wasm32"))]
        #[cfg(feature = "realtime")]
        self.button(id!(audio)).set_visible(cx, supports_realtime);

        if supports_realtime {
            self.interactivity = Interactivity::Disabled;
            self.text_input_ref().set_is_read_only(cx, true);
            self.text_input_ref()
                .set_text(cx, "For realtime models, use the audio feature ->");
            self.redraw(cx);
        } else {
            self.interactivity = Interactivity::Enabled;
            self.text_input_ref().set_is_read_only(cx, false);
            self.text_input_ref().set_text(cx, "");
            self.redraw(cx);
        }
    }
}

impl PromptInputRef {
    /// Immutable access to the underlying [[PromptInput]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<'_, PromptInput> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [[PromptInput]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<'_, PromptInput> {
        self.borrow_mut().unwrap()
    }

    /// Immutable reader to the underlying [[PromptInput]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read_with<R>(&self, f: impl FnOnce(&PromptInput) -> R) -> R {
        f(&*self.read())
    }

    /// Mutable writer to the underlying [[PromptInput]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write_with<R>(&mut self, f: impl FnOnce(&mut PromptInput) -> R) -> R {
        f(&mut *self.write())
    }
}
