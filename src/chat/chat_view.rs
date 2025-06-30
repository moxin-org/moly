use makepad_widgets::*;
use moly_kit::*;

use crate::data::chats::chat::ChatID;
use crate::data::store::{ProviderSyncingStatus, Store};

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
}

impl LiveHook for ChatView {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.prompt_input(id!(chat.prompt)).write().disable();
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
    fn handle_actions(&mut self, _cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let mut chat_widget = self.chat(id!(chat));

        for action in actions {
            // Handle model selector actions
            match action.cast() {
                ModelSelectorAction::BotSelected(chat_id, bot) => {
                    if chat_id == self.chat_id {
                        chat_widget.write().bot_id = Some(bot.id.clone());

                        if let Some(chat) = store.chats.get_chat_by_id(chat_id) {
                            chat.borrow_mut().associated_bot = Some(bot.id.clone());
                            chat.borrow().save_and_forget();
                        }
                        // self.focus_on_prompt_input_pending = true;
                    }
                }
                _ => {}
            }

            let chat_id = self.chat_id.clone();
            // Hook into message updates to update the persisted chat history
            self.chat(id!(chat)).write_with(|chat| {
                let ui = self.ui_runner();
                chat.set_hook_after(move |group, _, _| {
                    for task in group.iter() {
                        // Handle new User messsages
                        if let ChatTask::InsertMessage(_index, message) = task {
                            let message = message.clone();
                            ui.defer_with_redraw(move |_me, _cx, scope| {
                                let chat_to_update = scope
                                    .data
                                    .get::<Store>()
                                    .unwrap()
                                    .chats
                                    .get_chat_by_id(chat_id);

                                if let Some(store_chat) = chat_to_update {
                                    let mut store_chat = store_chat.borrow_mut();
                                    let mut new_message = message.clone();
                                    new_message.metadata.is_writing = false;
                                    store_chat.messages.push(new_message);
                                    store_chat.update_title_based_on_first_message();
                                    store_chat.save_and_forget();
                                }
                            });
                        }

                        // Handle updated Bot messages
                        // UpdateMessage tasks mean that a bot message has been updated, either a User edit or a Bot message delta from the stream
                        // We fetch the current chat from the store and update the corresponding message, or insert it if it's not present
                        // (if it's the first chunk from the bot message)
                        if let ChatTask::UpdateMessage(index, message) = task {
                            let message = message.clone();
                            let index = index.clone();
                            ui.defer_with_redraw(move |me, _cx, scope| {
                                let chat_to_update = scope
                                    .data
                                    .get::<Store>()
                                    .unwrap()
                                    .chats
                                    .get_chat_by_id(chat_id);

                                if let Some(store_chat) = chat_to_update {
                                    let mut store_chat = store_chat.borrow_mut();
                                    if let Some(message_to_update) =
                                        store_chat.messages.get_mut(index)
                                    {
                                        message_to_update.content = message.content.clone();
                                        message_to_update.metadata.is_writing = false;
                                    } else {
                                        let mut new_message = message.clone();
                                        new_message.metadata.is_writing = false;
                                        store_chat.messages.push(new_message);
                                    }

                                    // Keep track of whether the message was updated while the chat view was inactive
                                    if !me.focused {
                                        me.message_updated_while_inactive = true;
                                    }

                                    store_chat.save_and_forget();
                                }
                            });
                        }

                        if let ChatTask::SetMessages(messages) = task {
                            let messages = messages.clone();
                            ui.defer_with_redraw(move |_me, _cx, scope| {
                                let chat_to_update = scope
                                    .data
                                    .get::<Store>()
                                    .unwrap()
                                    .chats
                                    .get_chat_by_id(chat_id);

                                if let Some(store_chat) = chat_to_update {
                                    let mut store_chat = store_chat.borrow_mut();
                                    store_chat.messages = messages;
                                    store_chat.save_and_forget();
                                }
                            });
                        }

                        if let ChatTask::DeleteMessage(index) = task {
                            let index = index.clone();
                            ui.defer_with_redraw(move |me, cx, scope| {
                                let store = scope.data.get_mut::<Store>().unwrap();
                                store.chats.delete_chat_message(index);
                                me.redraw(cx);
                            });
                        }
                    }
                });
            });
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
            self.chat(id!(chat)).write().bot_id = None;

            self.model_selector(id!(model_selector))
                .set_currently_selected_model(cx, None);

            self.redraw(cx);
        } else if bot_available && self.chat(id!(chat)).read().bot_id.is_none() {
            // If the bot is available and the chat widget doesn't have a bot_id, set the bot_id in the chat widget
            // This can happen if the bot or provider was re-enabled after being disabled while being selected
            self.chat(id!(chat)).write().bot_id = associiated_bot_id;
        }

        // If there is no selected bot, disable the prompt input
        if self.chat(id!(chat)).read().bot_id.is_none()
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
