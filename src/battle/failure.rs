use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    Failure = {{Failure}} {
        flow: Down,
        align: {x: 0.5, y: 0.5},
        spacing: (SM_GAP),
        <Icon> {
            margin: {bottom: (MD_GAP)},
            draw_icon: {
                svg_file: dep("crate://self/resources/icons/discover.svg"),
                fn get_color(self) -> vec4 {
                    return #ff0000;
                }
            }
            icon_walk: {width: 250, height: 250}
        }
        message = <Label> {
            draw_text: {
                color: #000,
                text_style: <BOLD_FONT> { font_size: 14 }
            }
            text: "An error occurred."
        }
        retry = <MoxinButton> {
            text: "Retry",
            draw_bg: {
                color: #000,
            },
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Failure {
    #[deref]
    view: View,
}

impl Widget for Failure {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Failure {
    fn retry_ref(&self) -> ButtonRef {
        self.button(id!(retry))
    }

    pub fn retried(&self, actions: &Actions) -> bool {
        self.retry_ref().clicked(actions)
    }

    pub fn set_message(&self, message: &str) {
        self.label(id!(message)).set_text(message);
    }
}

impl FailureRef {
    pub fn retried(&self, actions: &Actions) -> bool {
        self.borrow().map(|s| s.retried(actions)).unwrap_or(false)
    }

    pub fn set_message(&self, message: &str) {
        if let Some(failure) = self.borrow_mut() {
            failure.set_message(message);
        }
    }
}
