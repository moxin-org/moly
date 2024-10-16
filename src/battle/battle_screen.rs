use crate::data::{
    battle::{self, client::Client},
    store::ScopeStoreExt,
};

use super::{
    ending::EndingWidgetExt, failure::FailureWidgetExt, messages::MessagesWidgetExt,
    opening::OpeningWidgetExt, spinner::SpinnerWidgetExt, ui_runner::UiRunner, vote::VoteWidgetExt,
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
        messages = <Messages> {}
    }

    Frame = <View> {
        visible: false,
    }

    BattleScreen = {{BattleScreen}} {
        flow: Overlay,
        opening_frame = <Frame> {
            opening = <Opening> {}
        }
        ending_frame = <Frame> {
            ending = <Ending> {}
        }
        round_frame = <Frame> {
            round = <View> {
                flow: Down,
                align: {x: 0.5},
                padding: {top: 40, bottom: 40, left: (LG_GAP), right: (LG_GAP)},
                spacing: (MD_GAP),
                counter = <Label> {
                    draw_text: {
                        color: #000,
                        text_style: <BOLD_FONT> { font_size: 14 }
                    }
                }
                <View> {
                    spacing: (MD_GAP),
                    left = <Half> {}
                    <View> {
                        width: 1.5,
                        height: Fill,
                        show_bg: true,
                        draw_bg: {
                            fn pixel(self) -> vec4{
                                let distance_from_center = abs(self.pos.y - 0.5);
                                let dist = distance_from_center * 1.30;
                                let color = mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(#000.xyz, 0.15), 1.0 - dist);
                                return mix(color, vec4(0.0, 0.0, 0.0, 0.0), dist);
                            }
                        }
                    }
                    right = <Half> {}
                }
                vote = <Vote> {}
            }
        }
        loading_frame = <Frame> {
            visible: true,
            loading = <View> {
                align: {x: 0.5, y: 0.5},
                spinner = <Spinner> {}
            }
        }
        failure_frame = <Frame> {
            failure = <Failure> {}
        }
        blocker_overlay_frame = <Frame> {
            // Workaround: A button captures mouse events so it works well to block
            // the screen in (probably) fast operations where a spinner would flicker.
            // For some reason I can't achieve the same effect with a view using only the DSL.
            <MolyButton> {
                width: Fill,
                height: Fill,
                draw_bg: { color: #00000000, border_color: #00000000, color_hover: #00000000, border_color_hover: #00000000 }
            }
        }
    }
}

#[derive(Live, Widget)]
pub struct BattleScreen {
    #[deref]
    view: View,

    #[rust]
    ui_runner: UiRunner,

    #[rust]
    sheet: Option<battle::Sheet>,

    // `after_new_from_doc` doesn't have access to `scope`.
    #[rust]
    initialized: bool,
}

impl LiveHook for BattleScreen {}

impl Widget for BattleScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.initialized {
            self.initialized = true;
            self.handle_init(cx, scope);
        }

        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
        self.ui_runner.handle(cx, event, self);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for BattleScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.opening(id!(opening)).submitted(actions) {
            self.handle_opening_submit(cx, scope);
        }

        if let Some(weight) = self.vote(id!(vote)).voted(actions) {
            let round_index = self.sheet.as_ref().unwrap().current_round_index().unwrap();
            self.handle_vote(cx, scope, round_index, weight);
        }

        if self.ending(id!(ending)).ended(actions) {
            self.handle_end(cx, scope);
        }
    }
}

// event handlers
impl BattleScreen {
    fn handle_init(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let ui = self.ui_runner;
        let mut client = battle::AutoClient::new(scope.preferences().battle_url.clone());
        std::thread::spawn(move || {
            let sheet = client.restore_sheet_blocking();
            ui.defer_with_redraw(move |s: &mut Self, _cx| match sheet {
                Ok(sheet) => {
                    if sheet.is_completed() {
                        s.show_ending_frame();
                    } else {
                        s.show_round_frame(sheet);
                    }
                }
                Err(_) => {
                    s.show_opening_frame();
                }
            });
        });
    }

    fn handle_opening_submit(&mut self, cx: &mut Cx, scope: &mut Scope) {
        self.show_loading_frame("Downloading sheet...");
        self.redraw(cx);

        let code = self.opening(id!(opening)).code();
        let ui = self.ui_runner;
        let mut client = battle::AutoClient::new(scope.preferences().battle_url.clone());
        std::thread::spawn(move || {
            match client.download_sheet_blocking(code) {
                Ok(sheet) => {
                    if let Err(error) = client.save_sheet_blocking(&sheet) {
                        ui.defer_with_redraw(move |s: &mut Self, _cx| {
                            s.show_failure_frame(&error.to_string());
                        });

                        return;
                    }

                    ui.defer_with_redraw(move |s: &mut Self, _cx| {
                        // Server should not return an already completed sheet,
                        // but just in case...
                        if sheet.is_completed() {
                            s.show_ending_frame();
                        } else {
                            s.show_round_frame(sheet);
                        }
                    });
                }
                Err(error) => {
                    ui.defer_with_redraw(move |s: &mut Self, _cx| {
                        s.show_failure_frame(&error.to_string());
                    });
                }
            }
        });
    }

    fn handle_vote(&mut self, cx: &mut Cx, scope: &mut Scope, round_index: usize, weight: i8) {
        self.show_blocker_overlay_frame();
        self.redraw(cx);

        let mut sheet = self.sheet.as_ref().unwrap().clone();
        let mut client = battle::AutoClient::new(scope.preferences().battle_url.clone());
        let ui = self.ui_runner;
        std::thread::spawn(move || {
            sheet.rounds[round_index].vote = Some(weight);
            if let Err(error) = client.save_sheet_blocking(&sheet) {
                ui.defer_with_redraw(move |s: &mut Self, _cx| {
                    s.show_failure_frame(&error.to_string());
                });

                return;
            }

            if sheet.is_completed() {
                ui.defer_with_redraw(|s: &mut Self, _cx| {
                    s.show_loading_frame("Sending sheet...");
                });

                if let Err(error) = client.send_sheet_blocking(sheet) {
                    ui.defer_with_redraw(move |s: &mut Self, _cx| {
                        s.show_failure_frame(&error.to_string());
                    });

                    return;
                }

                ui.defer_with_redraw(|s: &mut Self, _cx| {
                    s.show_ending_frame();
                });

                return;
            }

            ui.defer_with_redraw(|s: &mut Self, _cx| {
                s.show_round_frame(sheet);
            });
        });
    }

    fn handle_end(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let ui = self.ui_runner;
        let mut client = battle::AutoClient::new(scope.preferences().battle_url.clone());
        std::thread::spawn(move || {
            let result = client.clear_sheet_blocking();
            ui.defer_with_redraw(move |s: &mut Self, _cx| {
                if let Err(error) = result {
                    s.show_failure_frame(&error.to_string());
                    return;
                }

                s.set_sheet(None);
                s.show_opening_frame();
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
                self.messages(id!(left.messages))
                    .set_messages(round.chats[0].messages.clone());
                self.messages(id!(right.messages))
                    .set_messages(round.chats[1].messages.clone());

                let rounds_count = sheet.rounds.len();
                let current_round_index = sheet.current_round_index().unwrap();
                self.label(id!(counter)).set_text(&format!(
                    "{}/{}",
                    current_round_index + 1,
                    rounds_count
                ));
            }
        }
    }

    fn show_loading_frame(&mut self, message: &str) {
        self.spinner(id!(spinner)).set_message(message);
        self.hide_all_frames();
        self.view(id!(loading_frame)).set_visible(true);
    }

    fn show_round_frame(&mut self, sheet: battle::Sheet) {
        let ui = self.ui_runner;
        let sheet_backup = sheet.clone();
        self.failure(id!(failure)).set_recovery_cb(move || {
            ui.defer_with_redraw(|s: &mut Self, _cx| {
                s.show_round_frame(sheet_backup);
            });
        });

        self.set_sheet(Some(sheet));
        self.hide_all_frames();
        self.view(id!(round_frame)).set_visible(true);
    }

    fn show_failure_frame(&mut self, message: &str) {
        self.hide_all_frames();
        self.failure(id!(failure)).set_message(message);
        self.view(id!(failure_frame)).set_visible(true);
    }

    fn show_opening_frame(&mut self) {
        let ui = self.ui_runner;
        self.failure(id!(failure)).set_recovery_cb(move || {
            ui.defer_with_redraw(|s: &mut Self, _cx| {
                s.show_opening_frame();
            });
        });

        self.opening(id!(opening)).clear();
        self.hide_all_frames();
        self.view(id!(opening_frame)).set_visible(true);
    }

    fn show_ending_frame(&mut self) {
        let ui = self.ui_runner;
        self.failure(id!(failure)).set_recovery_cb(move || {
            ui.defer_with_redraw(|s: &mut Self, _cx| {
                s.show_ending_frame();
            });
        });

        self.hide_all_frames();
        self.view(id!(ending_frame)).set_visible(true);
    }

    fn show_blocker_overlay_frame(&mut self) {
        // This is an overlay to block user interactions so it doesn't hide other frames.
        self.view(id!(blocker_frame)).set_visible(true);
    }

    fn hide_all_frames(&mut self) {
        [
            id!(opening_frame),
            id!(ending_frame),
            id!(loading_frame),
            id!(round_frame),
            id!(failure_frame),
            id!(blocker_frame),
        ]
        .iter()
        .for_each(|id| {
            self.view(*id).set_visible(false);
        });
    }
}
