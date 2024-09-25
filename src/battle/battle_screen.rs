use super::{
    battle_service::BattleService,
    battle_sheet::{Round, Sheet},
    messages::MessagesWidgetExt,
    vote::VoteWidgetExt,
};
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
    sheet: Option<Sheet>,
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
        let mut redraw = false;
        let mut scroll_to_bottom = false;

        if let Some(weight) = self.vote(id!(vote)).voted(actions) {
            if let Some(round) = self.current_round_mut() {
                round.weight = weight.into();
                self.update_view();
                redraw = true;
                scroll_to_bottom = true;
            }
        }

        if let Some(error) = self.service.failed(actions) {
            eprintln!("{}", error);
        }

        if let Some(sheet) = self.service.battle_sheet_downloaded(actions) {
            self.sheet = sheet.clone().into();
            self.update_view();

            redraw = true;
            scroll_to_bottom = true;
        }

        if scroll_to_bottom {
            self.messages(id!(left.messages)).scroll_to_bottom(cx);
            self.messages(id!(right.messages)).scroll_to_bottom(cx);
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

impl BattleScreen {
    fn current_round_index(&self) -> Option<usize> {
        self.sheet
            .as_ref()
            .map(|s| s.rounds.iter().position(|r| r.weight.is_none()))
            .flatten()
    }

    fn current_round(&self) -> Option<&Round> {
        self.current_round_index()
            .map(|i| self.sheet.as_ref().map(|s| &s.rounds[i]))
            .flatten()
    }

    fn current_round_mut(&mut self) -> Option<&mut Round> {
        self.current_round_index()
            .map(|i| self.sheet.as_mut().map(|s| &mut s.rounds[i]))
            .flatten()
    }

    fn update_view(&self) {
        if let Some(index) = self.current_round_index() {
            let round = self.current_round().unwrap();

            self.messages(id!(left.messages))
                .set_messages(round.chats[0].messages.clone());

            self.messages(id!(right.messages))
                .set_messages(round.chats[1].messages.clone());
        }
    }
}
