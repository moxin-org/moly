use makepad_widgets::*;
use markdown::MarkdownAction;
use moxin_mae::MaeBackend;

use super::{messages::MessagesWidgetExt, prompt::PromptWidgetExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;
    import crate::battle::prompt::Prompt;
    import crate::battle::agent_selector::AgentSelector;

    GAP = 12;

    Half = <View> {
        flow: Overlay,
        messages = <Messages> {
            margin: {top: (45 + GAP)},
        }
        <AgentSelector> {}
    }

    BattleScreen = {{BattleScreen}} {
        content = <View> {
            flow: Down,
            visible: false,
            padding: (GAP),
            spacing: (GAP),
            <View> {
                spacing: (GAP),
                left = <Half> {}
                right = <Half> {}
            }
            prompt = <Prompt> {}
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
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let prompt = self.prompt(id!(prompt));
        let left_messages = self.messages(id!(left.messages));
        let right_messages = self.messages(id!(right.messages));

        if prompt.submitted(actions) {
            let text = prompt.text();

            left_messages.add_message(text.clone());
            right_messages.add_message(text);

            left_messages.redraw(cx);
            right_messages.redraw(cx);
        }

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
        if MaeBackend::available_agents().len() >= 2 {
            self.view(id!(content)).set_visible(true);
        }
    }
}
