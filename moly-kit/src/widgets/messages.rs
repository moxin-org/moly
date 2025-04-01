use std::cell::{Ref, RefMut};

use crate::{
    protocol::*,
    utils::{events::EventExt, portal_list::ItemsRangeIter},
    widgets::{
        avatar::AvatarWidgetRefExt, message_loading::MessageLoadingWidgetRefExt,
        message_thinking_block::MessageThinkingBlockWidgetRefExt,
    },
};
use makepad_widgets::*;

use super::{citations::CitationsWidgetRefExt, deep_inquire_line::DeepInquireBotLineWidgetRefExt};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::widgets::chat_lines::*;
    use crate::widgets::deep_inquire_line::*;

    pub Messages = {{Messages}} {
        flow: Overlay,
        list = <PortalList> {
            scroll_bar: {
                bar_size: 0.0,
            }
            UserLine = <UserLine> {}
            BotLine = <BotLine> {}
            DeepInquireBotLine = <DeepInquireBotLine> {}
            LoadingLine = <LoadingLine> {}
            AppLine = <AppLine> {}
            ErrorLine = <ErrorLine> {}

            // Acts as marker for:
            // - Knowing if the end of the list has been reached.
            // - To jump to bottom with proper precision.
            EndOfChat = <View> {height: 0.1}
        }
        <View> {
            align: {x: 1.0, y: 1.0},
            jump_to_bottom = <Button> {
                width: 34,
                height: 34,
                margin: 2,
                padding: {bottom: 2},
                icon_walk: {width: 12, height: 12}
                draw_icon: {
                    svg_file: dep("crate://self/resources/jump_to_bottom.svg")
                    color: #1C1B1F,
                }
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        let center = self.rect_size * 0.5;
                        let radius = min(self.rect_size.x, self.rect_size.y) * 0.5;

                        sdf.circle(center.x, center.y, radius - 1.0);
                        sdf.fill_keep(#fff);
                        sdf.stroke(#EAECF0, 1.0);

                        return sdf.result
                    }
                }
            }
        }
    }
}

/// Relevant actions that should be handled by a parent.
///
/// If includes an index, it refers to the index of the message in the list.
#[derive(Debug, PartialEq, Copy, Clone, DefaultNone)]
pub enum MessagesAction {
    /// The message at the given index should be copied.
    Copy(usize),

    /// The message at the given index should be deleted.
    Delete(usize),

    /// The message at the given index should be edited and saved.
    EditSave(usize),

    /// The message at the given index should be edited, saved and the messages
    /// history should be regenerated from here.
    EditRegenerate(usize),

    None,
}

/// Represents the current open editor for a message.
#[derive(Debug)]
struct Editor {
    index: usize,
    buffer: String,
}

/// View over a conversation with messages.
///
/// This is mostly a dummy widget. Prefer using and adapting [crate::widgets::chat::Chat] instead.
#[derive(Live, LiveHook, Widget)]
pub struct Messages {
    #[deref]
    deref: View,

    /// The list of messages rendered by this widget.
    #[rust]
    pub messages: Vec<Message>,

    /// Bot repository to get bot information.
    #[rust]
    pub bot_repo: Option<BotRepo>,

    #[rust]
    current_editor: Option<Editor>,

    #[rust]
    is_list_end_drawn: bool,

    /// Keep track of the drawn items in the [[PortalList]] to be abale to retrive
    /// the visible items anytime.
    ///
    /// The method [[PortalList::visible_items]] just returns a count/length.
    #[rust]
    visible_range: Option<(usize, usize)>,

    #[rust]
    hovered_index: Option<usize>,
}

impl Widget for Messages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.handle_list(cx, event, scope);

        let jump_to_bottom = self.button(id!(jump_to_bottom));

        if jump_to_bottom.clicked(event.actions()) {
            self.scroll_to_bottom(cx);
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let list_uid = self.portal_list(id!(list)).widget_uid();

        while let Some(widget) = self.deref.draw_walk(cx, scope, walk).step() {
            if widget.widget_uid() == list_uid {
                self.draw_list(cx, widget.as_portal_list());
            }
        }

        DrawStep::done()
    }
}

impl Messages {
    fn draw_list(&mut self, cx: &mut Cx2d, list: PortalListRef) {
        self.is_list_end_drawn = false;
        self.visible_range = None;

        // Trick to render one more item representing the end of the chat without
        // risking a manual math bug. Removed immediately after rendering the items.
        self.messages.push(Message {
            from: EntityId::App,
            // End-of-chat marker
            content: MessageContent::PlainText {
                text: "EOC".into(),
                citations: Vec::new(),
            },
            is_writing: false,
        });

        let mut list = list.borrow_mut().unwrap();
        list.set_item_range(cx, 0, self.messages.len());

        while let Some(index) = list.next_visible_item(cx) {
            if index >= self.messages.len() {
                continue;
            }

            if let Some((_start, end)) = &mut self.visible_range {
                *end = (*end).max(index);
            } else {
                self.visible_range = Some((index, index));
            }

            let message = &self.messages[index];

            match &message.from {
                EntityId::System => {
                    // TODO: Can or should system messages be rendered?
                }
                EntityId::App => {
                    // Handle EOC marker
                    let body_text = message.visible_text();
                    if body_text == "EOC" {
                        let item = list.item(cx, index, live_id!(EndOfChat));
                        item.draw_all(cx, &mut Scope::empty());
                        self.is_list_end_drawn = true;
                        continue;
                    }

                    // Handle error messages
                    if let Some((left, right)) = body_text.split_once(':') {
                        if let Some("error") = left
                            .split_whitespace()
                            .last()
                            .map(|s| s.to_lowercase())
                            .as_deref()
                        {
                            let item = list.item(cx, index, live_id!(ErrorLine));
                            item.avatar(id!(avatar)).borrow_mut().unwrap().avatar =
                                Some(Picture::Grapheme("X".into()));
                            item.label(id!(name)).set_text(cx, left);
                            item.label(id!(text.markdown)).set_text(cx, right);
                            self.apply_actions_and_editor_visibility(cx, &item, index);
                            item.draw_all(cx, &mut Scope::empty());
                            continue;
                        }
                    }

                    // Handle regular app messages
                    let item = list.item(cx, index, live_id!(AppLine));
                    item.avatar(id!(avatar)).borrow_mut().unwrap().avatar =
                        Some(Picture::Grapheme("A".into()));
                    item.label(id!(text.markdown)).set_text(cx, &body_text);
                    self.apply_actions_and_editor_visibility(cx, &item, index);
                    item.draw_all(cx, &mut Scope::empty());
                }
                EntityId::User => {
                    let item = list.item(cx, index, live_id!(UserLine));
                    item.label(id!(text.label)).set_text(cx, &message.visible_text());
                    self.apply_actions_and_editor_visibility(cx, &item, index);
                    item.draw_all(cx, &mut Scope::empty());
                }
                EntityId::Bot(id) => {
                    let bot = self
                        .bot_repo
                        .as_ref()
                        .expect("no bot client set")
                        .get_bot(id);

                    let (name, avatar) = bot
                        .as_ref()
                        .map(|b| (b.name.as_str(), Some(b.avatar.clone())))
                        .unwrap_or(("Unknown bot", Some(Picture::Grapheme("B".into()))));

                    // If the message is empty and still writing, display a loading animation
                    let body_text = message.visible_text();
                    let item = if message.is_writing && body_text.is_empty() && !message.has_stages() {
                        let item = list.item(cx, index, live_id!(LoadingLine));
                        item.message_loading(id!(text.loading)).animate(cx);
                        item
                    } else if message.has_stages() {
                        // Use specialized DeepInquireBotLine for messages with stages
                        let item = list.item(cx, index, live_id!(DeepInquireBotLine));
                        
                        // Update the DeepInquireBotLine with the message content
                        item.as_deep_inquire_bot_line().set_message(cx, message, message.is_writing);
                        
                        item
                    } else {
                        let item = list.item(cx, index, live_id!(BotLine));
                        // Workaround: Because I had to set `paragraph_spacing` to 0 in `MessageMarkdown`,
                        // we need to add a "blank" line as a workaround.
                        //
                        // Warning: If you ever read the text from this widget and not
                        // from the list, you should remove the unicode character.
                        // TODO: Remove this workaround once the markdown widget is fixed.

                        let (thinking_block, message_body) =
                            extract_and_remove_think_tag(&body_text);

                        item.message_thinking_block(id!(text.thinking_block))
                            .set_thinking_text(thinking_block);

                        if let Some(body) = message_body {
                            item.label(id!(text.markdown))
                                .set_text(cx, &body.replace("\n\n", "\n\n\u{00A0}\n\n"));
                        }

                        // Set citations from the message
                        let citations = message.sources();
                        if !citations.is_empty() {
                            let mut citations_ref = item.citations(id!(citations));
                            citations_ref.set_citations(cx, &citations);
                        }
                        item
                    };

                    item.avatar(id!(avatar)).borrow_mut().unwrap().avatar = avatar;
                    item.label(id!(name)).set_text(cx, name);

                    self.apply_actions_and_editor_visibility(cx, &item, index);

                    item.draw_all(cx, &mut Scope::empty());
                }
            }
        }

        if let Some(message) = self.messages.pop() {
            assert!(message.from == EntityId::App);
            assert!(message.visible_text() == "EOC");
        }

        self.button(id!(jump_to_bottom))
            .set_visible(cx, !self.is_list_end_drawn);
    }

    /// Check if we're at the end of the messages list.
    pub fn is_at_bottom(&self) -> bool {
        self.is_list_end_drawn
    }

    /// Jump to the end of the list instantly.
    pub fn scroll_to_bottom(&self, _cx: &mut Cx) {
        if self.messages.len() > 0 {
            // This is not the last message, but the marker widget we added to
            // the list. I'm being explicit with the redundant -1/+1.

            let last_message_index = self.messages.len() - 1;
            let end_of_chat_index = last_message_index + 1;

            let list = self.portal_list(id!(list));

            // TODO: This works for scrolling to the end and works reliably even
            // while streaming, but the portal list event handling will bug unless
            // an scroll event ocurrs.
            list.set_first_id(end_of_chat_index);

            // list.smooth_scroll_to(cx, end_of_chat_index, 100., None);
        }
    }

    /// Show or hide the editor for a message.
    ///
    /// Limitation: Only one editor can be shown at a time. If you try to show another editor,
    /// the previous one will be hidden. If you try to hide an editor different from the one
    /// currently shown, nothing will happen.
    pub fn set_message_editor_visibility(&mut self, index: usize, visible: bool) {
        if index >= self.messages.len() {
            return;
        }

        if visible {
            let buffer = self.messages[index].visible_text();
            self.current_editor = Some(Editor { index, buffer });
        } else if self.current_editor.as_ref().map(|e| e.index) == Some(index) {
            self.current_editor = None;
        }
    }

    /// If currently editing a message, this will return the text in it's editor.
    pub fn current_editor_text(&self) -> Option<String> {
        self.current_editor
            .as_ref()
            .and_then(|editor| self.portal_list(id!(list)).get_item(editor.index))
            .map(|(_id, widget)| widget.text_input(id!(input)).text())
    }

    /// If currently editing a message, this will return the index of the message.
    pub fn current_editor_index(&self) -> Option<usize> {
        self.current_editor.as_ref().map(|e| e.index)
    }

    fn handle_list(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let Some(range) = self.visible_range else {
            return;
        };

        let list = self.portal_list(id!(list));
        let range = range.0..=range.1;

        for (index, item) in ItemsRangeIter::new(list, range) {
            if let Event::MouseMove(event) = event {
                if item.area().rect(cx).contains(event.abs) {
                    self.hovered_index = Some(index);
                    item.redraw(cx);
                }
            }

            let actions = event.actions();

            if item.button(id!(copy)).clicked(actions) {
                cx.widget_action(self.widget_uid(), &scope.path, MessagesAction::Copy(index));
            }

            if item.button(id!(delete)).clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    MessagesAction::Delete(index),
                );
            }

            if item.button(id!(edit)).clicked(actions) {
                self.set_message_editor_visibility(index, true);
                self.redraw(cx);
            }

            if item.button(id!(cancel)).clicked(actions) {
                self.set_message_editor_visibility(index, false);
                self.redraw(cx);
            }

            if item.button(id!(save)).clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    MessagesAction::EditSave(index),
                );
            }

            if item.button(id!(save_and_regenerate)).clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    MessagesAction::EditRegenerate(index),
                );
            }

            if let Some(change) = item.text_input(id!(input)).changed(actions) {
                self.current_editor.as_mut().unwrap().buffer = change;
            }
        }
    }

    fn apply_actions_and_editor_visibility(
        &mut self,
        cx: &mut Cx,
        widget: &WidgetRef,
        index: usize,
    ) {
        let editor = widget.view(id!(editor));
        let actions = widget.view(id!(actions));
        let edit_actions = widget.view(id!(edit_actions));
        let text = widget.view(id!(text));

        let is_hovered = self.hovered_index == Some(index);
        let is_current_editor = self.current_editor.as_ref().map(|e| e.index) == Some(index);

        edit_actions.set_visible(cx, is_current_editor);
        editor.set_visible(cx, is_current_editor);
        actions.set_visible(cx, !is_current_editor && is_hovered);
        text.set_visible(cx, !is_current_editor);

        if is_current_editor {
            editor
                .text_input(id!(input))
                .set_text(cx, &self.current_editor.as_ref().unwrap().buffer);
        }
    }
}

impl MessagesRef {
    /// Immutable access to the underlying [[Messages]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read(&self) -> Ref<Messages> {
        self.borrow().unwrap()
    }

    /// Mutable access to the underlying [[Messages]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write(&mut self) -> RefMut<Messages> {
        self.borrow_mut().unwrap()
    }

    /// Immutable reader to the underlying [[Messages]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn read_with<R>(&self, f: impl FnOnce(&Messages) -> R) -> R {
        f(&*self.read())
    }

    /// Mutable writer to the underlying [[Messages]].
    ///
    /// Panics if the widget reference is empty or if it's already borrowed.
    pub fn write_with<R>(&mut self, f: impl FnOnce(&mut Messages) -> R) -> R {
        f(&mut *self.write())
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
