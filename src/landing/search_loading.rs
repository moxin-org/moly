use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::landing::model_card::ModelCard;

    ANIMATION_SPEED = 0.33

    LoadingBall = <CircleView> {
        width: 28
        height: 28
        draw_bg: {
            border_radius: 14.0
            fn get_color(self) -> vec4 {
                let top_color = #E6F2D8;
                let bottom_color = #A4E0EF;
                let gradient_ratio = self.pos.y;
                return mix(top_color, bottom_color, gradient_ratio);
            }
        }
    }

    pub SearchLoading = {{SearchLoading}} {
        width: Fill,
        height: Fill,

        flow: Down,
        spacing: 60,
        align: {x: 0.5, y: 0.5},

        content = <View> {
            width: Fit,
            height: Fit,
            spacing: 30,
            circle1 = <LoadingBall> {}
            circle2 = <LoadingBall> {}
            circle3 = <LoadingBall> {}
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 14},
                color: #667085
            }
            text: "Searching..."
        }

        animator: {
            circle1 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {content = { circle1 = { draw_bg: {border_radius: 1.0} }}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {content = { circle1 = { draw_bg: {border_radius: 14.0} }}}
                }
            }

            circle2 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {content = { circle2 = { draw_bg: {border_radius: 1.0} }}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {content = { circle2 = { draw_bg: {border_radius: 14.0} }}}
                }
            }

            circle3 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {content = { circle3 = { draw_bg: {border_radius: 1.0} }}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {content = { circle3 = { draw_bg: {border_radius: 14.0} }}}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SearchLoading {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    timer: Timer,

    #[rust]
    current_animated_circle: usize,
}

impl Widget for SearchLoading {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.update_animation(cx);
        }
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl SearchLoading {
    pub fn update_animation(&mut self, cx: &mut Cx) {
        self.current_animated_circle = (self.current_animated_circle + 1) % 3;

        match self.current_animated_circle {
            0 => {
                self.animator_play(cx, id!(circle1.run));
                self.animator_play(cx, id!(circle3.start));
            }
            1 => {
                self.animator_play(cx, id!(circle1.start));
                self.animator_play(cx, id!(circle2.run));
            }
            2 => {
                self.animator_play(cx, id!(circle2.start));
                self.animator_play(cx, id!(circle3.run));
            }
            _ => unreachable!(),
        };

        self.timer = cx.start_timeout(0.33);
    }
}

impl SearchLoadingRef {
    pub fn animate(&mut self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.update_animation(cx);
    }

    pub fn stop_animation(&mut self) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.timer = Timer::default();
    }
}
