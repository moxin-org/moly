use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub Wrap = {{Wrap}} {}
}

#[derive(Live, Widget, LiveHook)]
pub struct Wrap {
    #[wrap]
    #[live]
    pub wrap: WidgetRef,
}

impl Widget for Wrap {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.wrap.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.wrap.handle_event(cx, event, scope)
    }
}