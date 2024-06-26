use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::FadeView;
    import crate::shared::widgets::MoxinButton;
    import makepad_draw::shader::std::*;


    // TODO: Use proper icons
    ICON_CLOSE_PANEL = dep("crate://self/resources/icons/open_left_panel.svg")
    ICON_OPEN_PANEL = dep("crate://self/resources/icons/close_left_panel.svg")

    MoxinSlider =  <Slider> {
        height: 40
        width: Fill
        draw_text: {
            // TODO: The text weight should be 500 (semi bold, not fully bold).
            text_style: <BOLD_FONT>{font_size: 10},
            color: #000
        }
        text_input: {
            /*draw_bg: {
                color: #000;
                radius: 2.0
            },*/
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 11},
                fn get_color(self) -> vec4 {
                    return #000;
                }
            }
        }
        draw_slider: {
            instance bipolar: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)

                let ball_radius = 10.0;
                let ball_border = 2.0;
                let padding_top = 29.0;
                let padding_x = 5.0;
                let rail_height = 4.0;

                let rail_width = self.rect_size.x;
                let rail_padding_x = padding_x + ball_radius / 2;
                let ball_rel_x = self.slide_pos;
                let ball_abs_x = ball_rel_x * (rail_width - 2.0 * rail_padding_x) + rail_padding_x;

                // The drawing area (for debug only)
                // sdf.rect(0, 0, self.rect_size.x, self.rect_size.y);
                // sdf.fill(#06b6d4);

                // The rail
                sdf.move_to(0 + padding_x, padding_top);
                sdf.line_to(self.rect_size.x - padding_x, padding_top);
                sdf.stroke(#D9D9D9, rail_height);

                // The filler
                sdf.move_to(0 + padding_x, padding_top);
                sdf.line_to(ball_abs_x, padding_top);
                sdf.stroke(#989898, rail_height);

                // The moving ball
                sdf.circle(ball_abs_x, padding_top, ball_radius);
                sdf.fill(#989898);
                sdf.circle(ball_abs_x, padding_top, ball_radius - ball_border);
                sdf.fill(#fff);


                return sdf.result;
            }
        }
    }

    ChatParamsActions = <View> {
        height: Fit
        flow: Right

        <View> {
            width: Fill
            height: Fit
        }


        close_panel_button = <MoxinButton> {
            width: Fit,
            height: Fit,
            icon_walk: {width: 20, height: 20},
            draw_icon: {
                svg_file: (ICON_CLOSE_PANEL),
                fn get_color(self) -> vec4 {
                    return #475467;
                }
            }
        }

        open_panel_button = <MoxinButton> {
            width: Fit,
            height: Fit,
            visible: false,
            icon_walk: {width: 20, height: 20},
            draw_icon: {
                svg_file: (ICON_OPEN_PANEL),
                fn get_color(self) -> vec4 {
                    return #475467;
                }
            }
        }
    }

    ChatParams = {{ChatParams}} {
        flow: Overlay,
        width: Fit,
        height: Fill,

        main_content = <FadeView> {
            width: 300
            height: Fill
            <View> {
                width: Fill
                height: Fill
                padding: {top: 70, left: 25.0, right: 25.0}
                spacing: 35
                flow: Down
                show_bg: true
                draw_bg: {
                    color: #F2F4F7
                }

                <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 12}
                        color: #000
                    }
                    text: "Inference Parameters"
                }

                <View> {
                    flow: Down
                    spacing: 24

                    <MoxinSlider> {
                        text: "Temperature"
                        min: 0.0
                        max: 2.0
                    }

                    <MoxinSlider> {
                        text: "Top P"
                        min: 0.0
                        max: 1.0
                    }

                    <MoxinSlider> {
                        text: "Max Tokens"
                        min: 100.0
                        max: 2048.0
                        step: 1.0
                    }

                    <MoxinSlider> {
                        text: "Frequency Penalty"
                        min: 0.0
                        max: 1.0
                    }

                    <MoxinSlider> {
                        text: "Presence Penalty"
                        min: 0.0
                        max: 1.0
                    }
                }
            }
        }

        <ChatParamsActions> {
            padding: {top: 58, left: 25, right: 25}
        }

        animator: {
            panel = {
                default: show,
                show = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {main_content = { width: 300, draw_bg: {opacity: 1.0} }}
                }
                hide = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {main_content = { width: 110, draw_bg: {opacity: 0.0} }}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatParams {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,
}

impl Widget for ChatParams {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatParams {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.button(id!(close_panel_button)).clicked(&actions) {
            self.button(id!(close_panel_button)).set_visible(false);
            self.button(id!(open_panel_button)).set_visible(true);
            self.animator_play(cx, id!(panel.hide));
        }

        if self.button(id!(open_panel_button)).clicked(&actions) {
            self.button(id!(open_panel_button)).set_visible(false);
            self.button(id!(close_panel_button)).set_visible(true);
            self.animator_play(cx, id!(panel.show));
        }
    }
}
