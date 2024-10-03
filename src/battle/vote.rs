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
        tooltip = <Tooltip> {
            content: <RoundedView> {
                width: Fit,
                height: Fit,
                content = <RoundedView> {
                    width: Fit,
                    height: Fit,
                    padding: {left: 10, right: 10, top: 5, bottom: 5},
                    draw_bg: {
                        color: #15859A,
                        radius: 5.0,
                    },
                    tooltip_label = <Label> {
                        draw_text: {
                            text_style: <REGULAR_FONT>{height_factor: 1.3, font_size: 12},
                            color: #fff,
                        }
                    }
                }
            }


        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Vote {
    #[deref]
    view: View,

    #[rust]
    tooltip_position_pending: bool,
}

impl Widget for Vote {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.handle_tooltip(cx, event);
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

    fn handle_tooltip(&mut self, cx: &mut Cx, event: &Event) {
        let mut tooltip = self.tooltip(id!(tooltip));

        if self.tooltip_position_pending {
            self.tooltip_position_pending = false;

            return;
        }

        if let Event::MouseMove(event) = event {
            let buttons_ids = [id!(a2), id!(a1), id!(o0), id!(b1), id!(b2)];
            let tooltip_messages = [
                "A is much better",
                "A is slightly better",
                "Tie",
                "B is slightly better",
                "B is much better",
            ];

            let pointer_pos = event.abs;

            let hovered_button =
                buttons_ids
                    .iter()
                    .zip(tooltip_messages.iter())
                    .find_map(|(button_id, message)| {
                        let button = self.button(*button_id);
                        if button.area().rect(cx).contains(pointer_pos) {
                            Some((button, message))
                        } else {
                            None
                        }
                    });

            if let Some((button, message)) = hovered_button {
                // let tooltip_rect = println!("hovered");
                let rect = button.area().rect(cx);
                let y = rect.pos.y - 5.0;
                let x = rect.pos.x - rect.size.x / 2.0;
                tooltip.show_with_options(cx, DVec2 { x, y }, message);
            } else {
                tooltip.hide(cx);
            }
        };
    }
}

impl VoteRef {
    pub fn voted(&self, actions: &Actions) -> Option<i8> {
        self.borrow().map(|inner| inner.voted(actions)).flatten()
    }
}
