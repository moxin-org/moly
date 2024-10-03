use super::ui_runner::UiRunner;
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
    }
}

#[derive(Live, Widget)]
pub struct Spinner {
    #[deref]
    view: View,

    #[rust]
    last_angle: f32,

    #[rust]
    ui_runner: UiRunner,
}

impl LiveHook for Spinner {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // TODO: I'm in a hurry, don't judge me. I will fix this later.
        let ui = self.ui_runner;
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            ui.run::<Self>(|s, cx| {
                s.last_angle = s.last_angle + 0.5;
                s.image(id!(img)).apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            rotation: (s.last_angle),
                        }
                    },
                );
                s.redraw(cx);
            });
        });
    }
}

impl Widget for Spinner {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.ui_runner.handle(cx, event, self);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
