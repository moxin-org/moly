use std::cell::{Ref, RefMut};

use crate::{avatar::AvatarWidgetRefExt, message_loading::MessageLoadingWidgetRefExt, protocol::*};
use makepad_widgets::*;

// use crate::chat::shared::ChatAgentAvatarWidgetRefExt;

live_design! {
    // import makepad_widgets::base::*;
    // import makepad_widgets::theme_desktop_dark::*;
    // import crate::shared::styles::*;
    // import crate::chat::chat_line_loading::ChatLineLoading;
    // import crate::chat::shared::ChatAgentAvatar;
    // import crate::battle::agent_markdown::AgentMarkdown;

    use link::theme::*;
    use link::widgets::*;

    use crate::message_markdown::*;
    use crate::message_loading::*;
    use crate::avatar::*;

    Sender = <View> {
        height: Fit,
        spacing: 8,
        align: {y: 0.5}
        avatar = <Avatar> {}
        name = <Label> {
            draw_text:{
                // text_style: <BOLD_FONT>{font_size: 10},
                color: #000
            }
        }
    }


    Bubble = <RoundedView> {
        height: Fit,
        padding: {left: 16, right: 18, top: 18, bottom: 14},
        margin: {bottom: 16},
        show_bg: true,
        draw_bg: {
            radius: 12.0,
        }
    }

    Actions = <View> {
        height: Fit,
        copy = <Button> { text: "copy", draw_text: {color: #000} }
        edit = <Button> { text: "edit", draw_text: {color: #000} }
        delete = <Button> { text: "delete", draw_text: {color: #000} }
    }

    EditActions = <View> {
        height: Fit,
        save = <Button> { text: "save", draw_text: {color: #000} }
        save_and_regenerate = <Button> { text: "save and regenerate", draw_text: {color: #000} }
        cancel = <Button> { text: "cancel", draw_text: {color: #000} }
    }

    Editor = <View> {
        height: Fit,
        input = <TextInput> {
            draw_text: {
                color: #000
            }
        }
    }

    ChatLine = <View> {
        flow: Down,
        height: Fit,
        sender = <Sender> {}
        bubble = <Bubble> {}
        actions = <Actions> {}
        edit_actions = <EditActions> { visible: false }
    }

    UserLine = <ChatLine> {
        height: Fit,
        sender = { visible: false }
        bubble = <Bubble> {
            margin: {left: 100}
            draw_bg: {color: #15859A}
            text = <View> {
                height: Fit
                label = <Label> {
                    width: Fill,
                    draw_text: {
                        // text_style: <REGULAR_FONT>{height_factor: (1.3*1.3), font_size: 10},
                        color: #fff
                    }
                }
            }
            editor = <Editor> { visible: false }
        }
    }

    BotLine = <ChatLine> {
        flow: Down,
        height: Fit,
        bubble = <Bubble> {
            margin: {left: 16}
            text = <View> {
                height: Fit,
                markdown = <MessageMarkdown> {}
            }
            editor = <Editor> { visible: false }
        }
    }

    LoadingLine = <BotLine> {
        bubble = {
            text = <View> {
                height: Fit,
                loading = <MessageLoading> {}
            }
        }
    }

    pub Messages = {{Messages}} {
        flow: Overlay,
        list = <PortalList> {
            scroll_bar: {
                bar_size: 0.0,
            }
            UserLine = <UserLine> {}
            BotLine = <BotLine> {}
            LoadingLine = <LoadingLine> {}

            // Acts as marker for:
            // - Knowing if the end of the list has been reached.
            // - To jump to bottom with proper precision.
            EndOfChat = <View> {height: 0.1}
        }
        <View> {
            align: {x: 1.0, y: 1.0},
            jump_to_bottom = <Button> {
                text: "v"
                draw_text: { color: #000 }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct MessageActionData {
    index: usize,
    kind: MessageActionKind,
}

/// Glue to use as `action_data` for the messages list.
#[derive(Debug, PartialEq)]
enum MessageActionKind {
    Copy,
    Edit,
    Delete,
    EditRegenerate,
    EditSave,
    EditCancel,
}

/// Relevant actions that should be handled by a parent.
#[derive(Debug, PartialEq, Copy, Clone, DefaultNone)]
pub enum MessagesAction {
    Copy(usize),
    Delete(usize),
    EditRegenerate(usize),
    EditSave(usize),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct Messages {
    #[deref]
    view: View,

    #[rust]
    pub messages: Vec<Message>,

    #[rust]
    pub bot_repo: Option<BotRepo>,

    #[rust]
    current_editor: Option<usize>,

    #[rust]
    is_list_end_drawn: bool,
}

impl Widget for Messages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let Event::Actions(actions) = event else {
            return;
        };

        let jump_to_bottom = self.button(id!(jump_to_bottom));

        if jump_to_bottom.clicked(actions) {
            self.scroll_to_bottom();
            self.redraw(cx);
        }

        let list_uid = self.portal_list(id!(list)).widget_uid();
        for action in actions {
            let Some(action) = action.as_widget_action() else {
                continue;
            };

            let Some(group) = &action.group else {
                continue;
            };

            if group.group_uid != list_uid {
                continue;
            }

            let Some(data) = &action.data else {
                continue;
            };

            let Some(data) = data.downcast_ref::<MessageActionData>() else {
                continue;
            };

            if let ButtonAction::Clicked(_) = action.cast::<ButtonAction>() {
                log!("{:?}", &data);
                let action = match data.kind {
                    MessageActionKind::Copy => MessagesAction::Copy(data.index),
                    MessageActionKind::Delete => MessagesAction::Delete(data.index),
                    MessageActionKind::EditRegenerate => MessagesAction::EditRegenerate(data.index),
                    MessageActionKind::EditSave => MessagesAction::EditSave(data.index),
                    MessageActionKind::Edit => {
                        self.set_message_editor_visibility(data.index, true);
                        self.redraw(cx);
                        MessagesAction::None
                    }
                    MessageActionKind::EditCancel => {
                        self.set_message_editor_visibility(data.index, false);
                        self.redraw(cx);
                        MessagesAction::None
                    }
                };

                cx.widget_action(self.widget_uid(), &scope.path, action);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let list_uid = self.portal_list(id!(list)).widget_uid();

        while let Some(widget) = self.view.draw_walk(cx, scope, walk).step() {
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

        // Trick to render one more item representing the end of the chat without
        // risking a manual math bug. Removed immediately after rendering the items.
        self.messages.push(Message {
            from: EntityId::App,
            // End-of-chat
            body: "EOC".into(),
            is_writing: false,
        });

        let mut list = list.borrow_mut().unwrap();
        list.set_item_range(cx, 0, self.messages.len());

        while let Some(index) = list.next_visible_item(cx) {
            if index >= self.messages.len() {
                continue;
            }

            let message = &self.messages[index];

            match &message.from {
                EntityId::System => {
                    // TODO: Can or should system messages be rendered?
                }
                EntityId::App => {
                    // TODO: Display app messages. They may be errors.
                    if message.body == "EOC" {
                        let item = list.item(cx, index, live_id!(EndOfChat));
                        item.draw_all(cx, &mut Scope::empty());
                        self.is_list_end_drawn = true;
                    } else {
                        todo!();
                    }
                }
                EntityId::User => {
                    let item = list.item(cx, index, live_id!(UserLine));
                    item.label(id!(text.label)).set_text(cx, &message.body);

                    connect_action_data(index, &item);
                    apply_actions_and_editor_visibility(cx, &item, index, self.current_editor);

                    item.draw_all(cx, &mut Scope::empty());
                }
                EntityId::Bot(id) => {
                    let bot = self
                        .bot_repo
                        .as_ref()
                        .expect("no bot client set")
                        .get_bot(id);

                    let name = bot
                        .as_ref()
                        .map(|b| b.name.as_str())
                        .unwrap_or("Unknown bot");
                    let avatar = bot.as_ref().map(|b| b.avatar.clone());

                    let item = if message.is_writing && message.body.is_empty() {
                        let item = list.item(cx, index, live_id!(LoadingLine));

                        item.message_loading(id!(text.loading)).animate(cx);

                        item
                    } else {
                        let item = list.item(cx, index, live_id!(BotLine));
                        // Workaround: Because I had to set `paragraph_spacing` to 0 in `MessageMarkdown`,
                        // we need to add a "blank" line as a workaround.
                        //
                        // Warning: If you ever read the text from this widget and not
                        // from the list, you should remove the unicode character.
                        item.label(id!(text.markdown))
                            .set_text(cx, &message.body.replace("\n\n", "\n\n\u{00A0}\n\n"));

                        item
                    };

                    item.avatar(id!(avatar)).borrow_mut().unwrap().avatar = avatar;
                    item.label(id!(name)).set_text(cx, name);

                    connect_action_data(index, &item);
                    apply_actions_and_editor_visibility(cx, &item, index, self.current_editor);

                    item.draw_all(cx, &mut Scope::empty());
                }
            }
        }

        if let Some(message) = self.messages.pop() {
            assert!(message.from == EntityId::App);
            assert!(message.body == "EOC");
        }

        // dbg!(list_end_reached);
        // dbg!(list.is_at_end());

        self.button(id!(jump_to_bottom))
            .set_visible(cx, !self.is_list_end_drawn);
    }

    /// Check if we're at the end of the messages list.
    pub fn is_at_bottom(&self) -> bool {
        self.is_list_end_drawn
    }

    /// Jump to the end of the list instantly.
    pub fn scroll_to_bottom(&self) {
        if self.messages.len() > 0 {
            // This is not the last message, but the marker widget we added to
            // the list. I'm being explicit with the redundant -1/+1.

            let last_message_index = self.messages.len() - 1;
            let end_of_chat_index = last_message_index + 1;
            self.portal_list(id!(list)).set_first_id(end_of_chat_index);
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
            self.current_editor = Some(index);
        } else if self.current_editor == Some(index) {
            self.current_editor = None;
        }
    }

    /// If currently editing a message, this will return the text in it's editor.
    pub fn current_editor_text(&self) -> Option<String> {
        self.current_editor
            .and_then(|index| self.portal_list(id!(list)).get_item(index))
            .map(|(_id, widget)| widget.text_input(id!(input)).text())
    }
}

impl MessagesRef {
    pub fn read(&self) -> Ref<Messages> {
        self.borrow().unwrap()
    }

    pub fn write(&mut self) -> RefMut<Messages> {
        self.borrow_mut().unwrap()
    }
}

fn connect_action_data(index: usize, widget: &WidgetRef) {
    [
        (id!(copy), MessageActionKind::Copy),
        (id!(delete), MessageActionKind::Delete),
        (id!(regenerate), MessageActionKind::EditRegenerate),
        (id!(save), MessageActionKind::EditSave),
        (id!(edit), MessageActionKind::Edit),
        (id!(cancel), MessageActionKind::EditCancel),
    ]
    .into_iter()
    .for_each(|(id, kind)| {
        widget
            .button(id)
            .set_action_data(MessageActionData { index, kind });
    });
}

fn apply_actions_and_editor_visibility(
    cx: &mut Cx,
    widget: &WidgetRef,
    index: usize,
    current_editor: Option<usize>,
) {
    let editor = widget.view(id!(editor));
    let actions = widget.view(id!(actions));
    let edit_actions = widget.view(id!(edit_actions));
    let text = widget.view(id!(text));

    let is_current_editor = current_editor == Some(index);

    edit_actions.set_visible(cx, is_current_editor);
    editor.set_visible(cx, is_current_editor);
    actions.set_visible(cx, !is_current_editor);
    text.set_visible(cx, !is_current_editor);
}
