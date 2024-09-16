use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    Prompt = {{Prompt}} <RoundedView> {
        height: Fit,
        align: {y: 0.5}
        padding: {top: 8, bottom: 8, left: 10, right: 10},
        show_bg: true,
        draw_bg: {
            border_width: 1.0,
            border_color: #D0D5DD,
            color: #fff,
            radius: 5.0
        }
        input = <MoxinTextInput> {
            draw_label: {
                text_style: <REGULAR_FONT> { font_size: 11 },
            }
            draw_bg: {
                color: vec4(0, 0, 0, 0),
            },
            width: Fill,
            height: Fit,
            empty_message: "Enter a message",
        },
        button = <MoxinButton> {
            height: 35,
            draw_bg: {
                color: #000,
            },
            text: "Fight!",
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Prompt {
    #[deref]
    view: View,
}

impl Widget for Prompt {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for Prompt {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {}
}
