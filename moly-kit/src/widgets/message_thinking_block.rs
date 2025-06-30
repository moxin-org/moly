use crate::protocol::*;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;
    use link::moly_kit_theme::*;
    use crate::widgets::message_markdown::MessageMarkdown;

    ICON_COLLAPSE = dep("crate://self/resources/icons/collapse.svg")
    ANIMATION_SPEED = 0.66
    BALL_MAX_SIZE = 20.0
    BALL_MIN_SIZE = 10.0
    BALL_SPACING = 0.0

    LoadingBall = <CircleView> {
        width: (BALL_MAX_SIZE)
        height: (BALL_MAX_SIZE)
        margin: 0.0
        padding: 0.0
        draw_bg: {
            border_radius: (BALL_MAX_SIZE / 2.0)
        }
    }

    PulsingBalls = <View> {
        width: Fit
        height: Fit
        align: {x: 0.0, y: 0.5}
        spacing: (BALL_SPACING)
        flow: Right
        padding: 0
        margin: 0

        ball1 = <LoadingBall> {
            margin: 0.0
            padding: 0.0
            draw_bg: {
                color: #E55E50
            }
        }

        ball2 = <LoadingBall> {
            margin: 0.0
            padding: 0.0
            draw_bg: {
                color: #4D9CC0
            }
        }
    }

    Collapse = <RoundedView> {
        width: Fill, height: Fit
        padding: {top: 8, right: 12, bottom: 8, left: 12},
        margin: 2
        cursor: Hand
        flow: Right
        align: {x: 0.0, y: 0.5}

        draw_bg: {
            border_radius: 2.5
            color: #f7f7f7
        }

        thinking_title = <Label> {
            text: "Thinking..."
            draw_text: {
                text_style: <THEME_FONT_ITALIC> {
                    font_size: 10.5
                }
                color: #000
            }
        }

        <View> { width: Fill, height: Fill }
        balls = <PulsingBalls> {}
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
            width: 200, height: Fit,
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
            ball1 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {inner = { collapse = { balls = { ball1 = { width: (BALL_MIN_SIZE), height: (BALL_MIN_SIZE), draw_bg: {border_radius: (BALL_MIN_SIZE / 2.0)} }}}}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {inner = { collapse = { balls = { ball1 = { width: (BALL_MAX_SIZE), height: (BALL_MAX_SIZE), draw_bg: {border_radius: (BALL_MAX_SIZE / 2.0)} }}}}}
                }
            }

            ball2 = {
                default: start,
                start = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {inner = { collapse = { balls = { ball2 = { width: (BALL_MIN_SIZE), height: (BALL_MIN_SIZE), draw_bg: {border_radius: (BALL_MIN_SIZE / 2.0)} }}}}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {inner = { collapse = { balls = { ball2 = { width: (BALL_MAX_SIZE), height: (BALL_MAX_SIZE), draw_bg: {border_radius: (BALL_MAX_SIZE / 2.0)} }}}}}
                }
            }
        }
    }
}

const ANIMATION_SPEED_RUST: f64 = 0.33;

#[derive(Live, LiveHook, Widget)]
pub struct MessageThinkingBlock {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    timer: Timer,

    #[rust]
    is_expanded: bool,

    #[rust]
    is_visible: bool,

    #[rust]
    should_animate: bool,

    #[rust]
    current_animated_ball: usize,
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
        if self.is_visible {
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
        // Alternate between animating the first and second ball
        self.current_animated_ball = (self.current_animated_ball + 1) % 2;

        match self.current_animated_ball {
            0 => {
                self.animator_play(cx, id!(ball1.run));
                self.animator_play(cx, id!(ball2.start));
            }
            1 => {
                self.animator_play(cx, id!(ball1.start));
                self.animator_play(cx, id!(ball2.run));
            }
            _ => unreachable!(),
        }

        // Schedule the next animation step
        self.timer = cx.start_timeout(ANIMATION_SPEED_RUST);
    }

    pub fn set_content(
        &mut self,
        cx: &mut Cx,
        content: &MessageContent,
        metadata: &MessageMetadata,
    ) {
        let content_reasoning = content.reasoning.as_str();
        let content_text = content.text.as_str();

        self.is_visible = !content_reasoning.is_empty();

        self.markdown(id!(thinking_text))
            .set_text(cx, content_reasoning);

        let is_reasoning_ongoing =
            !content_reasoning.is_empty() && content_text.is_empty() && metadata.is_writing();

        if is_reasoning_ongoing {
            if self.timer.is_empty() {
                self.should_animate = true;
                self.view(id!(balls)).set_visible(cx, true);
                self.update_animation(cx);
            }
        } else {
            self.should_animate = false;
            self.view(id!(balls)).set_visible(cx, false);
            self.animator_play(cx, id!(ball1.start));
            self.animator_play(cx, id!(ball2.start));
            self.view(id!(thinking_title)).set_text(
                cx,
                &format!(
                    "Thought for {:0.2} seconds",
                    metadata.reasoning_time_taken_seconds()
                ),
            );
        }
    }

    fn toggle_collapse(&mut self, cx: &mut Cx) {
        self.is_expanded = !self.is_expanded;

        if self.is_expanded {
            // Expand the content to fit the text
            self.view(id!(content)).apply_over(
                cx,
                live! {
                    height: Fit
                },
            );
            // Expand the inner view to fit the content
            self.view(id!(inner)).apply_over(
                cx,
                live! {
                    width: Fill
                },
            );
            // Set a different color to the title background
            self.view(id!(collapse)).apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: #f0f0f0
                    }
                },
            );
        } else {
            // Collapse the content
            self.view(id!(content)).apply_over(
                cx,
                live! {
                    height: 0.0
                },
            );
            // Set the inner view width back to the default
            self.view(id!(inner)).apply_over(
                cx,
                live! {
                    width: 200
                },
            );
            // Set the title background color back to the default
            self.view(id!(collapse)).apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: #f7f7f7
                    }
                },
            );
            self.should_animate = false;
        }
        self.redraw(cx);
    }
}
