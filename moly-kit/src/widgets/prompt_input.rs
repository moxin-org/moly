use makepad_widgets::*;
use std::cell::{Ref, RefMut};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

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
                text_input = {
                    empty_text: "Start typing",
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
#[derive(Live, LiveHook, Widget)]
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
                button.apply_over(cx, live! {
                    draw_bg: {
                        enabled: 1.0
                    }
                });
                button.set_enabled(cx, true);
            }
            Interactivity::Disabled => {
                button.apply_over(cx, live! {
                    draw_bg: {
                        enabled: 0.0
                    }
                });
                button.set_enabled(cx, false);
            }
        }

        self.deref.draw_walk(cx, scope, walk)
    }
}

impl PromptInput {
    /// Check if the submit button or the return key was pressed.
    ///
    /// Note: To know what the button submission means, check [Self::task] or
    /// the utility methods.
    pub fn submitted(&self, actions: &Actions) -> bool {
        let submit = self.button(id!(submit));
        let input = self.text_input_ref();
        submit.clicked(actions) || input.returned(actions).is_some()
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
}

impl PromptInputRef {
    /// Immutable access to the underlying [[PromptInput]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<PromptInput> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [[PromptInput]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<PromptInput> {
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
