use makepad_widgets::*;
use markdown::MarkdownAction;
use moxin_mae::MaeBackend;

use super::agent_selector::AgentSelectorWidgetExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;
    import crate::battle::prompt::Prompt;
    import crate::battle::agent_selector::AgentSelector;

    GAP = 12;

    BattleScreen = {{BattleScreen}} {
        content = <View> {
            flow: Down,
            visible: false,
            padding: (GAP),
            spacing: (GAP),
            <View> {
                spacing: (GAP),
                <View> {
                    flow: Down,
                    spacing: (GAP),
                    agent_selector = <AgentSelector> {}
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
}

#[derive(Live, Widget)]
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

impl LiveHook for BattleScreen {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // Enable this screen only if there are enough agents, quick solution.
        let agents = MaeBackend::available_agents();
        if agents.len() >= 2 {
            self.view(id!(content)).set_visible(true);
            self.agent_selector(id!(agent_selector)).set_agents(agents);
        }
    }
}
