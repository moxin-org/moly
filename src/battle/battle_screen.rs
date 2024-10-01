use crate::data::battle;

use super::{
    dragonfly::Dragonfly,
    ending::{EndingRef, EndingWidgetExt},
    failure::{FailureRef, FailureWidgetExt},
    messages::{MessagesRef, MessagesWidgetExt},
    opening::{OpeningRef, OpeningWidgetExt},
    vote::{VoteRef, VoteWidgetExt},
};
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::messages::Messages;
    import crate::battle::vote::Vote;
    import crate::battle::opening::Opening;
    import crate::battle::opening::Ending;
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
        opening = <Opening> {
            visible: false,
        }
        ending = <Ending> {
            visible: false,
        }
        loading = <View> {
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

#[derive(Live, Widget)]
pub struct BattleScreen {
    #[deref]
    view: View,

    #[rust(Dragonfly::new())]
    dragonfly: Dragonfly,

    #[rust]
    sheet: Option<battle::Sheet>,
}

impl LiveHook for BattleScreen {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.dragonfly.spawn(|df| {
            let sheet = battle::restore_sheet_blocking();
            df.run(move |s: &mut Self, cx| {
                match sheet {
                    Ok(sheet) => {
                        s.sheet = Some(sheet);
                        s.show_frame(s.round_ref().widget_uid());
                    }
                    Err(_) => {
                        s.show_frame(s.opening_ref().widget_uid());
                    }
                }
                s.redraw(cx);
            });
        });
    }
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
            self.handle_opening_submitted();
        }

        if let Some(weight) = self.vote_ref().voted(actions) {
            self.handle_voted(cx, weight);
        }

        if self.failure_ref().retried(actions) {
            self.handle_retry(cx);
        }

        if self.ending_ref().ended(actions) {
            self.handle_ended(cx);
        }

        // TODO: Handle persistence ok.
        // TODO: Handle persistence error.
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

    fn ending_ref(&self) -> EndingRef {
        self.ending(id!(ending))
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
        let sheet = self.sheet.clone().unwrap();
        self.dragonfly.spawn(move |df| {
            if let Err(error) = battle::save_sheet_blocking(&sheet) {
                df.run(move |s: &mut Self, cx| {
                    s.failure_ref().set_message(&error.to_string());
                    s.show_frame(s.failure_ref().widget_uid());
                    // TODO: Handle retry button.
                    s.redraw(cx);
                });

                return;
            }

            if let Some(index) = sheet.current_round_index() {
                let round = sheet.current_round().unwrap();

                df.run(move |s: &mut Self, cx| {
                    s.left_messages_ref()
                        .set_messages(round.chats[0].messages.clone());

                    s.right_messages_ref()
                        .set_messages(round.chats[1].messages.clone());

                    s.left_messages_ref().scroll_to_bottom(cx);
                    s.right_messages_ref().scroll_to_bottom(cx);

                    let rounds_count = sheet.rounds.len();
                    s.counter_ref()
                        .set_text(&format!("{} / {}", index + 1, rounds_count));
                    s.redraw(cx);
                });
            } else {
            }
        });

        // TODO: Address possible concurrency issues and maybe move form here.
        if let Some(sheet) = self.sheet.as_ref() {
            self.service.save_sheet(sheet.clone());
        }

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

    fn handle_voted(&mut self, cx: &mut Cx, weight: i8) {
        if let Some(round) = self.current_round_mut() {
            round.weight = Some(weight);
            self.handle_round_updated(cx);
            self.redraw(cx);
        }
    }

    fn handle_opening_submitted(&mut self) {
        let code = self.opening_ref().code();
        self.dragonfly.spawn(move |df| {
            let sheet = battle::download_sheet_blocking(code);
            df.run(move |s: &mut Self, cx| {
                match sheet {
                    Ok(sheet) => {
                        s.sheet = Some(sheet);
                        s.show_frame(s.round_ref().widget_uid());
                    }
                    Err(error) => {
                        s.failure_ref().set_message(&error.to_string());
                        s.show_frame(s.failure_ref().widget_uid());
                    }
                }
                s.redraw(cx);
            });
        });
    }

    fn handle_retry(&mut self, cx: &mut Cx) {
        self.show_frame(self.opening_ref().widget_uid());
        self.redraw(cx);
    }

    fn handle_ended(&mut self, cx: &mut Cx) {
        // TODO: Handle outcomes.
        self.service.clear_sheet();
        self.opening_ref().clear();
        self.show_frame(self.opening_ref().widget_uid());
        self.redraw(cx);
    }
}

// other stuff
impl BattleScreen {
    fn show_frame(&self, uid: WidgetUid) {
        self.loading_ref()
            .set_visible(self.loading_ref().widget_uid() == uid);

        self.round_ref()
            .set_visible(self.round_ref().widget_uid() == uid);

        self.failure_ref().borrow_mut().unwrap().visible = self.failure_ref().widget_uid() == uid;
        self.opening_ref().borrow_mut().unwrap().visible = self.opening_ref().widget_uid() == uid;
        self.ending_ref().borrow_mut().unwrap().visible = self.ending_ref().widget_uid() == uid;
    }
}
