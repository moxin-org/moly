use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    Opening = {{Opening}} {
        align: {x: 0.5, y: 0.5},
        spacing: (LG_GAP * 3),
        <Image> {
            margin: {bottom: (MD_GAP)},
            source: dep("crate://self/resources/icons/prerendered/output/battle.png"),
            width: 338,
            height: 280,
        }
        <View> {
            flow: Down,
            width: 300,
            height: Fit,
            <Label> {
                draw_text: {
                    color: #000,
                    text_style: <BOLD_FONT> { font_size: 18 }
                }
                text: "Agents Arena"
            }
            <Label> {
                margin: {top: 3.0}
                width: Fill,
                draw_text: {
                    color: #000,
                    text_style: <REGULAR_FONT> { font_size: 11 }
                }
                text: "To join the game, please enter the code that was provided to you."
            }
            <RoundedView> {
                margin: {top: 14.0},
                width: Fill,
                height: Fit,
                padding: 4.0,
                draw_bg: {
                    radius: 4.0,
                    border_color: #D0D5DD,
                    border_width: 1.0,
                }
                input = <MolyTextInput> {
                    draw_bg: {
                        fn pixel(self) -> vec4 {
                            return vec4(0.0, 0.0, 0.0, 0.0);
                        }
                    }
                    width: Fill,
                    empty_message: "Your code",
                }
            }
            button = <MolyButton> {
                margin: {top: 14.0},
                width: Fill,
                padding: {top: (SM_GAP), bottom: (SM_GAP)},
                text: "Start",
                draw_bg: { color: #099250 }
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Opening {
    #[deref]
    view: View,
}

impl Widget for Opening {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Opening {
    fn input_ref(&self) -> TextInputRef {
        self.text_input(id!(input))
    }

    fn button_ref(&self) -> ButtonRef {
        self.button(id!(button))
    }

    pub fn submitted(&self, actions: &Actions) -> bool {
        self.input_ref().returned(actions).is_some() || self.button_ref().clicked(actions)
    }

    pub fn code(&self) -> String {
        self.input_ref().text()
    }

    pub fn clear(&self) {
        self.input_ref().set_text("");
    }
}

impl OpeningRef {
    pub fn submitted(&self, actions: &Actions) -> bool {
        self.borrow().map(|s| s.submitted(actions)).unwrap_or(false)
    }

    pub fn code(&self) -> String {
        self.borrow().map(|s| s.code()).unwrap_or_default()
    }

    pub fn clear(&self) {
        self.borrow().map(|s| s.clear());
    }
}
