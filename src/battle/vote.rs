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
        width: Fill,
        height: Fill,
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 0.5);
                sdf.fill(vec4(#fff.xyz, self.hover * 0.4));
                return sdf.result;
            }
        }
    }

    Split = <View> { width: 3, show_bg: true, draw_bg: {color: #fff}}

    Strip = <View> {
        width: (100.0 * 5),
        height: 32.0,
        show_bg: true,
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                // relative radius, 1.0 means fully rounded
                // don't know why 0.5 is the min or why the multiplier is 0.25 instead of 0.5
                let r = mix(0.5, self.rect_size.y * 0.25, 1.0);

                let edge_color = #15859A;
                let middle_color = mix(edge_color, #fff, 0.5);

                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, r);

                // oszilate the color from edge_color to middle_color
                let blend_factor = abs(mix(-1.0, 1.0, self.pos.x));
                let fill_color = mix(middle_color, edge_color, blend_factor);

                sdf.fill(fill_color);
                return sdf.result;
            }

        }
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
            <Strip> {
                a2 = <StripButton> {}
                <Split> {}
                a1 = <StripButton> {}
                <Split> {}
                o0 = <StripButton> {}
                <Split> {}
                b1 = <StripButton> {}
                <Split> {}
                b2 = <StripButton> {}
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
