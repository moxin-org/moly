use makepad_widgets::*;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use std::collections::HashSet;

use crate::data::chats::chat::ChatID;
use crate::data::store::{ProviderSyncingStatus, Store};
use crate::shared::utils::attachments::{
    delete_attachment, generate_persistence_key, set_persistence_key_and_reader,
    write_attachment_to_key,
};

use super::model_selector::ModelSelectorWidgetExt;
use super::model_selector_item::ModelSelectorAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::chat_panel::ChatPanel;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chat_params::ChatParams;
    use crate::chat::model_selector::ModelSelector;
    use moly_kit::widgets::chat::Chat;
    use moly_kit::widgets::prompt_input::PromptInput;

    PromptInputWithShadow = <PromptInput> {
        padding: {left: 10, right: 10, top: 8, bottom: 8}
        persistent = {
            // Shader to make the original RoundedView into a RoundedShadowView
            // (can't simply override the type of `persistent` because that removes the original children)
            clip_x:false, clip_y:false,

            show_bg: true,
            draw_bg: {
                color: #fefefe
                uniform border_radius: 5.0
                uniform border_size: 0.0
                uniform border_color: #0000
                uniform shadow_color: #0001
                uniform shadow_radius: 9.0,
                uniform shadow_offset: vec2(0.0,-2.5)

                varying rect_size2: vec2,
                varying rect_size3: vec2,
                varying rect_pos2: vec2,
                varying rect_shift: vec2,
                varying sdf_rect_pos: vec2,
                varying sdf_rect_size: vec2,

                fn get_color(self) -> vec4 {
                    return self.color
                }

                fn vertex(self) -> vec4 {
                    let min_offset = min(self.shadow_offset,vec2(0));
                    self.rect_size2 = self.rect_size + 2.0*vec2(self.shadow_radius);
                    self.rect_size3 = self.rect_size2 + abs(self.shadow_offset);
                    self.rect_pos2 = self.rect_pos - vec2(self.shadow_radius) + min_offset;
                    self.sdf_rect_size = self.rect_size2 - vec2(self.shadow_radius * 2.0 + self.border_size * 2.0)
                    self.sdf_rect_pos = -min_offset + vec2(self.border_size + self.shadow_radius);
                    self.rect_shift = -min_offset;

                    return self.clip_and_transform_vertex(self.rect_pos2, self.rect_size3)
                }

                fn get_border_color(self) -> vec4 {
                    return self.border_color
                }

                fn pixel(self) -> vec4 {

                    let sdf = Sdf2d::viewport(self.pos * self.rect_size3)
                    sdf.box(
                        self.sdf_rect_pos.x,
                        self.sdf_rect_pos.y,
                        self.sdf_rect_size.x,
                        self.sdf_rect_size.y,
                        max(1.0, self.border_radius)
                    )
                    if sdf.shape > -1.0{
                        let m = self.shadow_radius;
                        let o = self.shadow_offset + self.rect_shift;
                        let v = GaussShadow::rounded_box_shadow(vec2(m) + o, self.rect_size2+o, self.pos * (self.rect_size3+vec2(m)), self.shadow_radius*0.5, self.border_radius*2.0);
                        sdf.clear(self.shadow_color*v)
                    }

                    sdf.fill_keep(self.get_color())
                    if self.border_size > 0.0 {
                        sdf.stroke(self.get_border_color(), self.border_size)
                    }
                    return sdf.result
                }
            }
        }
    }

    pub ChatView = {{ChatView}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 0

        model_selector = <ModelSelector> {}
        chat = <Chat> {
            messages = { padding: {left: 10, right: 10} }
            prompt = <PromptInputWithShadow> {}
        }
    }
}

/// A self-contained chat view that wraps MolyKit's Chat widget
/// adding a model selector.
///
/// This allows ChatScreen to use multiple concurrent chats.
#[derive(Live, Widget)]
pub struct ChatView {
    #[deref]
    view: View,

    #[rust]
    chat_id: ChatID,

    #[rust]
    focused: bool,

    #[rust]
    message_updated_while_inactive: bool,

    #[rust]
    persisting_attachments: HashSet<Attachment>,
}

impl LiveHook for ChatView {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.prompt_input(id!(chat.prompt)).write().disable();

        // Glue between Moly and Moly Kit.
        let ui = self.ui_runner();
        self.chat(id!(chat))
            .write()
            .set_hook_after(move |group, _, _| {
                for task in group.iter() {
                    // Currently, we only need to process one task at a time in
                    // the context of our widget.
                    let task = task.clone();
                    ui.defer_with_redraw(move |me, cx, scope| {
                        me.handle_chat_task(task, cx, scope);
                    });
                }
            });
    }
}

impl Widget for ChatView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.widget_match_event(cx, event, scope);
        self.view.handle_event(cx, event, scope);

        self.handle_current_bot(cx, scope);
        self.handle_unread_messages(scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // On mobile, only set padding on top of the prompt
        // TODO: do this with AdaptiveView instead of apply_over
        if !cx.display_context.is_desktop() && cx.display_context.is_screen_size_known() {
            self.prompt_input(id!(chat.prompt)).apply_over(
                cx,
                live! {
                    padding: {bottom: 50, left: 20, right: 20}
                    persistent = {
                        height: 80
                    }
                },
            );
            self.model_selector(id!(model_selector)).apply_over(
                cx,
                live! {
                    width: Fill
                    button = { width: Fill }
                },
            );
        } else {
            self.prompt_input(id!(chat.prompt)).apply_over(
                cx,
                live! {
                    padding: {left: 10, right: 10, top: 8, bottom: 8}
                    persistent = {
                        height: Fit
                    }
                },
            );
            self.model_selector(id!(model_selector)).apply_over(
                cx,
                live! {
                    width: Fit
                    button = { width: Fit }
                },
            );
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let mut chat_widget = self.chat(id!(chat));

        for action in actions {
            // Handle model selector actions
            match action.cast() {
                ModelSelectorAction::BotSelected(chat_id, bot) => {
                    if chat_id == self.chat_id {
                        chat_widget.write().set_bot_id(cx, Some(bot.id.clone()));

                        if let Some(chat) = store.chats.get_chat_by_id(chat_id) {
                            chat.borrow_mut().associated_bot = Some(bot.id.clone());
                            chat.borrow().save_and_forget();
                        }
                        // self.focus_on_prompt_input_pending = true;
                    }
                }
                _ => {}
            }
        }
    }
}

impl ChatView {
    // TODO: Only perform this checks after certain actions like provider sync or provider updates (e.g. disable/enable provider)
    // Refactor this to be simpler and more unified with the behavior of the model selector
    fn handle_current_bot(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        // Check if the current chat's associated bot is still available
        let mut bot_available = false;
        let mut associiated_bot_id = None;
        if let Some(chat) = store.chats.get_chat_by_id(self.chat_id) {
            if let Some(bot_id) = &chat.borrow().associated_bot {
                associiated_bot_id = Some(bot_id.clone());
                bot_available = store
                    .chats
                    .get_all_bots(true)
                    .iter()
                    .any(|bot| &bot.id == bot_id)
            }
        }

        // If the bot is not available and we know it won't be available soon, clear the bot_id in the chat widget
        if !bot_available && store.provider_syncing_status == ProviderSyncingStatus::Synced {
            self.chat(id!(chat)).write().set_bot_id(cx, None);

            self.model_selector(id!(model_selector))
                .set_currently_selected_model(cx, None);
        } else if bot_available && self.chat(id!(chat)).read().bot_id().is_none() {
            // If the bot is available and the chat widget doesn't have a bot_id, set the bot_id in the chat widget
            // This can happen if the bot or provider was re-enabled after being disabled while being selected
            self.chat(id!(chat))
                .write()
                .set_bot_id(cx, associiated_bot_id);
        }

        // If there is no selected bot, disable the prompt input
        if self.chat(id!(chat)).read().bot_id().is_none()
            || !bot_available
            || store.provider_syncing_status != ProviderSyncingStatus::Synced
        {
            self.prompt_input(id!(chat.prompt)).write().disable();
        } else {
            self.prompt_input(id!(chat.prompt)).write().enable();
        }
    }

    fn handle_unread_messages(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        if self.message_updated_while_inactive {
            // If the message is done writing, and this chat view is not focused
            // set the chat as having unread messages (show a badge on the chat history card)
            if !self.chat(id!(chat)).read().is_streaming() && !self.focused {
                if let Some(chat) = store.chats.get_chat_by_id(self.chat_id) {
                    chat.borrow_mut().has_unread_messages = true;
                    self.message_updated_while_inactive = false;
                }
            }
        }
    }

    fn handle_chat_task(&mut self, task: ChatTask, _cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        // Let's get the chat in the store we will apply the modifications to.
        let Some(store_chat) = store.chats.get_chat_by_id(self.chat_id) else {
            return;
        };

        let mut store_chat = store_chat.borrow_mut();

        // Transform task operations into a single `splice` operation that can
        // be applied to the store chat messages and is easier to analyze for
        // attachment persistence.
        let (range, replacement) = match task {
            ChatTask::InsertMessage(index, message) => (index..index, vec![message]),
            ChatTask::UpdateMessage(index, message) => {
                if index == store_chat.messages.len() {
                    (index..index, vec![message])
                } else {
                    (index..(index + 1), vec![message])
                }
            }
            ChatTask::DeleteMessage(index) => (index..(index + 1), vec![]),
            ChatTask::SetMessages(messages) => (0..store_chat.messages.len(), messages),
            _ => (0..0, vec![]),
        };

        self.handle_attachments_persistence(&store_chat, &range, &replacement);
        self.handle_store_sync(&mut store_chat, &range, &replacement);
    }

    /// Handle the currently complex persistence of attachments logic.
    fn handle_attachments_persistence(
        &mut self,
        store_chat: &crate::data::chats::chat::Chat,
        range: &std::ops::Range<usize>,
        replacement: &[Message],
    ) {
        let store_len = store_chat.messages.len();
        let slice_start = range.start;
        let slice_end = std::cmp::min(range.end, store_len);

        // If the range is invalid, skip attachment processing to avoid panics
        if slice_start > slice_end {
            ::log::error!(
                "Invalid slice range at handle_attachments_persistence: {:?}",
                range
            );
            return;
        }

        // Track the attachments that are in the replacement part of the splice to
        // help with next steps.
        let attachments_in_replacement = replacement
            .iter()
            .flat_map(|m| &m.content.attachments)
            .cloned()
            .collect::<HashSet<_>>();

        // Make a list of previously PERSISTED attachments that would be lost after
        // applying the splice.
        // Focus on the range that will be replaced.
        let attachments_to_delete = store_chat.messages[slice_start..slice_end]
            .iter()
            .flat_map(|m| &m.content.attachments)
            // Focus on attachments already on disk.
            .filter(|a| a.has_persistence_key())
            // Only relevant if they really disappear from the replacement.
            .filter(|a| !attachments_in_replacement.contains(a))
            .cloned()
            .collect::<Vec<_>>();

        // Make a list of NOT PERSISTED attachments that will be applied after
        // after the splice.
        let attachments_to_persist = attachments_in_replacement
            .iter()
            // Focus on non-persisted attachments only.
            .filter(|a| !a.has_persistence_key())
            // Avoid attachments already being processed.
            .filter(|a| !self.persisting_attachments.contains(a))
            .cloned()
            .collect::<Vec<_>>();

        // Start deleting each removed attachment.
        for attachment in attachments_to_delete {
            // NOTE: This works for Moly use cases.
            // On reusable implementation, consider an attachment being reused across
            // messages and chats.
            spawn(async move {
                let key = attachment.get_persistence_key().unwrap();

                ::log::info!(
                    "Deleting persisted attachment, named {}, with key: {}",
                    attachment.name,
                    key
                );

                if let Err(e) = delete_attachment(&attachment).await {
                    ::log::error!(
                        "Failed to delete persisted attachment, named {}, with key {}: {}",
                        attachment.name,
                        key,
                        e
                    );
                }
            });
        }

        // Start persisting each new attachment.
        for attachment in attachments_to_persist {
            let ui = self.ui_runner();

            // Mark the attachment as being processed to avoid re-processing it.
            self.persisting_attachments.insert(attachment.clone());

            spawn(async move {
                let key = generate_persistence_key(&attachment);

                ::log::info!(
                    "Persisting attachment, named {}, with key: {}",
                    attachment.name,
                    key
                );

                if let Err(e) = write_attachment_to_key(&attachment, &key).await {
                    // log name and key
                    ::log::error!(
                        "Failed to persist (read & write) attachment, named {}, with key {}: {}",
                        attachment.name,
                        key,
                        e
                    );

                    // Note: Early return on failure will leave the attachment in the
                    // processing set to avoid re-processing it in the future.
                    return;
                }

                // Let's update the attachments back with the persisted key and reader.
                ui.defer_with_redraw(move |me, _cx, scope| {
                    let chat = me.chat(id!(chat));
                    let store = scope.data.get_mut::<Store>().unwrap();
                    let store_chat = store.chats.get_chat_by_id(me.chat_id).unwrap();

                    // Let's remove the attachment from the processing set before changing it.
                    me.persisting_attachments.remove(&attachment);

                    // Apply attachment changes to messages in the chat widget.
                    let _ = update_attachments_with_persisted_key(
                        &mut chat.read().messages_ref().write().messages,
                        &attachment,
                        key.clone(),
                    );

                    // Apply attachment changes to messages in the store chat.
                    let found = update_attachments_with_persisted_key(
                        &mut store_chat.borrow_mut().messages,
                        &attachment,
                        key.clone(),
                    );

                    // If while persisting, the attachment disappeared from the
                    // chat messages history, then delete it back from disk.
                    if !found {
                        ::log::info!(
                            "Attachment with name {} and key {} disappeared after persistence. Removing it.",
                            attachment.name,
                            key
                        );

                        spawn(async move {
                            if let Err(e) = delete_attachment(&attachment).await {
                                ::log::error!(
                                    "Failed to delete attachment that disappeared after persistence, named {}, with key {}: {}",
                                    attachment.name,
                                    key,
                                    e
                                );
                            }
                        });
                    }
                });
            });
        }
    }

    fn handle_store_sync(
        &mut self,
        store_chat: &mut crate::data::chats::chat::Chat,
        range: &std::ops::Range<usize>,
        replacement: &[Message],
    ) {
        // Apply the splice to the store chat.
        store_chat
            .messages
            .splice(range.clone(), replacement.iter().cloned());

        // Update the title if the first message changed (due to insert or update).
        if range.start == 0 && !replacement.is_empty() {
            store_chat.update_title_based_on_first_message();
        }

        // Write to disk.
        store_chat.save_and_forget();

        // Keep track of whether the message was updated while the chat view was inactive
        if !self.focused {
            self.message_updated_while_inactive = true;
        }
    }
}

impl ChatViewRef {
    pub fn set_chat_id(&mut self, chat_id: ChatID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.chat_id = chat_id;
            inner
                .model_selector(id!(model_selector))
                .set_chat_id(chat_id);
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.focused = focused;
        }
    }
}

#[must_use]
fn update_attachments_with_persisted_key<'m>(
    messages: impl IntoIterator<Item = &'m mut Message>,
    attachment: &Attachment,
    persisted_key: String,
) -> bool {
    let mut found = false;

    for message in messages {
        for att in message.content.attachments.iter_mut() {
            if att == attachment {
                set_persistence_key_and_reader(att, persisted_key.clone());
                found = true;
            }
        }
    }

    found
}

pub struct Plugin {
    ui: UiRunner<ChatView>,
}

impl