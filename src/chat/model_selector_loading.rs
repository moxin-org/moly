use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::landing::model_card::ModelCard;

    ANIMATION_SPEED = 1.5;

    Bar = <RoundedView> {
        width: Fill,
        height: 8,
        show_bg: true,
        draw_bg: {
            instance radius: 1.0,
            instance dither: 0.9

            fn get_color(self) -> vec4 {
                return mix(
                    #F3FFA2,
                    #E3FBFF,
                    self.pos.x + self.dither
                )
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.inset.x + self.border_width,
                    self.inset.y + self.border_width,
                    self.rect_size.x - (self.inset.x + self.inset.z + self.border_width * 2.0),
                    self.rect_size.y - (self.inset.y + self.inset.w + self.border_width * 2.0),
                    max(1.0, self.radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_width > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_width)
                }
                return sdf.result;
            }
        }
    }

    pub ModelSelectorLoading = {{ModelSelectorLoading}} {
        width: Fill,
        height: Fill,
        align: {x: 0, y: 1},

        line = <Bar> {}

        animator: {
            line = {
                default: restart,
                restart = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line = { draw_bg: {dither: 0.6} }}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {line = { draw_bg: {dither: 0.0} }}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorLoading {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    timer: Timer,
}

impl Widget for ModelSelectorLoading {
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

impl ModelSelectorLoading {
    pub fn update_animation(&mut self, cx: &mut Cx) {
        self.visible = true;
        if self.animator_in_state(cx, id!(line.restart)) {
            self.animator_play(cx, id!(line.run));
        } else {
            self.animator_play(cx, id!(line.restart));
        }
        self.timer = cx.start_timeout(1.5);
    }
}

impl ModelSelectorLoadingRef {
    pub fn _show_and_animate(&mut self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        if inner.timer.is_empty() {
            inner.timer = cx.start_timeout(0.2);
        }
    }

    pub fn _hide(&mut self) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.visible = false;
        inner.timer = Timer::default();
    }
}
