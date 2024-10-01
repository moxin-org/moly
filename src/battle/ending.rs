use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    Ending = {{Ending}} {
        flow: Down,
        align: {x: 0.5, y: 0.5},
        <Image> {
            margin: {bottom: (LG_GAP)},
            source: dep("crate://self/resources/icons/prerendered/output/ok.png"),
            width: 250,
            height: 250,
        }
        <Label> {
            draw_text: {
                color: #000,
                text_style: <BOLD_FONT> { font_size: 14 }
            }
            text: "You're done! Thank you for participating."
        }
        button = <MolyButton> {
            // For some reason, putting a margin bottom in the label doesn't
            // work propertly.
            margin: {top: (SM_GAP)},
            text: "Finish",
            draw_bg: {
                color: #000,
            },
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Ending {
    #[deref]
    view: View,
}

impl Widget for Ending {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Ending {
    fn button_ref(&self) -> ButtonRef {
        self.button(id!(button))
    }
}

impl Ending {
    pub fn ended(&self, actions: &Actions) -> bool {
        self.button_ref().clicked(actions)
    }
}

impl EndingRef {
    pub fn ended(&self, actions: &Actions) -> bool {
        self.borrow().map(|s| s.ended(actions)).unwrap_or(false)
    }
}
