use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::landing::model_card::ModelCard;

    ANIMATION_SPEED = 0.33

    Bar = <View> {
        width: Fill,
        height: 16,
        show_bg: true,
        draw_bg: {
            instance dither: 0.3

            fn get_color(self) -> vec4 {
                return mix(
                    #F3FFA2,
                    #E3FBFF,
                    self.pos.x + self.dither
                )
            }

            fn pixel(self) -> vec4 {
                return Pal::premul(self.get_color())
            }
        }
    }

    pub ChatLineLoading = {{ChatLineLoading}} {
        width: Fill,
        height: Fit,

        flow: Down,
        spacing: 4,

        line1 = <Bar> {}
        line2 = <Bar> {}
        <View> {
            width: Fill,
            height: 16,
            line3 = <Bar> {}
            <VerticalFiller> {}
        }

        animator: {
            line1 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line1 = { draw_bg: {dither: 0.1} }}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line1 = { draw_bg: {dither: 0.9} }}
                }
            }

            line2 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line2 = { draw_bg: {dither: 0.1} }}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line2 = { draw_bg: {dither: 0.9} }}
                }
            }

            line3 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line3 = { draw_bg: {dither: 0.1} }}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line3 = { draw_bg: {dither: 0.9} }}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatLineLoading {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    timer: Timer,

    #[rust]
    current_animated_bar: usize,
}

impl Widget for ChatLineLoading {
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

impl ChatLineLoading {
    pub fn update_animation(&mut self, cx: &mut Cx) {
        self.current_animated_bar = (self.current_animated_bar + 1) % 3;

        match self.current_animated_bar {
            0 => {
                self.animator_play(cx, id!(line1.run));
                self.animator_play(cx, id!(line3.start));
            }
            1 => {
                self.animator_play(cx, id!(line1.start));
                self.animator_play(cx, id!(line2.run));
            }
            2 => {
                self.animator_play(cx, id!(line2.start));
                self.animator_play(cx, id!(line3.run));
            }
            _ => unreachable!(),
        };

        self.timer = cx.start_timeout(0.33);
    }
}

impl ChatLineLoadingRef {
    pub fn animate(&mut self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        if inner.timer.is_empty() {
            inner.timer = cx.start_timeout(0.2);
        }
    }

    pub fn stop_animation(&mut self) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.timer = Timer::default();
    }
}
