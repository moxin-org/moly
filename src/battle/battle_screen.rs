use crate::data::chats::chat::MaeAgentResponseFormatter;
use makepad_widgets::*;
use markdown::MarkdownAction;

use super::{
    agent_selector::AgentSelectorWidgetExt,
    mae::Mae,
    messages::{Message, MessagesWidgetExt},
    no_messages::NoMessagesWidgetExt,
    prompt::PromptWidgetExt,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;
    import crate::battle::prompt::Prompt;
    import crate::battle::agent_selector::AgentSelector;
    import crate::battle::no_messages::NoMessages;
    import crate::chat::shared::ChatAgentAvatar;

    SM_GAP = 14;
    MD_GAP = 28;
    SELECTOR_HEIGHT = 45;

    Half = <View> {
        flow: Overlay,
        messages = <Messages> {
            margin: {top: (SELECTOR_HEIGHT + MD_GAP)},
        }
        no_messages = <NoMessages> {
            visible: false,
            margin: {top: (SELECTOR_HEIGHT + MD_GAP)},
        }
        selector = <AgentSelector> {}
    }

    BattleScreen = {{BattleScreen}} {
        content = <View> {
            flow: Down,
            visible: false,
            padding: {top: 40, bottom: 40, left: (MD_GAP), right: (MD_GAP)},

            spacing: (SM_GAP),
            <View> {
                spacing: (MD_GAP),
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
    left_mae: Mae,

    #[rust(Mae::new())]
    right_mae: Mae,
}

impl Widget for BattleScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.left_mae.ensure_initialized(scope);
        self.right_mae.ensure_initialized(scope);
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let left_messages = self.messages(id!(left.messages));
        // let right_messages = self.messages(id!(right.messages));
        let left_no_messages = self.no_messages(id!(left.no_messages));
        let right_no_messages = self.no_messages(id!(right.no_messages));

        if left_messages.len() == 0 {
            let left_agent = self
                .agent_selector(id!(left.selector))
                .selected_agent()
                .unwrap();

            let right_agent = self
                .agent_selector(id!(right.selector))
                .selected_agent()
                .unwrap();

            left_no_messages.set_visible(true);
            left_no_messages.set_agent(left_agent);

            right_no_messages.set_visible(true);
            right_no_messages.set_agent(right_agent);
        } else {
            left_no_messages.set_visible(false);
            right_no_messages.set_visible(false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for BattleScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let prompt = self.prompt(id!(prompt));
        let left_messages = self.messages(id!(left.messages));
        let right_messages = self.messages(id!(right.messages));
        let mut redraw = false;
        let mut scroll_to_bottom = false;

        if prompt.submitted(actions) {
            let text = prompt.text();
            prompt.clear();

            left_messages.add_message(Message::User(text.clone()));
            right_messages.add_message(Message::User(text.clone()));

            let left_agent = self
                .agent_selector(id!(left.selector))
                .selected_agent()
                .unwrap();

            let right_agent = self
                .agent_selector(id!(right.selector))
                .selected_agent()
                .unwrap();

            left_messages.add_message(Message::AgentWriting(left_agent));
            right_messages.add_message(Message::AgentWriting(right_agent));

            self.left_mae.send_prompt(left_agent, text.clone());
            self.right_mae.send_prompt(right_agent, text);

            redraw = true;
            scroll_to_bottom = true;
        }

        self.left_mae
            .responses(actions)
            .map(|r| (r.to_agent(), r.to_text_messgae()))
            .for_each(|(a, m)| {
                left_messages.add_message(Message::Agent(a, m.clone()));
                redraw = true;
                scroll_to_bottom = true;
            });

        self.right_mae
            .responses(actions)
            .map(|r| (r.to_agent(), r.to_text_messgae()))
            .for_each(|(a, m)| {
                right_messages.add_message(Message::Agent(a, m.clone()));
                redraw = true;
                scroll_to_bottom = true;
            });

        for action in actions {
            if let MarkdownAction::LinkNavigated(url) = action.as_widget_action().cast() {
                let _ = robius_open::Uri::new(&url).open();
            }
        }

        if scroll_to_bottom {
            left_messages.scroll_to_bottom(cx);
            right_messages.scroll_to_bottom(cx);
        }

        if redraw {
            self.redraw(cx);
        }
    }
}

impl LiveHook for BattleScreen {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let agents = moxin_mae::MaeBackend::available_agents();

        // Enable this screen only if there are enough agents, quick solution.
        if agents.len() >= 2 {
            self.view(id!(content)).set_visible(true);

            let left_agent = agents[0];
            let right_agent = agents[1];

            let left_selector = self.agent_selector(id!(left.selector));
            let right_selector = self.agent_selector(id!(right.selector));

            left_selector.set_agents(agents.clone());
            left_selector.set_agent(left_agent);

            right_selector.set_agents(agents);
            right_selector.set_agent(right_agent);
        }
    }
}
