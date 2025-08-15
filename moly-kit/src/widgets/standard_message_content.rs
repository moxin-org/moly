use crate::{
    protocol::*,
    widgets::{
        attachment_list::AttachmentListWidgetExt,
        attachment_viewer_modal::AttachmentViewerModalWidgetExt,
    },
};
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
    use crate::widgets::attachment_viewer_modal::*;

    pub StandardMessageContent = {{StandardMessageContent}} {
        flow: Down
        height: Fit,
        spacing: 5
        thinking_block = <MessageThinkingBlock> {}
        markdown = <MessageMarkdown> {}
        citations = <CitationList> { visible: false }
        attachments = <AttachmentList> {}
        attachment_viewer_modal = <AttachmentViewerModal> {}
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
        self.ui_runner().handle(cx, event, scope, self);
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

        let ui = self.ui_runner();
        attachments.write().on_tap(move |list, index| {
            if let Some(attachment) = list.attachments.get(index).cloned() {
                if crate::widgets::attachment_view::can_preview(&attachment) {
                    ui.defer(move |me, cx, _| {
                        let modal = me.attachment_viewer_modal(id!(attachment_viewer_modal));
                        modal.borrow_mut().unwrap().open(cx, attachment);
                    });
                } else {
                    attachment.save();
                }
            }
        });

        self.message_thinking_block(id!(thinking_block))
            .borrow_mut()
            .unwrap()
            .set_content(cx, content, metadata);

        let markdown = self.label(id!(markdown));

        // Create enhanced text that includes tool calls
        let enhanced_text = if !content.tool_calls.is_empty() {
            let mut text = content.text.clone();
            if !text.is_empty() {
                text.push_str("\n\n");
            }

            if content.tool_calls.len() == 1 {
                let tool_call = &content.tool_calls[0];
                let args_str = if tool_call.arguments.is_empty() {
                    "no parameters".to_string()
                } else {
                    format!(
                        "parameters: {}",
                        tool_call
                            .arguments
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                text.push_str(&format!(
                    "🔧 **Calling tool:** `{}` with {}",
                    tool_call.name, args_str
                ));
            } else {
                text.push_str(&format!(
                    "🔧 **Calling {} tools:**\n",
                    content.tool_calls.len()
                ));
                for tool_call in &content.tool_calls {
                    let args_str = if tool_call.arguments.is_empty() {
                        "no parameters".to_string()
                    } else {
                        format!(
                            "parameters: {}",
                            tool_call
                                .arguments
                                .iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    text.push_str(&format!("- `{}` with {}\n", tool_call.name, args_str));
                }
            }
            text
        } else {
            content.text.clone()
        };

        if metadata.is_writing() {
            let text_with_typing = format!("{} {}", enhanced_text, TYPING_INDICATOR);
            markdown.set_text(cx, &text_with_typing);
        } else {
            markdown.set_text(cx, &enhanced_text);
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
