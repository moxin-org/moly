use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub PromptInput = {{PromptInput}} <CommandTextInput> {
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
            }
        }
    }
}

#[derive(Default)]
enum Task {
    #[default]
    Submit,
    Stop,
}

#[derive(Default)]
enum Interactivity {
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
    task: Task,

    #[rust]
    interactivity: Interactivity,
}

impl Widget for PromptInput {
    fn text(&self) -> String {
        self.text_input(id!(input)).text()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // self.button(id!(submit)).apply_over(
        //     cx,
        //     live! {
        //         draw_icon: {
        //             svg_file: (self.send_icon),
        //         }
        //     },
        // );

        self.deref.draw_walk(cx, scope, walk)
    }
}

impl PromptInput {
    pub fn submitted(&self, actions: &Actions) -> bool {
        // let submit = self.button(id!(submit));
        // let input = self.text_input(id!(input));
        // submit.clicked(actions) || input.returned(actions).is_some()

        self.text_input_ref().returned(actions).is_some()
    }
}

impl PromptInputRef {
    pub fn submitted(&self, actions: &Actions) -> bool {
        self.borrow()
            .map(|inner| inner.submitted(actions))
            .unwrap_or(false)
    }
}
