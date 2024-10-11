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


    StripButton = <Button> {
        draw_bg: {
            // relative radius, 0.0 is rect, 1.0 is full rounded
            instance left_radius: 0.0;
            instance right_radius: 0.0;
            instance step: 1.0;
            instance step_influence: 0.1;

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                // idk why 0.5 must be the minimum nor idk why 0.25 and not 0.5 for the multiplier
                let rl = mix(0.5, self.rect_size.y * 0.25, self.left_radius);
                let rr = mix(0.5, self.rect_size.y * 0.25, self.right_radius);

                // base color
                let fill_color = #15859A;

                // make the base color ligther as step grows
                let fill_color = mix(fill_color, #fff, self.step * self.step_influence);

                // make a gradiant over the x axis
                // use the step_infuence so this gradiant can be continued by the next button
                let fill_color = mix(fill_color, #fff, self.pos.x * self.step_influence);

                // make the color a little bit ligther when hovered
                let fill_color = mix(fill_color, #fff, self.hover * 0.3);

                sdf.box_all(0.0, 0.0, self.rect_size.x, self.rect_size.y, rl, rr, rr, rl);

                sdf.fill_keep(fill_color);

                return sdf.result;
            }
        },
    }

    SizedStripButton = <StripButton> {
        width: 100.0,
        height: 32.0,
    }

    EdgeLabel = <Label> {
        draw_text: {
            text_style: <BOLD_FONT>{height_factor: 1.3, font_size: 12},
            color: #000,
        }
    }

    Vote = {{Vote}} <View> {
        flow: Overlay,
        height: Fit,
        align: {x: 0.5, y: 0.5},
        <View> {
            height: Fit,
            width: Fit,
            align: {x: 0.5, y: 0.5},
            spacing: 2,

            <EdgeLabel> { text: "Left is better", margin: {right: 4} }
            a2 = <SizedStripButton> {
                draw_bg: {
                    instance step: 1.0;
                    left_radius: 1.0;
                }
            }
            a1 = <SizedStripButton> {
                draw_bg: {
                    instance step: 2.0;
                }
            }
            o0 = <SizedStripButton> {
                draw_bg: {
                    instance step: 3.0;
                }
            }
            b1 = <SizedStripButton> {
                draw_bg: {
                    instance step: 4.0;
                }
            }
            b2 = <SizedStripButton> {
                draw_bg: {
                    instance step: 5.0;
                    right_radius: 1.0;
                }
            }
            <EdgeLabel> { text: "Right is better", margin: {left: 4} }
        }
        tooltip = <Tooltip> {
            content: <View> {
                width: Fit,
                height: Fit,
                content = <View> {
                    // Content width seems to flicker when set to Fit, making calculations
                    // fail even for the second pass.
                    width: 200,
                    height: Fit,
                    align: {x: 0.5, y: 0.5},
                    <RoundedView> {
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
}

#[derive(Live, Widget)]
pub struct Vote {
    #[deref]
    view: View,
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

        if let Event::MouseMove(event) = event {
            let buttons_with_messages = [
                (id!(a2), "Left is much better"),
                (id!(a1), "Left is slightly better"),
                (id!(o0), "Tie"),
                (id!(b1), "Right is slightly better"),
                (id!(b2), "Right is much better"),
            ];

            let pointer_pos = event.abs;

            let hovered_button = buttons_with_messages
                .iter()
                .find_map(|(button_id, message)| {
                    let button = self.button(*button_id);
                    if button.area().rect(cx).contains(pointer_pos) {
                        Some((button, message))
                    } else {
                        None
                    }
                });

            if let Some((button, message)) = hovered_button {
                let tooltip_content_rect = tooltip.view(id!(content)).area().rect(cx);
                let btn_rect = button.area().rect(cx);
                let y = btn_rect.pos.y - tooltip_content_rect.size.y - 7.5;
                let x = btn_rect.pos.x + btn_rect.size.x / 2.0 - tooltip_content_rect.size.x / 2.0;
                tooltip.set_pos(cx, DVec2 { x, y });
                tooltip.set_text(message);
                self.redraw(cx);
            } else {
                tooltip.set_pos(
                    cx,
                    DVec2 {
                        x: 10000.,
                        y: 10000.,
                    },
                );
                self.redraw(cx);
            }
        };
    }
}

impl VoteRef {
    pub fn voted(&self, actions: &Actions) -> Option<i8> {
        self.borrow().map(|inner| inner.voted(actions)).flatten()
    }
}

impl LiveHook for Vote {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.tooltip(id!(tooltip)).show_with_options(
            cx,
            DVec2 {
                x: 10000.,
                y: 10000.,
            },
            // We need to init it with a non empty string to avoid flickering,
            // on the first mouse move event hover, caused by the changing height.
            "a",
        );
    }
}
