use std::sync::mpsc::channel;

use makepad_widgets::*;
use markdown::MarkdownAction;
use moxin_mae::{MaeAgentCommand, MaeBackend};
use std::sync::mpsc::Sender;

use crate::data::{chats::chat::MaeAgentResponseFormatter, store::Store};

use super::{
    agent_selector::AgentSelectorWidgetExt,
    mae::{self, Mae},
    messages::{Message, MessagesWidgetExt},
    prompt::PromptWidgetExt,
};

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
        selector = <AgentSelector> {}
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

    #[rust(Mae::new())]
    mae: Mae,
}

impl Widget for BattleScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.mae.ensure_initialized(scope);
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
        let mut redraw = false;

        if prompt.submitted(actions) {
            let text = prompt.text();

            left_messages.add_message(Message::User(text.clone()));
            right_messages.add_message(Message::User(text.clone()));

            left_messages.add_message(Message::AgentWriting);
            right_messages.add_message(Message::AgentWriting);

            redraw = true;

            let left_agent = self
                .agent_selector(id!(left.selector))
                .selected_agent()
                .unwrap();
            let right_agent = self
                .agent_selector(id!(right.selector))
                .selected_agent()
                .unwrap();
            self.mae.send_prompt(left_agent, text.clone());
            self.mae.send_prompt(right_agent, text);
        }

        mae::responses(actions)
            .map(|r| r.to_text_messgae())
            .for_each(|m| {
                left_messages.add_message(Message::Agent(m.clone()));
                right_messages.add_message(Message::Agent(m));
                redraw = true;
            });

        for action in actions {
            if let MarkdownAction::LinkNavigated(url) = action.as_widget_action().cast() {
                let _ = robius_open::Uri::new(&url).open();
            }
        }

        if redraw {
            self.redraw(cx);
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
