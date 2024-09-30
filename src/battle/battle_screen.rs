use super::{
    failure::{FailureRef, FailureWidgetExt},
    messages::{MessagesRef, MessagesWidgetExt},
    opening::{OpeningRef, OpeningWidgetExt},
    vote::{VoteRef, VoteWidgetExt},
};
use crate::data::battle::{self, Service};
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::messages::Messages;
    import crate::battle::vote::Vote;
    import crate::battle::opening::Opening;
    import crate::battle::spinner::Spinner;
    import crate::battle::styles::*;
    import crate::battle::failure::Failure;
    import crate::battle::ending::Ending;

    Half = <View> {
        flow: Down,
        spacing: (MD_GAP),
        title_layout = <View> {
            height: Fit,
            align: { x: 0.5 },
            title = <Label> {
                draw_text: {
                    color: #000,
                    text_style: <BOLD_FONT> { font_size: 14 }
                }
            }
        }
        <RoundedView> {
            padding: {top: 24, bottom: 24, left: 20, right: 20},
            draw_bg: {
                color: #fff,
                border_color: #f6f6f6,
                border_width: 1.5,
                radius: 15,
            }
            messages = <Messages> {}
        }
    }

    BattleScreen = {{BattleScreen}} {
        flow: Overlay,
        show_bg: true,
        draw_bg: {
            color: #F8F8F8,
        }
        opening = <Opening> {}
        ending = <Ending> {
            visible: false,
        }
        loading = <View> {
            visible: false,
            align: {x: 0.5, y: 0.5},
            <Spinner> {}
        }
        round = <View> {
            visible: false,
            flow: Down,
            padding: {top: 40, bottom: 40, left: (MD_GAP), right: (MD_GAP)},
            spacing: (MD_GAP),
            <View> {
                flow: Overlay,
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
                <View> {
                    align: {x: 0.5}
                    counter = <Label> {
                        draw_text: {
                            color: #000,
                            text_style: <BOLD_FONT> { font_size: 14 }
                        }
                    }
                }
            }
            vote = <Vote> {}
        }
        failure = <Failure> {
            visible: false,
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct BattleScreen {
    #[deref]
    view: View,

    #[rust(battle::Service::new())]
    // Issue: For some reason, makepad's macro doesn't like `battle::Service`
    // in this line.
    service: Service,

    #[rust]
    sheet: Option<battle::Sheet>,
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
        if self.opening_ref().submitted(actions) {
            self.handle_opening_submitted(cx);
        }

        if let Some(weight) = self.vote_ref().voted(actions) {
            self.handle_voted(cx, weight);
        }

        if let Some(error) = self.service.failed(actions).map(|e| e.to_string()) {
            self.handle_failure(cx, error);
        }

        if let Some(sheet) = self.service.battle_sheet_downloaded(actions).cloned() {
            self.handle_battle_sheet_downloaded(cx, sheet);
        }

        if self.service.battle_sheet_sent(actions) {
            self.handle_battle_sheet_sent(cx);
        }

        if self.failure_ref().retried(actions) {
            self.handle_retry(cx);
        }
    }
}

// widget accessors
impl BattleScreen {
    fn left_messages_ref(&self) -> MessagesRef {
        self.messages(id!(left.messages))
    }

    fn right_messages_ref(&self) -> MessagesRef {
        self.messages(id!(right.messages))
    }

    fn opening_ref(&self) -> OpeningRef {
        self.opening(id!(opening))
    }

    fn ending_ref(&self) -> ViewRef {
        self.view(id!(ending))
    }

    fn round_ref(&self) -> ViewRef {
        self.view(id!(round))
    }

    fn loading_ref(&self) -> ViewRef {
        self.view(id!(loading))
    }

    fn counter_ref(&self) -> LabelRef {
        self.label(id!(counter))
    }

    fn vote_ref(&self) -> VoteRef {
        self.vote(id!(vote))
    }

    fn failure_ref(&self) -> FailureRef {
        self.failure(id!(failure))
    }
}

// event handlers
impl BattleScreen {
    fn handle_round_updated(&mut self, cx: &mut Cx) {
        if let Some(index) = self.current_round_index() {
            let round = self.current_round().unwrap();

            self.left_messages_ref()
                .set_messages(round.chats[0].messages.clone());

            self.right_messages_ref()
                .set_messages(round.chats[1].messages.clone());

            self.left_messages_ref().scroll_to_bottom(cx);
            self.right_messages_ref().scroll_to_bottom(cx);

            let rounds_count = self.sheet.as_ref().unwrap().rounds.len();
            self.counter_ref()
                .set_text(&format!("{} / {}", index + 1, rounds_count));
        } else {
            self.service.send_battle_sheet(self.sheet.take().unwrap());
            self.round_ref().set_visible(false);
            self.loading_ref().set_visible(true);
        }

        self.redraw(cx);
    }

    fn handle_battle_sheet_sent(&mut self, cx: &mut Cx) {
        self.show_frame(self.ending_ref().widget_uid());
        self.redraw(cx);
    }

    fn handle_battle_sheet_downloaded(&mut self, cx: &mut Cx, sheet: battle::Sheet) {
        self.sheet = Some(sheet);
        self.show_frame(self.round_ref().widget_uid());
        self.handle_round_updated(cx);
        self.redraw(cx);
    }

    fn handle_voted(&mut self, cx: &mut Cx, weight: i8) {
        if let Some(round) = self.current_round_mut() {
            round.weight = Some(weight);
            self.handle_round_updated(cx);
            self.redraw(cx);
        }
    }

    fn handle_opening_submitted(&mut self, cx: &mut Cx) {
        let code = self.opening_ref().code();
        self.service.download_battle_sheet(code);
        self.show_frame(self.loading_ref().widget_uid());
        self.redraw(cx);
    }

    fn handle_failure(&mut self, cx: &mut Cx, error: String) {
        self.failure_ref().set_message(&error);
        self.show_frame(self.failure_ref().widget_uid());
        self.redraw(cx);
    }

    fn handle_retry(&mut self, cx: &mut Cx) {
        self.show_frame(self.opening_ref().widget_uid());
        self.redraw(cx);
    }
}

// other stuff
impl BattleScreen {
    fn current_round_index(&self) -> Option<usize> {
        self.sheet
            .as_ref()
            .map(|s| s.rounds.iter().position(|r| r.weight.is_none()))
            .flatten()
    }

    fn current_round(&self) -> Option<&battle::Round> {
        self.current_round_index()
            .map(|i| self.sheet.as_ref().map(|s| &s.rounds[i]))
            .flatten()
    }

    fn current_round_mut(&mut self) -> Option<&mut battle::Round> {
        self.current_round_index()
            .map(|i| self.sheet.as_mut().map(|s| &mut s.rounds[i]))
            .flatten()
    }

    fn show_frame(&self, uid: WidgetUid) {
        self.loading_ref()
            .set_visible(self.loading_ref().widget_uid() == uid);

        self.round_ref()
            .set_visible(self.round_ref().widget_uid() == uid);

        self.ending_ref()
            .set_visible(self.ending_ref().widget_uid() == uid);

        self.failure_ref().borrow_mut().unwrap().visible = self.failure_ref().widget_uid() == uid;

        self.opening_ref().borrow_mut().unwrap().visible = self.opening_ref().widget_uid() == uid;
    }
}
