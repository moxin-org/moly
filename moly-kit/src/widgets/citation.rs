use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub Citation = {{Citation}} {
        height: Fit,
        width: Fit,
        <View> {
            width: 64,
            height: 64,
            show_bg: true,
            draw_bg: {
                color: #f00
            }
        }
        <View> {
            width: 128,
            height: 64,
            show_bg: true,
            draw_bg: {
                color: #0f0
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Citation {
    #[deref]
    deref: View,
}

impl Widget for Citation {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}
