use crate::{protocol::*, widgets::attachment_list::AttachmentListWidgetExt};
use makepad_widgets::*;

use super::{
    citation_list::CitationListWidgetExt, message_thinking_block::MessageThinkingBlockWidgetExt,
};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;

    use crate::widgets::message_thinking_block::*;
    use crate::widgets::message_markdown::*;
    use crate::widgets::citation_list::*;
    use crate::widgets::attachment_list::*;

    pub StandardMessageContent = {{StandardMessageContent}} {
        flow: Down
        height: Fit,
        spacing: 10
        thinking_block = <MessageThinkingBlock> {}
        markdown = <MessageMarkdown> {}
        citations = <CitationList> { visible: false }
        attachments = <AttachmentList> {}
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct StandardMessageContent {
    #[deref]
    deref: View,

    #[rust]
    smooth_typing: SmoothTyping,
}

/// The state of the virtual typing animation.
///
/// Used to simulate someone typing the message.
#[derive(Default)]
struct SmoothTyping {
    pub target_text: String,
    pub current_char_len: usize,
    pub typing_speed_chars_sec: usize,
    pub last_update: f64,
    pub next_frame: NextFrame,
    /// Flag to track if an Ended action has already been dispatched for this animation cycle
    pub ended_action_dispatched: bool,
}

const DEFAULT_TYPING_SPEED_CHARS_SEC: usize = 200;
const TYPING_ANIMATION_CHAR: &str = "●";

impl Widget for StandardMessageContent {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        match event {
            Event::NextFrame(frame_event) => {
                if frame_event.set.contains(&self.smooth_typing.next_frame) {
                    if !self.smooth_typing.target_text.is_empty()
                        && self.smooth_typing.current_char_len
                            < self.smooth_typing.target_text.chars().count()
                    {
                        self.animate_typing(cx, frame_event.time);
                    }
                }
            }
            _ => (),
        }
        self.deref.handle_event(cx, event, scope);
    }
}

impl StandardMessageContent {
    /// Set a message content to display it.
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent, is_writing: bool) {
        let citation_list = self.citation_list(id!(citations));
        citation_list.borrow_mut().unwrap().urls = content.citations.clone();
        citation_list.borrow_mut().unwrap().visible = !content.citations.is_empty();

        if let Some(reasoning) = &content.reasoning {
            self.message_thinking_block(id!(thinking_block))
                .set_thinking_content(cx, reasoning, is_writing);
        }

        let attachments = self.attachment_list(id!(attachments));
        attachments.borrow_mut().unwrap().attachments = content.attachments.clone();

        if !content.text.is_empty() {
            if is_writing {
                // Check if we're starting a new message or changing the text
                let is_new_message = self.smooth_typing.target_text != content.text;

                // Track if we need to send animation started
                let was_empty = self.smooth_typing.target_text.is_empty()
                    || self.smooth_typing.current_char_len == 0
                    || is_new_message;

                if is_new_message {
                    self.smooth_typing.ended_action_dispatched = false;
                }

                self.smooth_typing.target_text = content.text.clone();

                if self.smooth_typing.typing_speed_chars_sec == 0 {
                    self.smooth_typing.typing_speed_chars_sec = DEFAULT_TYPING_SPEED_CHARS_SEC;
                }

                if self.smooth_typing.current_char_len
                    < self.smooth_typing.target_text.chars().count()
                {
                    if was_empty {
                        cx.widget_action(
                            self.widget_uid(),
                            &Scope::empty().path,
                            MessageAnimationAction::Started,
                        );
                    }
                    self.smooth_typing.next_frame = cx.new_next_frame();
                }
            } else {
                // For non-writing messages, check if we're in the middle of typing animation
                let body_chars = content.text.chars().count();
                let currently_showing = if self.smooth_typing.target_text == content.text {
                    // If target text is already this message, use current_char_len
                    self.smooth_typing.current_char_len
                } else {
                    // Otherwise show it completely
                    self.smooth_typing.ended_action_dispatched = false;
                    body_chars
                };

                // If we're in the middle of typing this exact message, continue animation
                if self.smooth_typing.target_text == content.text && currently_showing < body_chars
                {
                    // Keep the animation going to completion
                    self.smooth_typing.next_frame = cx.new_next_frame();
                } else {
                    // Either a different message or already showing completely,
                    // so display it immediately
                    self.label(id!(markdown)).set_text(cx, &content.text);
                    self.smooth_typing.target_text = content.text.clone();
                    self.smooth_typing.current_char_len = body_chars;
                    self.smooth_typing.last_update = 0.0;
                }
            }
        } else {
            // No body text
            self.label(id!(markdown)).set_text(cx, "");
            self.smooth_typing.target_text.clear();
            self.smooth_typing.current_char_len = 0;
            self.smooth_typing.last_update = 0.0;
            self.smooth_typing.ended_action_dispatched = false;
        }
    }

    fn animate_typing(&mut self, cx: &mut Cx, time: f64) {
        if self.smooth_typing.target_text.is_empty()
            || self.smooth_typing.typing_speed_chars_sec == 0
        {
            return;
        }

        // If we've already shown the entire message, don't animate
        let target_char_count = self.smooth_typing.target_text.chars().count();
        if self.smooth_typing.current_char_len >= target_char_count {
            return;
        }

        let current_frame_time = time;
        if self.smooth_typing.last_update == 0.0 {
            self.smooth_typing.last_update = current_frame_time;
        }

        let time_delta = current_frame_time - self.smooth_typing.last_update;

        if time_delta <= 0.0 && self.smooth_typing.current_char_len < target_char_count {
            self.smooth_typing.next_frame = cx.new_next_frame();
            return;
        }

        // Calculate how many characters to reveal based on time delta
        let chars_to_reveal_float = time_delta * self.smooth_typing.typing_speed_chars_sec as f64;

        if chars_to_reveal_float >= 1.0 {
            let num_chars_to_add = chars_to_reveal_float.floor() as usize;

            let prev_len = self.smooth_typing.current_char_len;
            let new_len = self.smooth_typing.current_char_len + num_chars_to_add;
            self.smooth_typing.current_char_len = new_len.min(target_char_count);

            let mut display_text = self
                .smooth_typing
                .target_text
                .chars()
                .take(self.smooth_typing.current_char_len)
                .collect::<String>();

            // Add a character at the end to simulate typing
            display_text.push_str(format!(" {}", TYPING_ANIMATION_CHAR).as_str());

            self.label(id!(markdown)).set_text(cx, &display_text);
            self.smooth_typing.last_update = current_frame_time;

            // The animation completed naturally
            if prev_len < target_char_count
                && self.smooth_typing.current_char_len >= target_char_count
                && !self.smooth_typing.ended_action_dispatched
            {
                cx.widget_action(
                    self.widget_uid(),
                    &Scope::empty().path,
                    MessageAnimationAction::Ended,
                );
                self.smooth_typing.ended_action_dispatched = true;
            }
        }

        if self.smooth_typing.current_char_len < target_char_count {
            self.smooth_typing.next_frame = cx.new_next_frame();
        }
    }
}

impl StandardMessageContentRef {
    /// See [StandardMessageContent::set_content].
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent, is_writing: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.set_content(cx, content, is_writing);
    }
}

/// Action emitted by StandardMessageContent to notify about animation state changes.
#[derive(Debug, PartialEq, Copy, Clone, DefaultNone)]
pub enum MessageAnimationAction {
    /// Animation for smooth typing has started.
    Started,
    /// Animation for smooth typing has ended.
    Ended,
    None,
}
