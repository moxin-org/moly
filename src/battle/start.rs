use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    Start = {{Start}} {
        flow: Down,
        align: {x: 0.5, y: 0.5},
        spacing: (SM_GAP),
        <Label> {
            draw_text: {
                color: #000,
                text_style: <BOLD_FONT> { font_size: 14 }
            }
            text: "Welcome to the battle!"
        }
        input = <MoxinTextInput> {
            empty_message: "Enter your code...",
        }
        button = <MoxinButton> {
            text: "Start",
            draw_bg: {
                color: #000,
            },
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Start {
    #[deref]
    view: View,
}

impl Widget for Start {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Start {
    fn input(&self) -> TextInputRef {
        self.text_input(id!(input))
    }

    fn btn(&self) -> ButtonRef {
        self.button(id!(button))
    }

    pub fn submitted(&self, actions: &Actions) -> bool {
        self.input().returned(actions).is_some() || self.btn().clicked(actions)
    }

    pub fn code(&self) -> String {
        self.input().text()
    }
}

impl StartRef {
    pub fn submitted(&self, actions: &Actions) -> bool {
        self.borrow().map(|s| s.submitted(actions)).unwrap_or(false)
    }

    pub fn code(&self) -> String {
        self.borrow().map(|s| s.code()).unwrap_or_default()
    }
}
