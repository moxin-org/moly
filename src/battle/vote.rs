use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    SM_SIZE = 32;
    MD_SIZE = 44;
    LG_SIZE = 60;

    SM_RADIUS = 7;
    MD_RADIUS = 10;
    LG_RADIUS = 15;

    VoteButton = <MolyButton> {
        draw_bg: {
            border_color: #15859A,
            color: #dae8ec,
        },
    }

    EdgeLabel = <Label> {
        draw_text: {
            text_style: <BOLD_FONT>{height_factor: 1.3, font_size: 14},
            color: #000,
        }
    }

    Vote = {{Vote}} <View> {
        flow: Overlay,
        height: Fit,
        align: {x: 0.5, y: 0.5},
        <View> {
            height: 1.5,
            width: 500,
            show_bg: true,
            draw_bg: {
                color: #15859A,
            }
        }
        <View> {
            height: Fit,
            width: Fit,
            align: {x: 0.5, y: 0.5},
            <EdgeLabel> { text: "A better" }
            a2 = <VoteButton> {
                margin: {left: 30},
                height: (LG_SIZE),
                width: (LG_SIZE),
                draw_bg: {
                    radius: (LG_RADIUS),
                },
            }
            a1 = <VoteButton> {
                margin: {left: 120},
                height: (MD_SIZE),
                width: (MD_SIZE),
                draw_bg: {
                    radius: (MD_RADIUS),
                }
            }
            o0 = <VoteButton> {
                margin: {left: 60, right: 60},
                height: (SM_SIZE),
                width: (SM_SIZE),
                draw_bg: {
                    radius: (SM_RADIUS),
                }
            }
            b1 = <VoteButton> {
                margin: {right: 120},
                height: (MD_SIZE),
                width: (MD_SIZE),
                draw_bg: {
                    radius: (MD_RADIUS),
                }
            }
            b2 = <VoteButton> {
                margin: {right: 30},
                height: (LG_SIZE),
                width: (LG_SIZE),
                draw_bg: {
                    radius: (LG_RADIUS),
                }
            }
            <EdgeLabel> { text: "B better" }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Vote {
    #[deref]
    view: View,
}

impl Widget for Vote {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Vote {
    pub fn voted(&self, actions: &Actions) -> Option<i8> {
        if self.button(id!(a2)).clicked(actions) {
            return Some(-2);
        }

        if self.button(id!(a1)).clicked(actions) {
            return Some(-1);
        }

        if self.button(id!(o0)).clicked(actions) {
            return Some(0);
        }

        if self.button(id!(b1)).clicked(actions) {
            return Some(1);
        }

        if self.button(id!(b2)).clicked(actions) {
            return Some(2);
        }

        None
    }
}

impl VoteRef {
    pub fn voted(&self, actions: &Actions) -> Option<i8> {
        self.borrow().map(|inner| inner.voted(actions)).flatten()
    }
}
