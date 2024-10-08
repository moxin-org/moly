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
        <Image> {
            margin: {bottom: (MD_GAP)},
            source: dep("crate://self/resources/icons/prerendered/output/error.png"),
            width: 250,
            height: 250,
        }
        message = <Label> {
            draw_text: {
                color: #000,
                text_style: <BOLD_FONT> { font_size: 14 }
            }
            text: "An error occurred."
        }
        retry = <MolyButton> {
            text: "Dismiss",
            margin: {top: (SM_GAP)},
            padding: {top: (SM_GAP), bottom: (SM_GAP), left: (MD_GAP), right: (MD_GAP)},
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

    #[rust]
    recovery_cb: Option<Box<dyn FnOnce() + 'static>>,
}

impl Widget for Failure {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        if let Event::Actions(actions) = event {
            if self.button(id!(retry)).clicked(actions) {
                if let Some(recovery_cb) = self.recovery_cb.take() {
                    recovery_cb();
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Failure {
    pub fn set_message(&self, message: &str) {
        self.label(id!(message)).set_text(message);
    }

    pub fn set_recovery_cb(&mut self, recovery_cb: impl FnOnce() + 'static) {
        self.recovery_cb = Some(Box::new(recovery_cb));
    }
}

impl FailureRef {
    pub fn set_message(&self, message: &str) {
        if let Some(failure) = self.borrow_mut() {
            failure.set_message(message);
        }
    }

    pub fn set_recovery_cb(&self, recovery_cb: impl FnOnce() + 'static) {
        if let Some(mut failure) = self.borrow_mut() {
            failure.set_recovery_cb(recovery_cb);
        }
    }
}
