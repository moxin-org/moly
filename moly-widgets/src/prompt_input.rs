use std::cell::{Ref, RefMut};

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    pub PromptInput = {{PromptInput}} <CommandTextInput> {
        send_icon: dep("crate://self/assets/send.svg"),
        stop_icon: dep("crate://self/assets/stop.svg"),

        persistent = {
            center = {
                text_input = {
                    empty_message: "Prompt...",
                    draw_bg: {
                        fn pixel(self) -> vec4 {
                            return vec4(0.);
                        }
                    }
                    draw_text: {
                        // text_style:<REGULAR_FONT>{font_size: 10},
                        instance prompt_enabled: 1.0

                        fn get_color(self) -> vec4 {
                            return mix(#98A2B3, #000, self.prompt_enabled)
                        }
                    }
                    draw_selection: {
                        fn pixel(self) -> vec4 {
                            return #bbb;
                        }
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
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                let center = self.rect_size * 0.5;
                                let radius = min(self.rect_size.x, self.rect_size.y) * 0.5;

                                sdf.circle(center.x, center.y, radius);
                                sdf.fill_keep(#000);

                                return sdf.result
                            }
                        }
                        icon_walk: {
                            width: 12,
                            height: 12
                            margin: {top: 0, left: -4},
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

#[derive(Live, LiveHook, Widget)]
pub struct PromptInput {
    #[deref]
    deref: CommandTextInput,

    #[live]
    send_icon: LiveValue,

    #[live]
    stop_icon: LiveValue,

    #[rust]
    pub task: Task,

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
                button.set_enabled(cx, true);
            }
            Interactivity::Disabled => {
                button.set_enabled(cx, false);
            }
        }

        self.deref.draw_walk(cx, scope, walk)
    }
}

impl PromptInput {
    pub fn submitted(&self, actions: &Actions) -> bool {
        let submit = self.button(id!(submit));
        let input = self.text_input_ref();
        submit.clicked(actions) || input.returned(actions).is_some()
    }

    pub fn has_send_task(&self) -> bool {
        self.task == Task::Send
    }

    pub fn has_stop_task(&self) -> bool {
        self.task == Task::Stop
    }

    pub fn enable(&mut self) {
        self.interactivity = Interactivity::Enabled;
    }

    pub fn disable(&mut self) {
        self.interactivity = Interactivity::Disabled;
    }

    pub fn set_send(&mut self) {
        self.task = Task::Send;
    }

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
