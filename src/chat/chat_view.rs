use makepad_widgets::*;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use std::collections::HashSet;
use std::path::Path;

use crate::data::chats::chat::ChatID;
use crate::data::store::{ProviderSyncingStatus, Store};
use crate::shared::utils::filesystem;

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

        // Hook into message updates to update the persisted chat history
        let ui = self.ui_runner();
        self.chat(id!(chat))
            .write()
            .set_hook_after(move |group, _, _| {
                for task in group.iter() {
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
                    padding: {top: 8, left: 0, right: 0, bottom: 0}
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

    fn handle_chat_task(&mut self, task: ChatTask, cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let Some(store_chat) = store.chats.get_chat_by_id(self.chat_id) else {
            return;
        };

        let mut store_chat = store_chat.borrow_mut();

        // Simplify by translating message mutations into a splice operation.
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

        let chat = self.chat(id!(chat));

        let attachments_in_replacement = replacement
            .iter()
            .flat_map(|m| &m.content.attachments)
            .cloned()
            .collect::<HashSet<_>>();

        let attachments_to_delete = store_chat.messages[range.clone()]
            .iter()
            .flat_map(|m| &m.content.attachments)
            .filter(|a| a.has_persisted_key())
            .filter(|a| !attachments_in_replacement.contains(a))
            .cloned()
            .collect::<Vec<_>>();

        let attachments_to_persist = attachments_in_replacement
            .iter()
            .filter(|a| !a.has_persisted_key())
            .filter(|a| !self.persisting_attachments.contains(a))
            .cloned()
            .collect::<Vec<_>>();

        for attachment in attachments_to_delete {
            // TODO: Consider re-used attachments before deleting.
            spawn(async move {
                let key = attachment.get_persisted_key().unwrap();
                let path = Path::new(key);
                filesystem::global().remove(path).await;
            });
        }

        for attachment in attachments_to_persist {
            let ui = self.ui_runner();
            self.persisting_attachments.insert(attachment.clone());
            spawn(async move {
                // TODO: Not unique enough.
                let key = format!("attachments/{}", attachment.name);
                let path = Path::new(&key).to_path_buf();

                let Ok(content) = attachment.read().await else {
                    return;
                };

                let Ok(()) = filesystem::global()
                    .queue_write(path, content.to_vec())
                    .await
                else {
                    return;
                };

                // Note: Early returns on failure will leave the attachment in the
                // processing set to avoid re-processing it in the future.

                ui.defer_with_redraw(move |me, _cx, scope| {
                    let chat = me.chat(id!(chat));
                    let store = scope.data.get_mut::<Store>().unwrap();
                    let store_chat = store.chats.get_chat_by_id(me.chat_id).unwrap();

                    update_attachments_with_persisted_key(
                        &mut chat.read().messages_ref().write().messages,
                        &attachment,
                        key.clone(),
                    );

                    update_attachments_with_persisted_key(
                        &mut store_chat.borrow_mut().messages,
                        &attachment,
                        key.clone(),
                    );

                    // TODO: Trigger delete if noone hold the persisted key anymore.
                });
            });
        }

        store_chat
            .messages
            .splice(range.clone(), replacement.clone());

        if range.start == 0 && !replacement.is_empty() {
            store_chat.update_title_based_on_first_message();
        }

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

fn update_attachments_with_persisted_key(
    messages: &mut [Message],
    attachment: &Attachment,
    persisted_key: String,
) -> bool {
    let mut found = false;

    for message in messages.iter_mut() {
        for att in message.content.attachments.iter_mut() {
            if att == attachment {
                let persisted_key = persisted_key.clone();
                att.set_persistent_key(persisted_key.clone());
                att.set_persisted_reader(move || {
                    let persisted_key = persisted_key.clone();
                    Box::pin(async move {
                        let fs = filesystem::global();
                        let content = fs
                            .read(Path::new(&persisted_key))
                            .await
                            .map_err(|e| std::io::Error::other(e))?;
                        Ok(content.into())
                    })
                });
                found = true;
            }
        }
    }

    found
}
