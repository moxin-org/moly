use makepad_widgets::*;
use markdown::MarkdownAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::chat::chat_panel::ChatPanel;
    import crate::chat::chat_history::ChatHistory;
    import crate::chat::chat_params::ChatParams;

    BattleScreen = {{BattleScreen}} {
        width: Fill,
        height: Fill,
        <View> {
            width: Fill,
            height: Fill,
            align: {x: 0.5},
            padding: {top: 48, bottom: 48 }

            chat_panel = <ChatPanel> {}
        }
        <View> {
            width: Fill,
            height: Fill,
            align: {x: 0.5},
            padding: {top: 48, bottom: 48 }

            chat_panel = <ChatPanel> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BattleScreen {
    #[deref]
    view: View,
}

impl Widget for BattleScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // TODO This check is actually copied from Makepad view.rs file
        // It's not clear why it's needed here, but without this line
        // the "View all files" link in Discover section does not work after visiting the chat screen
        if self.visible || !event.requires_visibility() {
            self.view.handle_event(cx, event, scope);
            self.widget_match_event(cx, event, scope);
        }
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
