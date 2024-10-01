use crate::data::battle;

use super::{
    ending::{EndingRef, EndingWidgetExt},
    failure::{FailureRef, FailureWidgetExt},
    messages::{MessagesRef, MessagesWidgetExt},
    opening::{OpeningRef, OpeningWidgetExt},
    ui_runner::UiRunner,
    vote::{VoteRef, VoteWidgetExt},
};
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    import crate::battle::messages::Messages;
    import crate::battle::vote::Vote;
    import crate::battle::opening::Opening;
    import crate::battle::spinner::Spinner;
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

    #[rust(UiRunner::new())]
    ui_runner: UiRunner,

    #[rust]
    sheet: Option<battle::Sheet>,
}

impl LiveHook for BattleScreen {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let ui = self.ui_runner;
        std::thread::spawn(move || {
            let sheet = battle::restore_sheet_blocking();
            ui.run(move |s: &mut Self, cx| {
                match sheet {
                    Ok(sheet) => {
                        let completed = sheet.is_completed();
                        s.set_sheet(Some(sheet));

                        if completed {
                            s.show_frame(s.ending_ref().widget_uid());
                        } else {
                            s.show_frame(s.round_ref().widget_uid());
                        }
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
        self.ui_runner.handle(cx, event, self);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for BattleScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.opening_ref().submitted(actions) {
            self.handle_opening_submit(cx);
        }

        if let Some(weight) = self.vote_ref().voted(actions) {
            self.handle_vote(weight);
        }

        if self.failure_ref().retried(actions) {
            self.handle_retry(cx);
        }

        if self.ending_ref().ended(actions) {
            self.handle_end();
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
    fn handle_opening_submit(&mut self, cx: &mut Cx) {
        self.show_frame(self.loading_ref().widget_uid());
        self.redraw(cx);

        let code = self.opening_ref().code();
        let ui = self.ui_runner;
        std::thread::spawn(move || match battle::download_sheet_blocking(code) {
            Ok(sheet) => {
                if let Err(error) = battle::save_sheet_blocking(&sheet) {
                    ui.run(move |s: &mut Self, cx| {
                        s.failure_ref().set_message(&error.to_string());
                        s.show_frame(s.failure_ref().widget_uid());
                        s.redraw(cx);
                    });

                    return;
                }

                ui.run(move |s: &mut Self, cx| {
                    s.set_sheet(Some(sheet));
                    s.show_frame(s.round_ref().widget_uid());
                    s.redraw(cx);
                });
            }
            Err(error) => {
                ui.run(move |s: &mut Self, cx| {
                    s.failure_ref().set_message(&error.to_string());
                    s.show_frame(s.failure_ref().widget_uid());
                    s.redraw(cx);
                });
            }
        });
    }

    fn handle_vote(&mut self, weight: i8) {
        let sheet = self.sheet.as_mut().unwrap();
        sheet.current_round_mut().unwrap().vote = Some(weight);

        let sheet = sheet.clone();
        let ui = self.ui_runner;
        std::thread::spawn(move || {
            if let Err(error) = battle::save_sheet_blocking(&sheet) {
                ui.run(move |s: &mut Self, cx| {
                    s.failure_ref().set_message(&error.to_string());
                    s.show_frame(s.failure_ref().widget_uid());
                    s.redraw(cx);
                });

                return;
            }

            let sheet_clone = sheet.clone();
            ui.run(move |s: &mut Self, cx| {
                s.set_sheet(Some(sheet_clone));
                s.redraw(cx);
            });

            if sheet.is_completed() {
                ui.run(|s: &mut Self, cx| {
                    s.show_frame(s.loading_ref().widget_uid());
                    s.redraw(cx);
                });

                let result = battle::send_sheet_blocking(sheet);

                ui.run(move |s: &mut Self, cx| {
                    if let Err(error) = result {
                        s.failure_ref().set_message(&error.to_string());
                        s.show_frame(s.failure_ref().widget_uid());
                        s.redraw(cx);
                        return;
                    }

                    s.show_frame(s.ending_ref().widget_uid());
                    s.redraw(cx);
                });
            }
        });
    }

    fn handle_retry(&mut self, cx: &mut Cx) {
        self.show_frame(self.opening_ref().widget_uid());
        self.redraw(cx);
    }

    fn handle_end(&mut self) {
        let ui = self.ui_runner;
        std::thread::spawn(move || {
            let result = battle::clear_sheet_blocking();
            ui.run(move |s: &mut Self, cx| {
                if let Err(error) = result {
                    s.failure_ref().set_message(&error.to_string());
                    s.show_frame(s.failure_ref().widget_uid());
                    s.redraw(cx);
                    return;
                }

                s.set_sheet(None);
                s.show_frame(s.opening_ref().widget_uid());
                s.redraw(cx);
            });
        });
    }
}

// other stuff
impl BattleScreen {
    fn set_sheet(&mut self, sheet: Option<battle::Sheet>) {
        self.sheet = sheet;

        if let Some(sheet) = self.sheet.as_ref() {
            if let Some(round) = sheet.current_round() {
                self.left_messages_ref()
                    .set_messages(round.chats[0].messages.clone());
                self.right_messages_ref()
                    .set_messages(round.chats[1].messages.clone());

                let rounds_count = sheet.rounds.len();
                let current_round_index = sheet.current_round_index().unwrap();
                self.counter_ref().set_text(&format!(
                    "{}/{}",
                    current_round_index + 1,
                    rounds_count
                ));
            }
        }
    }

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
