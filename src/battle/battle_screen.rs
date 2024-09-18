use makepad_widgets::*;
use markdown::MarkdownAction;
use moxin_mae::{MaeAgent, MaeBackend};

use crate::data::{
    chats::{chat::ChatEntity, Chats},
    downloads::Downloads,
    search::Search,
    store::Store,
};

use super::{half::HalfWidgetExt, prompt::PromptWidgetExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::prompt::Prompt;
    import crate::battle::half::Half;
    import crate::chat::chat_panel::ChatPanel;

    GAP = 12;

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
            <View> {
                height: Fit,
                padding: {left: 20, right: 20},
                prompt = <Prompt> {}
            }
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
        if prompt.submitted(actions) {
            println!("Prompt submitted");
            let left = self.half(id!(left));
            let right = self.half(id!(right));

            let message = prompt.text();
            left.send_message(message.clone());
            right.send_message(message);
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
