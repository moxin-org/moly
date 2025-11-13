use makepad_widgets::*;
use moly_kit::controllers::chat::{
    ChatController, ChatControllerPlugin, ChatControllerPluginRegistrationId, ChatState,
    ChatStateMutation,
};
use moly_kit::utils::asynchronous::spawn;
use moly_kit::utils::vec::{VecEffect, VecMutation};
use moly_kit::*;

use crate::data::chats::chat::ChatID;
use crate::data::store::{ProviderSyncingStatus, Store};
use crate::shared::bot_context::BotContext;
use crate::shared::utils::attachments::{
    delete_attachment, generate_persistence_key, set_persistence_key_and_reader,
    write_attachment_to_key,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::chat_panel::ChatPanel;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chat_params::ChatParams;
    use moly_kit::widgets::chat::Chat;
    use moly_kit::widgets::prompt_input::PromptInput;

    PromptInputWithShadow = <PromptInput> {
        padding: {left: 15, right: 15, top: 8, bottom: 8}
        persistent = {
            // Shader to make the original RoundedView into a RoundedShadowView
            // (can't simply override the type of `persistent` because that removes the original children)
            clip_x:false, clip_y:false,

            show_bg: true,
            draw_bg: {
                color: #fefefe
                uniform border_radius: 7.0
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
    plugin_id: Option<ChatControllerPluginRegistrationId>,

    #[rust]
    focused: bool,

    // `chat_deck.rs` uses `WidgetRef::new_from_ptr` where `after_new_from_doc` is
    // not yet called and then tries to work with data from the widget, so ensuring
    // a controller is ready is necessary.
    // Do not expose this mutably unless you handle plugin unlinking on controller swap.
    // The plugin is still constructed in `after_new_from_doc`.
    #[rust(ChatController::new_arc())]
    chat_controller: Arc<Mutex<ChatController>>,

    #[rust]
    bot_context: Option<BotContext>,

    #[rust]
    prev_bot_context_id: Option<usize>,

    #[rust]
    prev_available_bots_len: usize,

    #[rust]
    message_updated_while_inactive: bool,

    #[rust]
    initial_bot_synced: bool,
}

impl LiveHook for ChatView {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.prompt_input(ids!(chat.prompt)).write().disable();
        let plugin_id = self
            .chat_controller
            .lock()
            .unwrap()
            .append_plugin(Glue::new(self.ui_runner()));
        self.plugin_id = Some(plugin_id);

        self.chat(ids!(chat))
            .write()
            .set_chat_controller(cx, Some(self.chat_controller.clone()));
    }
}

impl Drop for ChatView {
    fn drop(&mut self) {
        if let Some(plugin_id) = self.plugin_id.take() {
            self.chat(ids!(chat))
                .write()
                .chat_controller()
                .as_ref()
                .expect("chat controller missing")
                .lock()
                .unwrap()
                .remove_plugin(plugin_id);
        }

        self.unbind_bot_context();
    }
}

impl Widget for ChatView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.bind_bot_context(scope);

        self.ui_runner().handle(cx, event, scope, self);
        self.view.handle_event(cx, event, scope);

        self.handle_current_bot(scope);
        self.handle_unread_messages(scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.bind_bot_context(scope);

        // Sync bot_id from Store to Controller on first draw
        if !self.initial_bot_synced {
            self.sync_bot_from_store(scope);
            self.initial_bot_synced = true;
        }

        // On mobile, only set padding on top of the prompt
        // TODO: do this with AdaptiveView instead of apply_over
        if !cx.display_context.is_desktop() && cx.display_context.is_screen_size_known() {
            self.prompt_input(ids!(chat.prompt)).apply_over(
                cx,
                live! {
                    padding: {bottom: 50, left: 20, right: 20}
                },
            );
        } else {
            self.prompt_input(ids!(chat.prompt)).apply_over(
                cx,
                live! {
                    padding: {left: 10, right: 10, top: 8, bottom: 8}
                },
            );
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatView {
    // TODO: Only perform this checks after certain actions like provider sync or provider updates (e.g. disable/enable provider)
    // Refactor this to be simpler and more unified with the behavior of the model selector
    fn handle_current_bot(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        // Read bot_id from controller (runtime source of truth)
        let controller_bot_id = self.chat_controller.lock().unwrap().state().bot_id.clone();

        // Check if the controller's bot is currently available (prevents race condition with Store)
        let controller_bot_available = if let Some(bot_id) = &controller_bot_id {
            store
                .chats
                .get_all_bots(true)
                .iter()
                .any(|bot| &bot.id == bot_id)
        } else {
            false
        };

        let mut prompt_input = self.prompt_input(ids!(chat.prompt));

        // CLEARING: Use controller_bot_id for availability check
        // If the controller's bot is not available and we know it won't be available soon, clear it
        if !controller_bot_available
            && controller_bot_id.is_some()
            && store.provider_syncing_status == ProviderSyncingStatus::Synced
        {
            // Temporarily clear controller state while provider is disabled
            // (Store's associated_bot persists for restoration)
            self.chat_controller
                .lock()
                .unwrap()
                .dispatch_mutation(ChatStateMutation::SetBotId(None));
        }
        // RESTORATION: Use Store's persistent state when controller is None
        else if controller_bot_id.is_none() {
            // Skip restoration if bots haven't loaded yet (prevents spam during async load)
            let controller_bots = self.chat_controller.lock().unwrap().state().bots.clone();
            if controller_bots.is_empty() {
                // Bots not loaded yet, will retry on next event after load completes
                return;
            }

            // Read stored bot from Store (persistent state)
            if let Some(stored_bot_id) = store
                .chats
                .get_chat_by_id(self.chat_id)
                .and_then(|chat| chat.borrow().associated_bot.clone())
            {
                // Check if stored bot is available
                let stored_bot_available = store
                    .chats
                    .get_all_bots(true)
                    .iter()
                    .any(|bot| bot.id == stored_bot_id);

                if stored_bot_available {
                    self.chat_controller
                        .lock()
                        .unwrap()
                        .dispatch_mutation(ChatStateMutation::SetBotId(Some(stored_bot_id)));
                }
            }
        }

        // If there is no selected bot, disable the prompt input
        let is_streaming = self.chat_controller.lock().unwrap().state().is_streaming;
        if !is_streaming
            && (controller_bot_id.is_none()
                || !controller_bot_available
                || store.provider_syncing_status != ProviderSyncingStatus::Synced)
        {
            prompt_input.write().disable();
        } else {
            prompt_input.write().enable();
        }
    }

    fn handle_unread_messages(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        if self.message_updated_while_inactive {
            // If the message is done writing, and this chat view is not focused
            // set the chat as having unread messages (show a badge on the chat history card)
            if !self.chat(ids!(chat)).read().is_streaming() && !self.focused {
                if let Some(chat) = store.chats.get_chat_by_id(self.chat_id) {
                    chat.borrow_mut().has_unread_messages = true;
                    self.message_updated_while_inactive = false;
                }
            }
        }
    }

    /// Syncs the bot_id from Store's associated_bot to ChatController state.
    /// This ensures ChatController reflects the persisted bot selection.
    fn sync_bot_from_store(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if let Some(chat) = store.chats.get_chat_by_id(self.chat_id) {
            let associated_bot = chat.borrow().associated_bot.clone();

            // Get current bot_id from controller
            let current_bot_id = self.chat_controller.lock().unwrap().state().bot_id.clone();

            // Only sync if they differ to avoid unnecessary mutations
            if current_bot_id != associated_bot {
                self.chat_controller
                    .lock()
                    .unwrap()
                    .dispatch_mutation(ChatStateMutation::SetBotId(associated_bot));
            }
        }
    }

    pub fn bind_bot_context(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let store_bot_context_id = store.bot_context.as_ref().map(|bc| bc.id());
        let self_bot_context_id = self.bot_context.as_ref().map(|bc| bc.id());

        if self_bot_context_id != store_bot_context_id {
            self.bot_context = store.bot_context.clone();
            if let Some(bot_context) = &mut self.bot_context {
                bot_context.add_chat_controller(self.chat_controller.clone());
            }
        }

        // Only rebuild grouping and filter when bot_context or available_bots changes
        // This handles both intentional changes (provider enable/disable) and
        // async loading (bots fetched after bot_context created at startup)
        let current_bots_len = store.chats.available_bots.len();
        if self.prev_bot_context_id != store_bot_context_id
            || self.prev_available_bots_len != current_bots_len
        {
            self.prev_bot_context_id = store_bot_context_id;
            self.prev_available_bots_len = current_bots_len;

            // Build lookup table for grouping
            let mut bot_groups: HashMap<BotId, moly_kit::BotGroup> = HashMap::new();

            for (bot_id, provider_bot) in &store.chats.available_bots {
                if let Some(provider) = store.chats.providers.get(&provider_bot.provider_id) {
                    let icon = store
                        .get_provider_icon(&provider.name)
                        .map(|dep| moly_kit::protocol::Picture::Dependency(dep));
                    bot_groups.insert(
                        bot_id.clone(),
                        moly_kit::BotGroup {
                            id: provider.id.clone(),
                            label: provider.name.clone(),
                            icon,
                        },
                    );
                }
            }

            // Create grouping function using lookup utility
            use moly_kit::widgets::model_selector::create_lookup_grouping;

            let grouping_fn =
                create_lookup_grouping(move |bot_id: &BotId| bot_groups.get(bot_id).cloned());

            // Set grouping on the ModelSelector inside PromptInput
            let chat = self.chat(ids!(chat));
            chat.read()
                .prompt_input_ref()
                .widget(ids!(model_selector))
                .as_model_selector()
                .set_grouping(Some(grouping_fn));

            // Update filter when bot_context changes
            let chat = self.chat(ids!(chat));
            if let Some(mut list) = chat
                .read()
                .prompt_input_ref()
                .widget(ids!(model_selector.options.list_container.list))
                .borrow_mut::<moly_kit::widgets::model_selector_list::ModelSelectorList>()
            {
                let filter = crate::chat::moly_bot_filter::MolyBotFilter::new(
                    store.chats.available_bots.clone(),
                );
                list.filter = Some(Box::new(filter));
            }
        }
    }

    pub fn unbind_bot_context(&mut self) {
        if let Some(mut bot_context) = self.bot_context.take() {
            bot_context.remove_chat_controller(&self.chat_controller);
        }
    }

    pub fn chat_controller(&self) -> &Arc<Mutex<ChatController>> {
        &self.chat_controller
    }
}

impl ChatViewRef {
    pub fn set_chat_id(&mut self, chat_id: ChatID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.chat_id = chat_id;
            // Reset sync flag so bot_id will be synced from Store on next draw
            inner.initial_bot_synced = false;
        }
    }

    pub fn set_bot_id(&mut self, bot_id: Option<BotId>) {
        if let Some(inner) = self.borrow_mut() {
            inner
                .chat_controller
                .lock()
                .unwrap()
                .dispatch_mutation(ChatStateMutation::SetBotId(bot_id));
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.focused = focused;
        }
    }
}

/// Glue between Moly and Moly Kit.
pub struct Glue {
    ui: UiRunner<ChatView>,
    marked_attachments: HashSet<Attachment>,
    persisting_attachments: Arc<Mutex<HashSet<Attachment>>>,
}

impl ChatControllerPlugin for Glue {
    fn on_state_mutation(&mut self, mutation: &ChatStateMutation, state: &ChatState) {
        match mutation {
            ChatStateMutation::MutateMessages(mutation) => {
                self.replicate_messages_mutation_to_store(mutation);
                self.mark_attachments(mutation, state);
            }
            ChatStateMutation::SetBotId(bot_id) => {
                self.replicate_bot_id_to_store(bot_id.clone());
            }
            _ => {}
        }
    }

    fn on_state_ready(&mut self, state: &ChatState, _mutatins: &[ChatStateMutation]) {
        self.sweep_attachments(state);
    }
}

impl Glue {
    pub fn new(ui: UiRunner<ChatView>) -> Self {
        Self {
            ui,
            marked_attachments: HashSet::new(),
            persisting_attachments: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    fn replicate_messages_mutation_to_store(&self, mutation: &VecMutation<Message>) {
        let mutation = mutation.clone();

        self.ui.defer(move |chat_view, _, scope| {
            let store = scope.data.get_mut::<Store>().unwrap();

            let Some(store_chat) = store.chats.get_chat_by_id(chat_view.chat_id) else {
                return;
            };

            let modified_first_message =
                mutation
                    .effects(&store_chat.borrow().messages)
                    .any(|effect| match effect {
                        VecEffect::Insert(index, _) | VecEffect::Update(index, _, _) => index == 0,
                        VecEffect::Remove(_, _, _) => false,
                    });

            mutation.apply(&mut store_chat.borrow_mut().messages);

            if modified_first_message {
                store_chat
                    .borrow_mut()
                    .update_title_based_on_first_message();
            }

            // Write to disk.
            store_chat.borrow_mut().save_and_forget();

            // Keep track of whether the message was updated while the chat view was inactive
            if !chat_view.focused {
                chat_view.message_updated_while_inactive = true;
            }
        });
    }

    fn replicate_bot_id_to_store(&self, bot_id: Option<BotId>) {
        self.ui.defer(move |chat_view, _, scope| {
            let store = scope.data.get_mut::<Store>().unwrap();

            let Some(store_chat) = store.chats.get_chat_by_id(chat_view.chat_id) else {
                return;
            };

            store_chat.borrow_mut().associated_bot = bot_id;

            // Write to disk.
            store_chat.borrow_mut().save_and_forget();
        });
    }

    fn mark_attachments(&mut self, mutation: &VecMutation<Message>, state: &ChatState) {
        self.marked_attachments.clear();

        for effect in mutation.effects(&state.messages) {
            match effect {
                VecEffect::Insert(_, messages) => {
                    // Dev note: To make this reusable outside of Moly, attachment inserts
                    // should be treated in the same way as deletes, re-scanning to
                    // verify an actual insert happened.

                    for message in messages {
                        for attachment in &message.content.attachments {
                            if !attachment.has_persistence_key()
                                && !self
                                    .persisting_attachments
                                    .lock()
                                    .unwrap()
                                    .contains(attachment)
                            {
                                self.persist_attachment(attachment.clone());
                            }
                        }
                    }
                }
                VecEffect::Remove(_start, _end, removed) => {
                    for messages in removed {
                        for attachment in &messages.content.attachments {
                            if attachment.has_persistence_key() {
                                self.marked_attachments.insert(attachment.clone());
                            }
                        }
                    }
                }
                VecEffect::Update(_index, _from, _to) => {
                    // Dev note: To make this reusable outside of Moly, attachment updates
                    // should be analyzed as well.
                }
            }
        }
    }

    fn sweep_attachments(&mut self, state: &ChatState) {
        if self.marked_attachments.is_empty() {
            return;
        }

        for message in &state.messages {
            for attachment in &message.content.attachments {
                self.marked_attachments.remove(attachment);
            }
        }

        for attachment in &self.marked_attachments {
            let attachment = attachment.clone();
            spawn(async move {
                let key = attachment.get_persistence_key().unwrap();

                ::log::info!(
                    "Sweeping persisted attachment, named {}, with key: {}",
                    attachment.name,
                    key
                );

                if let Err(e) = delete_attachment(&attachment).await {
                    ::log::error!(
                        "Failed to sweep persisted attachment, named {}, with key {}: {}",
                        attachment.name,
                        key,
                        e
                    );
                }
            });
        }
    }

    fn persist_attachment(&self, attachment: Attachment) {
        let ui = self.ui;

        let persisting_attachments = self.persisting_attachments.clone();

        // Mark the attachment as being processed to avoid re-processing it.
        persisting_attachments
            .lock()
            .unwrap()
            .insert(attachment.clone());

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
            ui.defer(move |me, _cx, _scope| {
                let chat = me.chat(ids!(chat));
                let chat_controller = chat.read().chat_controller().expect("chat controller missing").clone();
                let mut found = false;

                {
                    // Important to hold the lock to avoid differences between reads and writes.
                    let mut lock = chat_controller.lock().unwrap();
                    let mut mutations: Vec<ChatStateMutation> = Vec::new();

                    for (index, message) in lock.state().messages.iter().enumerate() {
                        if message.content.attachments.iter().any(|att| att == &attachment) {
                            found = true;
                            let mut updated_message = message.clone();

                            for att in &mut updated_message.content.attachments {
                                if att == &attachment {
                                    set_persistence_key_and_reader(att, key.clone());
                                }
                            }

                            mutations.push(VecMutation::Update(index, updated_message).into());
                        }
                    }

                    lock.dispatch_mutations(mutations);
                }

                persisting_attachments.lock().unwrap().remove(&attachment);

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
