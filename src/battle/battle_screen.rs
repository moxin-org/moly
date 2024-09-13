use makepad_widgets::*;
use markdown::MarkdownAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;
    import crate::battle::prompt::Prompt;
    import crate::battle::agent_selector::AgentSelector;

    GAP = 12;

    BattleScreen = {{BattleScreen}} {
        flow: Down,
        padding: (GAP),
        spacing: (GAP),
        <View> {
            spacing: (GAP),
            <View> {
                flow: Down,
                spacing: (GAP),
                <AgentSelector> {}
                <Messages> {}
            }
            <View> {
                flow: Down,
                spacing: (GAP),
                <AgentSelector> {}
                <Messages> {}
            }
        }
        <Prompt> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BattleScreen {
    #[deref]
    view: View,
}

impl Widget for BattleScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for BattleScreen {
    fn handle_actions(&mut self, _cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            if let MarkdownAction::LinkNavigated(url) = action.as_widget_action().cast() {
                let _ = robius_open::Uri::new(&url).open();
            }
        }
    }
}
