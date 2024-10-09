use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    import makepad_draw::shader::std::*;

    SM_SIZE = 32;
    MD_SIZE = 44;
    LG_SIZE = 60;

    VoteButton = <Button> {
        draw_bg: {
            fn pixel(self) -> vec4 {
                let fill_color = #dae8ec
                let stroke_color = mix(#15859A, vec4(0.0, 0.0, 0.0, 0.0), self.pos.x * 0.5);

                let center = vec2(0.5, 0.5);
                let distance_from_center = distance(self.pos, center);

                if distance_from_center > 0.5 {
                    return vec4(0.0, 0.0, 0.0, 0.0);
                }

                if distance_from_center > 0.475 {
                    return stroke_color;
                }

                return fill_color;
            }
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
                fn pixel(self) -> vec4{
                    let distance_from_center = abs(self.pos.x - 0.5);
                    let dist = distance_from_center * 1.5;
                    let color = mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(#15859A.xyz, 0.8), 1.0 - dist);
                    return mix(color, vec4(0.0, 0.0, 0.0, 0.0), dist);
                }
            }
        }
        <View> {
            height: Fit,
            width: Fit,
            align: {x: 0.5, y: 0.5},
            <EdgeLabel> { text: "Left is better" }
            a2 = <VoteButton> {
                margin: {left: 30},
                height: (LG_SIZE),
                width: (LG_SIZE),
            }
            a1 = <VoteButton> {
                margin: {left: 120},
                height: (MD_SIZE),
                width: (MD_SIZE),
            }
            o0 = <VoteButton> {
                margin: {left: 60, right: 60},
                height: (SM_SIZE),
                width: (SM_SIZE),
            }
            b1 = <VoteButton> {
                margin: {right: 120},
                height: (MD_SIZE),
                width: (MD_SIZE),
            }
            b2 = <VoteButton> {
                margin: {right: 30},
                height: (LG_SIZE),
                width: (LG_SIZE),
            }
            <EdgeLabel> { text: "Right is better" }
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
    move_tooltip_over: Option<ButtonRef>,
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

        if let Some(button) = self.move_tooltip_over.take() {
            let tooltip_content_rect = tooltip.view(id!(content)).area().rect(cx);
            let btn_rect = button.area().rect(cx);
            let y = btn_rect.pos.y - tooltip_content_rect.size.y - 5.0;
            let x = btn_rect.pos.x + btn_rect.size.x / 2.0 - tooltip_content_rect.size.x / 2.0;
            tooltip.set_pos(cx, DVec2 { x, y });

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
                tooltip.set_text(message);
                tooltip.show(cx);
                self.move_tooltip_over = Some(button);
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
