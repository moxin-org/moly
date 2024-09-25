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

        <Icon> {
            draw_icon: {
                svg_file: dep("crate://self/resources/icons/discover.svg"),
                fn get_color(self) -> vec4 {
                    return #127487;
                }
            }
            icon_walk: {width: 50, height: 50}
        }

        <Label> {
            draw_text: {
                text_style: {font_size: 10},
                color: #000
            }
            text: "Trust me, I'm spinning."
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Spinner {
    #[deref]
    view: View,
}

impl Widget for Spinner {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
