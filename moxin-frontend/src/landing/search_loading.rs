use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_card::ModelCard;

    SearchLoading = {{SearchLoading}} {
        width: Fill,
        height: Fill,

        flow: Down,
        spacing: 60,
        align: {x: 0.5, y: 0.5},

        content = <View> {
            width: Fit,
            height: Fit,
            spacing: 80,
            circle1 = <CircleView> {
                width: 48,
                height: 48,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 24.0,
                }
            }
            circle2 = <CircleView> {
                width: 48,
                height: 48,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 24.0,
                }
            }
            circle3 = <CircleView> {
                width: 48,
                height: 48,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 24.0,
                }
            }
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
                    from: {all: Snap}
                    apply: {content = { circle1 = { draw_bg: {radius: 24.0} }}}
                }
                run = {
                    redraw: true,
                    from: {all: BounceLoop {duration: 0.6, end: 1.0}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {content = { circle1 = { draw_bg: {radius: 0.0} }}}
                }
            }

            circle2 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Snap}
                    apply: {content = { circle2 = { draw_bg: {radius: 24.0} }}}
                }
                run = {
                    redraw: true,
                    from: {all: BounceLoop {duration: 0.6, end: 1.0}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {content = { circle2 = { draw_bg: {radius: 0.0} }}}
                }
            }

            circle3 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Snap}
                    apply: {content = { circle3 = { draw_bg: {radius: 24.0} }}}
                }
                run = {
                    redraw: true,
                    from: {all: BounceLoop {duration: 0.6, end: 1.0}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {content = { circle3 = { draw_bg: {radius: 0.0} }}}
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
        if let Event::Startup = event {
            self.update_animation(cx);
        }
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
                self.animator_play(cx, id!(circle2.start));
                self.animator_play(cx, id!(circle3.start));
            }
            1 => {
                self.animator_play(cx, id!(circle1.start));
                self.animator_play(cx, id!(circle2.run));
                self.animator_play(cx, id!(circle3.start));
            
            }
            2 => {
                self.animator_play(cx, id!(circle1.start));
                self.animator_play(cx, id!(circle2.start));
                self.animator_play(cx, id!(circle3.run));
            }
            _ => unreachable!(),
        };

        self.timer = cx.start_timeout(0.5);
    }
}