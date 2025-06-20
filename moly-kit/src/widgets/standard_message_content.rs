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
}

impl Widget for StandardMessageContent {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}

impl StandardMessageContent {
    fn set_content_impl(&mut self, cx: &mut Cx, content: &MessageContent, is_typing: bool) {
        /// String to add as suffix to the message text when its being typed.
        const TYPING_INDICATOR: &str = "●";

        let citation_list = self.citation_list(id!(citations));
        citation_list.borrow_mut().unwrap().urls = content.citations.clone();
        citation_list.borrow_mut().unwrap().visible = !content.citations.is_empty();

        let attachments = self.attachment_list(id!(attachments));
        attachments.borrow_mut().unwrap().attachments = content.attachments.clone();

        if let Some(reasoning) = &content.reasoning {
            self.message_thinking_block(id!(thinking_block))
                .set_thinking_content(cx, reasoning, is_typing);
        }

        let markdown = self.label(id!(markdown));
        if is_typing {
            let text_with_typing = format!("{} {}", content.text, TYPING_INDICATOR);
            markdown.set_text(cx, &text_with_typing);
        } else {
            markdown.set_text(cx, &content.text);
        }
    }

    /// Set a message content to display it.
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        self.set_content_impl(cx, content, false);
    }

    /// Same as [`set_content`], with an optional typing indicator automatically added.
    pub fn set_content_with_typing(
        &mut self,
        cx: &mut Cx,
        content: &MessageContent,
        is_typing: bool,
    ) {
        self.set_content_impl(cx, content, is_typing);
    }
}

impl StandardMessageContentRef {
    /// See [`StandardMessageContent::set_content`].
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.set_content(cx, content);
    }

    /// See [`StandardMessageContent::set_content_with_typing`].
    pub fn set_content_with_typing(
        &mut self,
        cx: &mut Cx,
        content: &MessageContent,
        is_typing: bool,
    ) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.set_content_with_typing(cx, content, is_typing);
    }
}
