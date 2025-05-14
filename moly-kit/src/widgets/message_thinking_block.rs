use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;
    use link::moly_kit_theme::*;
    use crate::widgets::message_markdown::MessageMarkdown;

    ICON_COLLAPSE = dep("crate://self/resources/icons/collapse.svg")
    ANIMATION_SPEED = 1.2;

    Collapse = <RoundedView> {
        width: Fill, height: Fit
        padding: {top: 8, right: 12, bottom: 8, left: 12},
        margin: 2
        cursor: Hand

        draw_bg: {
            border_radius: 2.5
            instance dither: 0.9

            fn get_color(self) -> vec4 {
                return mix(
                   #f7f7f7,
                    #677483,
                    self.pos.x + self.dither
                )
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_size,
                    self.border_size,
                    self.rect_size.x - (self.border_size * 2.0),
                    self.rect_size.y - (self.border_size * 2.0),
                    max(1.0, self.border_radius)
                )
                sdf.fill_keep(self.get_color())
                if self.border_size > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_size)
                }
                return sdf.result;
            }
        }

        <Label> {
            text: "Thinking..."
            draw_text: {
                text_style: {
                    font_size: 10.5
                }
                color: #000
            }
        }
    }

    Content = <RoundedView> {
        width: Fill,
        height: Fit,

        flow: Right,
        spacing: 12,
        height: 0,
        padding: {left: 20, right: 8, top: 10, bottom: 15},

        thinking_text = <MessageMarkdown> {
            width: Fill, height: Fit,
            font_size: 10.5
        }
    }

    pub MessageThinkingBlock = {{MessageThinkingBlock}} {
        width: Fill,
        height: Fit,
        flow: Down,
        show_bg: true,
        padding: {top: 5, bottom: 5, left: 5, right: 5}

        inner = <RoundedShadowView> {
            width: Fill, height: Fit,
            flow: Down
            padding: 0
            draw_bg: {
                color: #f7f7f7,
                border_radius: 4.5,
                uniform shadow_color: #0001
                shadow_radius: 9.0,
                shadow_offset: vec2(0.0,-1.0)
            }
            collapse = <Collapse> {}
            content = <Content> {}
        }

        animator: {
            loading = {
                default: run,
                restart = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {inner = { collapse = { draw_bg: {dither: 0.6} }}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {inner = { collapse = { draw_bg: {dither: 0.0} }}}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct MessageThinkingBlock {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    thinking_text: Option<String>,

    #[rust]
    timer: Timer,

    #[rust]
    is_expanded: bool,

    #[rust]
    should_animate: bool,
}

impl Widget for MessageThinkingBlock {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.update_animation(cx);
        }

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(text) = &self.thinking_text {
            // Use message_markdown widget to render the thinking text
            self.markdown(id!(thinking_text)).set_text(cx, text);
            self.view.draw_walk(cx, scope, walk)
        } else {
            DrawStep::done()
        }
    }
}

impl WidgetMatchEvent for MessageThinkingBlock {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(_evt) = self.view(id!(collapse)).finger_up(&actions) {
            self.toggle_collapse(cx);
        }
    }
}

impl MessageThinkingBlock {
    pub fn update_animation(&mut self, cx: &mut Cx) {
        if self.animator_in_state(cx, id!(loading.restart)) {
            if self.should_animate {
                self.timer = cx.start_timeout(0.5);
                self.animator_play(cx, id!(loading.run));
            } else {
                self.animator_play(cx, id!(loading.restart));
            }
        } else {
            self.animator_play(cx, id!(loading.restart));
            self.timer = cx.start_timeout(0.5);
        }
    }

    pub fn set_thinking_text(&mut self, cx: &mut Cx, text: Option<String>, is_writing: bool) {
        self.thinking_text = text;
        if is_writing {
            if self.timer.is_empty() {
                self.should_animate = true;
                self.timer = cx.start_timeout(0.5);
            }
        } else {
            self.should_animate = false;
            self.animator_play(cx, id!(loading.restart));
        }
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
        } else {
            self.view(id!(content)).apply_over(
                cx,
                live! {
                    height: 0.0
                },
            );
            self.should_animate = false;
        }
        self.redraw(cx);
    }
}

impl MessageThinkingBlockRef {
    pub fn set_thinking_text(&mut self, cx: &mut Cx, text: Option<String>, is_writing: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_thinking_text(cx, text, is_writing);
        }
    }
}
