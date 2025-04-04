use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::widgets::message_markdown::MessageMarkdown;

    ICON_COLLAPSE = dep("crate://self/resources/icons/collapse.svg")

    CollapseButton = <Button> {
        width: Fit,
        height: Fit,
        padding: {top: 8, right: 12, bottom: 8, left: 12},
        margin: 2,

        text: "Thinking >"

        draw_text: {
            color: #000
        }
        icon_walk: {
            width: 12,
            height: 12
            margin: {top: 0, left: -4},
        }
    }


    Content = <RoundedView> {
        width: Fill,
        height: Fit,

        flow: Right,
        spacing: 12,

        show_bg: true
        draw_bg: {
            color: #e0
            border_radius: 5
        }

        thinking_text = <MessageMarkdown> {
            width: Fill, height: Fit
        }
    }

    pub MessageThinkingBlock = {{MessageThinkingBlock}} {
        width: Fill,
        height: Fit,
        flow: Down,
        show_bg: true,
        padding: 5

        collapse = <CollapseButton> {}
        content = <Content> {
            height: 0,
            padding: {left: 43, right: 43, top: 12, bottom: 12},
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct MessageThinkingBlock {
    #[deref]
    view: View,

    #[rust]
    thinking_text: Option<String>,

    #[rust]
    is_expanded: bool,
}

impl Widget for MessageThinkingBlock {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(text) = &self.thinking_text {
            // Use message_markdown widget to render the thinking text
            dbg!(self.markdown(id!(thinking_text)), text);
            self.markdown(id!(thinking_text)).set_text(cx, text);
            self.view.draw_walk(cx, scope, walk)
        } else {
            DrawStep::done()
        }
    }
}

impl WidgetMatchEvent for MessageThinkingBlock {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.button(id!(collapse)).clicked(&actions) {
            self.toggle_collapse(cx);
        }
    }
}

impl MessageThinkingBlock {
    pub fn set_thinking_text(&mut self, text: Option<String>) {
        self.thinking_text = text;
    }

    fn toggle_collapse(&mut self, cx: &mut Cx) {
        self.is_expanded = !self.is_expanded;

        if self.is_expanded {
            self.view(id!(content)).apply_over(
                cx,
                live! {
                    height: Fit
                },
            );
            self.set_collapse_button_open(cx, true);
        } else {
            self.view(id!(content)).apply_over(
                cx,
                live! {
                    height: 0.0
                },
            );
            self.set_collapse_button_open(cx, false);
        }
        self.redraw(cx);
    }

    fn set_collapse_button_open(&mut self, cx: &mut Cx, is_open: bool) {
        let rotation_angle = if is_open { 0.0 } else { 180.0 };
        self.button(id!(collapse)).apply_over(
            cx,
            live! {
                draw_icon: { rotation_angle: (rotation_angle) }
            },
        );
    }
}

impl MessageThinkingBlockRef {
    pub fn set_thinking_text(&mut self, text: Option<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_thinking_text(text);
        }
    }
}
