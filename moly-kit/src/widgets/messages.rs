use std::{
    cell::{Ref, RefMut},
    collections::HashMap,
};

use crate::{
    protocol::*,
    utils::{events::EventExt, portal_list::ItemsRangeIter},
    widgets::{avatar::AvatarWidgetRefExt, message_loading::MessageLoadingWidgetRefExt},
};
use makepad_code_editor::code_view::CodeViewWidgetRefExt;
use makepad_widgets::*;

use super::{
    citation::CitationAction, slot::SlotWidgetRefExt,
    standard_message_content::StandardMessageContentWidgetRefExt,
};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::widgets::chat_lines::*;
    use crate::clients::deep_inquire::widgets::deep_inquire_content::*;

    pub Messages = {{Messages}} {
        flow: Overlay,

        deep_inquire_content: <DeepInquireContent> {}

        list = <PortalList> {
            scroll_bar: {
                bar_size: 0.0,
            }
            UserLine = <UserLine> {}
            BotLine = <BotLine> {}
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
                icon_walk: {
                    width: 12, height: 12
                    margin: {left: 4.5},
                }
                draw_icon: {
                    svg_file: dep("crate://self/resources/jump_to_bottom.svg")
                    color: #1C1B1F,
                    color_hover: #x0
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
#[derive(Live, Widget)]
pub struct Messages {
    #[deref]
    deref: View,

    /// The list of messages rendered by this widget.
    #[rust]
    pub messages: Vec<Message>,

    /// Bot repository to get bot information.
    #[rust]
    pub bot_repo: Option<BotRepo>,

    /// Registry of DSL templates used by custom content widgets.
    ///
    /// This is exposed as it is for easy manipulation and it's passed to
    /// [BotClient::content_widget] method allowing it to create widgets with
    /// `new_from_ptr(cx)`.
    #[rust]
    pub templates: HashMap<LiveId, LivePtr>,

    // TODO: Consider moving this out to it's own crate now that custom content
    // is supported.
    #[live]
    deep_inquire_content: LivePtr,

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

    /// Used to trigger a defered scroll to bottom after the message list has been replaced.
    #[rust]
    should_defer_scroll_to_bottom: bool,

    #[rust]
    hovered_index: Option<usize>,

    #[rust]
    user_scrolled: bool,

    #[rust]
    sticking_to_bottom: bool,
}

impl Widget for Messages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.handle_list(cx, event, scope);

        let jump_to_bottom = self.button(id!(jump_to_bottom));

        if jump_to_bottom.clicked(event.actions()) {
            self.scroll_to_bottom(cx, false);
            // Reset the scrolling state, so that if the user clicks the button during a stream,
            // we forget they scrolled, and assume they want to stick to the bottom.
            self.user_scrolled = false;
            self.sticking_to_bottom = false;
            self.redraw(cx);
        }

        for action in event.widget_actions() {
            if let CitationAction::Open(url) = action.cast() {
                let _ = robius_open::Uri::new(url.as_str()).open();
            }
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
    fn draw_list(&mut self, cx: &mut Cx2d, list_ref: PortalListRef) {
        self.is_list_end_drawn = false;
        self.visible_range = None;

        // Trick to render one more item representing the end of the chat without
        // risking a manual math bug. Removed immediately after rendering the items.
        self.messages.push(Message {
            from: EntityId::App,
            // End-of-chat marker
            content: MessageContent {
                text: "EOC".into(),
                ..Default::default()
            },
            is_writing: false,
        });

        if self.should_defer_scroll_to_bottom {
            // Note: Not using `smooth_scroll_to_end` because it makes asumptions about the list range and the items
            // that are only true after we've updated the list through itreation on next_visible_item.
            list_ref.set_first_id(self.messages.len().saturating_sub(1));
            self.should_defer_scroll_to_bottom = false;
        }

        let mut list = list_ref.borrow_mut().unwrap();
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
                    if message.content.text == "EOC" {
                        let item = list.item(cx, index, live_id!(EndOfChat));
                        item.draw_all(cx, &mut Scope::empty());
                        self.is_list_end_drawn = true;
                        continue;
                    }

                    // Handle error messages
                    if let Some((left, right)) = message.content.text.split_once(':') {
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
                    item.label(id!(text.markdown))
                        .set_text(cx, &message.content.text);
                    self.apply_actions_and_editor_visibility(cx, &item, index);
                    item.draw_all(cx, &mut Scope::empty());
                }
                EntityId::User => {
                    let item = list.item(cx, index, live_id!(UserLine));

                    item.avatar(id!(avatar)).borrow_mut().unwrap().avatar =
                        Some(Picture::Grapheme("Y".into()));
                    item.label(id!(name)).set_text(cx, "You");

                    item.slot(id!(content))
                        .current()
                        .as_standard_message_content()
                        .set_content(cx, &message.content);

                    self.apply_actions_and_editor_visibility(cx, &item, index);
                    item.draw_all(cx, &mut Scope::empty());
                }
                EntityId::Bot(id) => {
                    let repo = self.bot_repo.as_ref().expect("no bot client set");
                    let bot = repo.get_bot(id);

                    let (name, avatar) = bot
                        .as_ref()
                        .map(|b| (b.name.as_str(), Some(b.avatar.clone())))
                        .unwrap_or(("Unknown bot", Some(Picture::Grapheme("B".into()))));

                    let item = if message.is_writing && message.content.is_empty() {
                        let item = list.item(cx, index, live_id!(LoadingLine));
                        item.message_loading(id!(content_section.loading))
                            .animate(cx);
                        item
                    } else {
                        list.item(cx, index, live_id!(BotLine))
                    };

                    item.avatar(id!(avatar)).borrow_mut().unwrap().avatar = avatar;
                    item.label(id!(name)).set_text(cx, name);

                    let mut slot = item.slot(id!(content));
                    if let Some(custom_content) = repo.client().content_widget(
                        cx,
                        slot.current().clone(),
                        &self.templates,
                        &message.content,
                    ) {
                        slot.replace(custom_content);
                    } else {
                        // Since portal list may reuse widgets, we must restore
                        // the default widget just in case.
                        slot.restore();
                        slot.default()
                            .as_standard_message_content()
                            .set_content(cx, &message.content);
                    }

                    self.apply_actions_and_editor_visibility(cx, &item, index);
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
        }

        if let Some(message) = self.messages.pop() {
            assert!(message.from == EntityId::App);
            assert!(message.content.text == "EOC");
        }

        self.button(id!(jump_to_bottom))
            .set_visible(cx, !self.is_at_bottom() && !self.sticking_to_bottom);
    }

    /// Check if we're at the end of the messages list.
    pub fn is_at_bottom(&self) -> bool {
        self.is_list_end_drawn
    }

    pub fn user_scrolled(&self) -> bool {
        self.user_scrolled
    }

    /// Jump to the end of the list instantly.
    pub fn scroll_to_bottom(&mut self, cx: &mut Cx, triggered_by_stream: bool) {
        if self.messages.len() > 0 {
            let list = self.portal_list(id!(list));

            list.smooth_scroll_to_end(cx, 100.0, None);
            self.sticking_to_bottom = triggered_by_stream;
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
            let buffer = self.messages[index].content.text.clone();
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

        // Handle item actions
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

        // Handle code copy
        // Since the Markdown widget could have multiple code blocks, we need the widget that triggered the action
        if let Some(wa) = event.actions().widget_action(id!(copy_code_button)) {
            if wa.widget().as_button().pressed(event.actions()) {
                // nth(2) refers to the code view in the MessageMarkdown widget
                let code_view = wa.widget_nth(2).widget(id!(code_view));
                let text_to_copy = code_view.as_code_view().text();
                cx.copy_to_clipboard(&text_to_copy);
            }
        }

        // Detect if the user has manually scrolled the list.
        // Ideally we should use `PortalList::was_scrolled` or `PortalList::scrolled` but they aren't reliable.
        match event.hits(cx, self.area()) {
            Hit::FingerScroll(_e) => {
                self.user_scrolled = true;
                self.sticking_to_bottom = false;
            }
            _ => {}
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
        let content_section = widget.view(id!(content_section));

        let is_hovered = self.hovered_index == Some(index);
        let is_current_editor = self.current_editor.as_ref().map(|e| e.index) == Some(index);

        edit_actions.set_visible(cx, is_current_editor);
        editor.set_visible(cx, is_current_editor);
        actions.set_visible(cx, !is_current_editor && is_hovered);
        content_section.set_visible(cx, !is_current_editor);

        if is_current_editor {
            editor
                .text_input(id!(input))
                .set_text(cx, self.current_editor.as_ref().unwrap().buffer.clone());
        }
    }

    /// Set the messages and defer a scroll to bottom if requested.
    pub fn set_messages(&mut self, messages: Vec<Message>, scroll_to_bottom: bool) {
        self.messages = messages;
        self.should_defer_scroll_to_bottom = scroll_to_bottom;
    }

    pub fn reset_scroll_state(&mut self) {
        self.user_scrolled = false;
        self.sticking_to_bottom = false;
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

impl LiveHook for Messages {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.templates
            .insert(live_id!(DeepInquireContent), self.deep_inquire_content);
    }
}
