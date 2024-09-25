use super::{
    battle_service::BattleService,
    battle_sheet::{Round, Sheet},
    messages::{MessagesRef, MessagesWidgetExt},
    start::{StartRef, StartWidgetExt},
    vote::VoteWidgetExt,
};
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::messages::Messages;
    import crate::battle::vote::Vote;
    import crate::battle::start::Start;
    import crate::battle::spinner::Spinner;
    import crate::battle::styles::*;

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
        start = <Start> {}
        end = <View> {
            visible: false,
            align: {x: 0.5, y: 0.5},
            <Label> {
                draw_text: {
                    color: #000,
                    text_style: <BOLD_FONT> { font_size: 14 }
                }
                text: "You're done! Thank you for participating. ✅"
            }
        }
        loading = <View> {
            visible: false,
            align: {x: 0.5, y: 0.5},
            <Spinner> {}
        }
        compare = <View> {
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
    }
}

#[derive(Live, Widget, LiveHook)]
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

        if self.start_frame().submitted(actions) {
            self.service
                .download_battle_sheet(self.start_frame().code());
            self.start_frame().borrow_mut().unwrap().visible = false;
            self.loading_frame().set_visible(true);
            redraw = true;
        }

        if let Some(weight) = self.vote(id!(vote)).voted(actions) {
            if let Some(round) = self.current_round_mut() {
                round.weight = weight.into();
                self.update_round();
                redraw = true;
                scroll_to_bottom = true;
            }
        }

        if let Some(error) = self.service.failed(actions) {
            eprintln!("{}", error);
        }

        if let Some(sheet) = self.service.battle_sheet_downloaded(actions) {
            self.sheet = sheet.clone().into();
            self.loading_frame().set_visible(false);
            self.round_frame().set_visible(true);
            self.update_round();

            redraw = true;
            scroll_to_bottom = true;
        }

        if scroll_to_bottom {
            self.left_messages().scroll_to_bottom(cx);
            self.right_messages().scroll_to_bottom(cx);
        }

        if redraw {
            self.redraw(cx);
        }
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

    fn left_messages(&self) -> MessagesRef {
        self.messages(id!(left.messages))
    }

    fn right_messages(&self) -> MessagesRef {
        self.messages(id!(right.messages))
    }

    fn start_frame(&self) -> StartRef {
        self.start(id!(start))
    }

    fn round_frame(&self) -> ViewRef {
        self.view(id!(compare))
    }

    fn loading_frame(&self) -> ViewRef {
        self.view(id!(loading))
    }

    fn counter(&self) -> LabelRef {
        self.label(id!(counter))
    }

    fn update_round(&self) {
        if let Some(index) = self.current_round_index() {
            let round = self.current_round().unwrap();

            self.left_messages()
                .set_messages(round.chats[0].messages.clone());

            self.right_messages()
                .set_messages(round.chats[1].messages.clone());

            let rounds_count = self.sheet.as_ref().unwrap().rounds.len();
            self.counter()
                .set_text(&format!("{} / {}", index + 1, rounds_count));
        }
    }
}
