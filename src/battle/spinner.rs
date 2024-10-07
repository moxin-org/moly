use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::styles::*;

    Spinner = {{Spinner}} {
        flow: Down,
        spacing: (SM_GAP),
        align: {x: 0.5, y: 0.5},
        height: Fit,
        width: Fit,

        img = <RotatedImage> {
            source: dep("crate://self/resources/icons/prerendered/output/spinner.png"),
            width: 50,
            height: 50,
            draw_bg: {
                rotation: 180.,
            }
        }

        <Label> {
            draw_text: {
                text_style: {font_size: 10},
                color: #000
            }
            text: ""
        }

        animator: {
            spinner = {
                default: off
                off = {
                    from: {all: Forward {duration: 2.0}}
                    apply: {
                        img = { draw_bg: { rotation: 0. } }
                    }
                }
                spin = {
                    redraw: true,
                    from: {all: Loop {duration: 1.0, end: 1.0}}
                    apply: {
                        img = { draw_bg: { rotation: 6.283 } }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Spinner {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,
}

impl Widget for Spinner {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        if self.animator.need_init() {
            self.animator_play(cx, id!(spinner.spin));
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
