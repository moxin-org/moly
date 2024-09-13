use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    AgentSelector = {{AgentSelector}} {
        width: Fill,
        height: 60,
        show_bg: true,
        draw_bg: {
            color: #7777de,
        },
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct AgentSelector {
    #[deref]
    view: View,
}

impl Widget for AgentSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for AgentSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {}
}
