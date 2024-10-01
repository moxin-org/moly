use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    Opening = {{Opening}} {
        flow: Down,
        align: {x: 0.5, y: 0.5},
        spacing: (SM_GAP),
        <Icon> {
            margin: {bottom: (MD_GAP)},
            draw_icon: {
                svg_file: dep("crate://self/resources/icons/my_models.svg"),
                fn get_color(self) -> vec4 {
                    return #6b7475;
                }
            }
            icon_walk: {width: 250, height: 250}
        }
        <Label> {
            draw_text: {
                color: #000,
                text_style: <BOLD_FONT> { font_size: 14 }
            }
            text: "Welcome to the battle!"
        }
        input = <MolyTextInput> {
            empty_message: "Enter your code...",
        }
        button = <MolyButton> {
            text: "Start",
            draw_bg: {
                color: #000,
            },
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
