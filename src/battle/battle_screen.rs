use super::messages::{Message, MessagesWidgetExt};
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;

    SM_GAP = 14;
    MD_GAP = 28;
    SELECTOR_HEIGHT = 45;

    Half = <View> {
        flow: Overlay,
        messages = <Messages> {
            margin: {top: (SELECTOR_HEIGHT + MD_GAP)},
        }
        title = <Label> {} // ex selector
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
            vote = <View> {} // ex prompt
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
        let left_messages = self.messages(id!(left.messages));
        let right_messages = self.messages(id!(right.messages));
        let mut redraw = false;
        let mut scroll_to_bottom = false;

        if scroll_to_bottom {
            left_messages.scroll_to_bottom(cx);
            right_messages.scroll_to_bottom(cx);
        }

        if redraw {
            self.redraw(cx);
        }
    }
}

impl LiveHook for BattleScreen {}
