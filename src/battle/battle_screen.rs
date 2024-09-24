use super::{battle_service::BattleService, messages::MessagesWidgetExt, vote::VoteWidgetExt};
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;
    import crate::battle::vote::Vote;

    SM_GAP = 14;
    MD_GAP = 28;
    SELECTOR_HEIGHT = 45;

    Half = <View> {
        flow: Overlay,
        messages = <Messages> {
            margin: {top: (SELECTOR_HEIGHT + MD_GAP)},
        }
        title_layout = <View> {
            height: Fit,
            align: { x: 0.5 },
            title = <Label> {
                draw_text: {
                    color: #000,
                    text_style: <BOLD_FONT> { font_size: 18 }
                }
            }
        }
    }

    BattleScreen = {{BattleScreen}} {
        content = <View> {
            flow: Down,
            padding: {top: 40, bottom: 40, left: (MD_GAP), right: (MD_GAP)},

            spacing: (SM_GAP),
            <View> {
                spacing: (MD_GAP),
                left = <Half> {
                    title_layout = {
                        title = { text: "Agent A" }
                    }
                }
                right = <Half> {
                    title_layout = {
                        title = { text: "Agent B" }
                    }
                }
            }
            vote = <Vote> {}
        }
    }
}

#[derive(Live, Widget)]
pub struct BattleScreen {
    #[deref]
    view: View,

    #[rust(BattleService::new())]
    service: BattleService,

    #[rust]
    round_index: usize,
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

        if let Some(weight) = self.vote(id!(vote)).voted(actions) {
            println!("Voted: {}", weight);
            self.round_index = self.round_index + 1;
        }

        if let Some(error) = self.service.failed(actions) {
            eprintln!("{}", error);
        }

        if let Some(sheet) = self.service.battle_sheet_downloaded(actions) {
            self.round_index = 0;
            left_messages.set_messages(sheet.rounds[0].chats[0].messages.clone());
            right_messages.set_messages(sheet.rounds[0].chats[1].messages.clone());
            redraw = true;
            scroll_to_bottom = true;
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
        self.service.download_battle_sheet("abc123".into());
    }
}
