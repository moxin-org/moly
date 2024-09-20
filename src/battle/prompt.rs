use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    Prompt = {{Prompt}} <RoundedView> {
        height: Fit,
        align: {y: 0.5}
        padding: {top: 8, bottom: 8, left: 10, right: 10},
        draw_bg: {
            border_width: 1.0,
            border_color: #D0D5DD,
            color: #fff,
            radius: 12.0
        }

        send_icon: dep("crate://self/resources/icons/prompt.svg")
        stop_icon: dep("crate://self/resources/icons/stop.svg")

        input = <MoxinTextInput> {
            draw_text: {
                text_style: <REGULAR_FONT> { font_size: 11 },
            }
            draw_bg: {
                color: vec4(0, 0, 0, 0),
            },
            width: Fill,
            height: Fit,
            empty_message: "Enter a message",
        },
        submit = <MoxinButton> {
            height: 35,
            width: 35,
            draw_bg: {
                radius: 8,
                color: #000,
            },
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
pub struct Prompt {
    #[deref]
    view: View,

    #[live]
    send_icon: LiveValue,

    #[live]
    stop_icon: LiveValue,

    #[rust]
    task: Task,

    #[rust]
    interactivity: Interactivity,
}

impl Widget for Prompt {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.button(id!(submit)).apply_over(
            cx,
            live! {
                draw_icon: {
                    svg_file: (self.send_icon),
                }
            },
        );

        self.view.draw_walk(cx, scope, walk)
    }
}

impl Prompt {
    pub fn submitted(&self, actions: &Actions) -> bool {
        let submit = self.button(id!(submit));
        let input = self.text_input(id!(input));
        submit.clicked(actions) || input.returned(actions).is_some()
    }

    pub fn text(&self) -> String {
        self.text_input(id!(input)).text()
    }

    pub fn clear(&self) {
        self.text_input(id!(input)).set_text("");
    }
}

impl PromptRef {
    pub fn submitted(&self, actions: &Actions) -> bool {
        self.borrow()
            .map(|inner| inner.submitted(actions))
            .unwrap_or(false)
    }

    pub fn text(&self) -> String {
        self.borrow().map(|inner| inner.text()).unwrap_or_default()
    }

    pub fn clear(&self) {
        self.borrow().map(|inner| inner.clear());
    }
}
