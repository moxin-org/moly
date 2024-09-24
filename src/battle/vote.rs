use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    VoteButton = <MoxinButton> {
        height: 35,
        width: 35,
        draw_bg: {
            radius: 8,
            color: #000,
        },
    }

    Vote = {{Vote}} <View> {
        height: Fit,
        align: {x: 0.5}
        a2 = <VoteButton> {text: "A"}
        a1 = <VoteButton> {text: "A-"}
        o0 = <VoteButton> {text: "0"}
        b1 = <VoteButton> {text: "B-"}
        b2 = <VoteButton> {text: "B"}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Vote {
    #[deref]
    view: View,
}

impl Widget for Vote {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Vote {
    pub fn voted(&self, actions: &Actions) -> Option<i8> {
        if self.button(id!(a2)).clicked(actions) {
            return Some(-2);
        }

        if self.button(id!(a1)).clicked(actions) {
            return Some(-1);
        }

        if self.button(id!(o0)).clicked(actions) {
            return Some(0);
        }

        if self.button(id!(b1)).clicked(actions) {
            return Some(1);
        }

        if self.button(id!(b2)).clicked(actions) {
            return Some(2);
        }

        None
    }
}

impl VoteRef {
    pub fn voted(&self, actions: &Actions) -> Option<i8> {
        self.borrow().map(|inner| inner.voted(actions)).flatten()
    }
}
