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
    fn set_content_impl(
        &mut self,
        cx: &mut Cx,
        content: &MessageContent,
        metadata: &MessageMetadata,
    ) {
        /// String to add as suffix to the message text when its being typed.
        const TYPING_INDICATOR: &str = "●";

        let citation_list = self.citation_list(id!(citations));
        citation_list.borrow_mut().unwrap().urls = content.citations.clone();
        citation_list.borrow_mut().unwrap().visible = !content.citations.is_empty();

        let mut attachments = self.attachment_list(id!(attachments));
        attachments.write().attachments = content.attachments.clone();
        attachments.write().on_tap = Some(Box::new(|list, index| {
            if let Some(attachment) = list.attachments.get(index).cloned() {
                attachment.save();
            }
        }));

        self.message_thinking_block(id!(thinking_block))
            .borrow_mut()
            .unwrap()
            .set_content(cx, content, metadata);

        let markdown = self.label(id!(markdown));
        if metadata.is_writing() {
            let text_with_typing = format!("{} {}", content.text, TYPING_INDICATOR);
            markdown.set_text(cx, &text_with_typing);
        } else {
            markdown.set_text(cx, &content.text);
        }
    }

    /// Set a message content to display it.
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        self.set_content_impl(cx, content, &MessageMetadata::new());
    }

    /// Same as [`set_content`], but also passes down metadata which is required
    /// by certain features.
    pub fn set_content_with_metadata(
        &mut self,
        cx: &mut Cx,
        content: &MessageContent,
        metadata: &MessageMetadata,
    ) {
        self.set_content_impl(cx, content, metadata);
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
    pub fn set_content_with_metadata(
        &mut self,
        cx: &mut Cx,
        content: &MessageContent,
        metadata: &MessageMetadata,
    ) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.set_content_with_metadata(cx, content, metadata);
    }
}
