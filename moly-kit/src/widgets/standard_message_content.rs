use crate::protocol::*;
use makepad_widgets::*;

use super::{
    citation_list::CitationListWidgetExt, message_thinking_block::MessageThinkingBlockWidgetExt,
};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::widgets::message_thinking_block::*;
    use crate::widgets::message_markdown::*;
    use crate::widgets::citation_list::*;

    pub StandardMessageContent = {{StandardMessageContent}} {
        flow: Down
        height: Fit,
        spacing: 10
        thinking_block = <MessageThinkingBlock> {}
        markdown = <MessageMarkdown> {}
        citations = <CitationList> { visible: false }
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
    /// Set a message content to display it.
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        let citation_list = self.citation_list(id!(citations));
        citation_list.borrow_mut().unwrap().urls = content.citations.clone();
        citation_list.borrow_mut().unwrap().visible = !content.citations.is_empty();

        let (thinking_block, message_body) = extract_and_remove_think_tag(&content.text);
        self.message_thinking_block(id!(thinking_block))
            .set_thinking_text(thinking_block);

        // Workaround: Because I had to set `paragraph_spacing` to 0 in `MessageMarkdown`,
        // we need to add a "blank" line as a workaround.
        //
        // Warning: If you ever read the text from this widget and not
        // from the list, you should remove the unicode character.
        // TODO: Remove this workaround once the markdown widget is fixed.
        if let Some(body) = message_body {
            self.label(id!(markdown))
                .set_text(cx, &body.replace("\n\n", "\n\n\u{00A0}\n\n"));
        }
    }
}

impl StandardMessageContentRef {
    /// See [StandardMessageContent::set_content].
    pub fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        inner.set_content(cx, content);
    }
}

fn extract_and_remove_think_tag(text: &str) -> (Option<String>, Option<String>) {
    let (start_tag, end_tag) = ("<think>", "</think>");

    let start_search = text.find(start_tag);
    let end_search = text.find(end_tag);

    let Some(start) = start_search else {
        return (None, Some(text.to_string()));
    };

    let thinking_content = if let Some(end) = end_search {
        text[start + start_tag.len()..end].trim().to_string()
    } else {
        text[start + start_tag.len()..].trim().to_string()
    };

    let thinking = if thinking_content.len() > 0 {
        Some(thinking_content)
    } else {
        None
    };

    let body = if let Some(end) = end_search {
        let body = text[end + end_tag.len()..].trim().to_string();
        Some(body)
    } else {
        None
    };

    (thinking, body)
}
