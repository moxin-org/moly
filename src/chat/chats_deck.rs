use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use makepad_widgets::*;
use moly_kit::controllers::chat::ChatController;
use moly_kit::*;

use super::chat_view::ChatViewRef;
use super::model_selector::ModelSelectorWidgetRefExt;
use crate::chat::chat_view::ChatViewWidgetRefExt;
use crate::data::capture::CaptureAction;
use crate::data::chats::chat::Chat as ChatData;
use crate::data::chats::chat::ChatID;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::modal::*;
    use crate::shared::widgets::*;
    use crate::chat::chat_view::ChatView;

    pub ChatsDeck = {{ChatsDeck}} {
        width: Fill, height: Fill
        padding: {top: 18, bottom: 10, right: 28, left: 28},

        chat_view_template: <ChatView> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatsDeck {
    #[deref]
    view: View,

    /// All the currently existing chat views. Keyed by their corresponding ChatID.
    #[rust]
    chat_view_refs: HashMap<ChatID, ChatViewRef>,

    /// The order in which the chat views were accessed.
    /// Used as a simple LRU cache to determine which chat view to remove when the deck is full.
    /// We only drop a chat view if it's not currently streaming a response from the bot.
    #[rust]
    chat_view_accesed_order: VecDeque<ChatID>,

    /// The currently visible chat id.
    #[rust]
    currently_visible_chat_id: Option<ChatID>,

    /// A list of chat views that need to be synced with the [BotContext].
    /// This is used to avoid interrumpting the chat stream when the [BotContext] is being updated.
    #[rust]
    chats_views_pending_sync: Vec<ChatViewRef>,

    /// The template for creating new chat views.
    #[live]
    chat_view_template: Option<LivePtr>,
}

/// The maximum number of chat views that can be displayed at once.
const MAX_CHAT_VIEWS: usize = 10;

impl Widget for ChatsDeck {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        for (_, chat_view) in self.chat_view_refs.iter_mut() {
            chat_view.handle_event(cx, event, scope);
        }

        // Sync the [BotContext] for chat views that are not currently streaming
        let store = scope.data.get_mut::<Store>().unwrap();
        for chat_view in self.chats_views_pending_sync.iter_mut() {
            if chat_view
                .messages(id!(chat.messages))
                .read()
                .messages
                .last()
                .unwrap()
                .metadata
                .is_idle()
            {
                chat_view
                    .chat(id!(chat))
                    .write()
                    .set_bot_context(cx, store.bot_context.clone());
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Because chats_deck is being cached, overriding its properties in the DSL does not take effect.
        // For now we'll override them through apply_over.
        // TODO: Do not use CachedWidget, create a shared structure of chat instances that is shared across layouts.
        if cx.display_context.is_desktop() {
            self.view.apply_over(
                cx,
                live! {padding: {top: 18, bottom: 10, right: 28, left: 28} },
            );
        } else {
            self.view.apply_over(
                cx,
                live! { padding: {top: 55, left: 0, right: 0, bottom: 0} },
            );
        }

        cx.begin_turtle(walk, self.layout);

        if let Some(chat_id) = self.currently_visible_chat_id {
            if let Some(chat_view) = self.chat_view_refs.get_mut(&chat_id) {
                let _ = chat_view.draw(cx, scope);
            }
        }

        cx.end_turtle();
        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatsDeck {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        for action in actions {
            // Handle chat start
            match action.cast() {
                ChatAction::Start(bot_id) => {
                    let chat_id = store.chats.create_empty_chat(Some(bot_id.clone()));
                    let chat = store.chats.get_chat_by_id(chat_id);
                    if let Some(chat) = chat {
                        self.create_or_update_chat_view(
                            cx,
                            &chat.borrow(),
                            store.bot_context.clone(),
                        );
                    }
                }
                ChatAction::StartWithoutEntity => {
                    let chat_id = store.chats.create_empty_chat(None);
                    let chat = store.chats.get_chat_by_id(chat_id);
                    if let Some(chat) = chat {
                        self.create_or_update_chat_view(
                            cx,
                            &chat.borrow(),
                            store.bot_context.clone(),
                        );
                    }
                }
                _ => {}
            }

            // Handle chat selection (from chat history)
            match action.cast() {
                ChatAction::ChatSelected(chat_id) => {
                    let selected_chat = store.chats.get_chat_by_id(chat_id);

                    if let Some(chat) = selected_chat {
                        store
                            .preferences
                            .set_current_chat_model(chat.borrow().associated_bot.clone());

                        self.create_or_update_chat_view(
                            cx,
                            &chat.borrow(),
                            store.bot_context.clone(),
                        );
                    }
                }
                _ => {}
            }

            // Handle Context Capture
            if let CaptureAction::Capture { event } = action.cast() {
                // Paste the captured text into the currently selected chat
                if let Some(chat_view) = self
                    .chat_view_refs
                    .get_mut(&self.currently_visible_chat_id.unwrap())
                {
                    chat_view
                        .prompt_input(id!(prompt))
                        .write()
                        .set_text(cx, event.contents());
                }
            }
        }
    }
}

impl ChatsDeck {
    pub fn create_or_update_chat_view(
        &mut self,
        cx: &mut Cx,
        chat: &ChatData,
        chat_controller: Option<Arc<Mutex<ChatController>>>,
    ) {
        let mut chat_view_to_update;
        // If the chat view already exists, update it
        if let Some(chat_view) = self.chat_view_refs.get_mut(&chat.id) {
            chat_view.set_chat_id(chat.id);
            self.currently_visible_chat_id = Some(chat.id);

            chat_view_to_update = chat_view.clone();
        } else {
            // Create a new chat view
            let chat_view = WidgetRef::new_from_ptr(cx, self.chat_view_template);
            chat_view
                .chat(id!(chat))
                .write()
                .set_chat_controller(cx, chat_controller.clone());
            chat_view.as_chat_view().set_chat_id(chat.id);

            self.chat_view_refs
                .insert(chat.id, chat_view.as_chat_view());
            self.currently_visible_chat_id = Some(chat.id);

            chat_view_to_update = chat_view.as_chat_view();
        }

        // Set messages
        // If the chat is already loaded do not set the messages again, as it might cause
        // unwanted side effects, i.e. canceling any ongoing streaming response from the bot
        if chat_view_to_update
            .messages(id!(chat.messages))
            .read()
            .messages
            .is_empty()
        {
            chat_view_to_update
                .messages(id!(chat.messages))
                .write()
                .set_messages(chat.messages.clone(), true);
        }

        // Set associated bot
        if let Some(bot_id) = &chat.associated_bot {
            chat_view_to_update
                .model_selector(id!(model_selector))
                .set_currently_selected_model(cx, Some(bot_id.clone()));
            chat_view_to_update
                .chat(id!(chat))
                .write()
                .set_bot_id(cx, Some(bot_id.clone()));
        }

        // Set this chat view as focused and all other chat views as not focused
        chat_view_to_update.set_focused(true);
        for (id, chat_view) in self.chat_view_refs.iter_mut() {
            if id != &chat.id {
                chat_view.set_focused(false);
            }
        }

        // Update the access order
        self.chat_view_accesed_order.retain(|id| *id != chat.id);
        self.chat_view_accesed_order.push_back(chat.id);

        // Remove the least recently used chat view if the deck is full
        if self.chat_view_accesed_order.len() > MAX_CHAT_VIEWS {
            let least_recently_used_chat_id = self.chat_view_accesed_order.pop_front().unwrap();
            if let Some(chat_view) = self.chat_view_refs.get_mut(&least_recently_used_chat_id) {
                let mut should_remove = true;
                // Check if the latest message is currently being streamed
                if chat_view.chat(id!(chat)).read().is_streaming() {
                    should_remove = false;
                }

                if should_remove {
                    self.chat_view_refs.remove(&least_recently_used_chat_id);
                    self.chat_view_accesed_order.remove(0);
                }
            }
        }

        // TODO: Focus on prompt input
    }

    fn sync_bot_contexts(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        for (_, chat_view) in self.chat_view_refs.iter_mut() {
            // Only set the BotContext if the chat is not currently streaming, otherwise it will be interrumpted.
            if !chat_view.chat(id!(chat)).read().is_streaming() {
                chat_view
                    .chat(id!(chat))
                    .write()
                    .set_bot_context(cx, store.bot_context.clone());
            } else {
                self.chats_views_pending_sync.push(chat_view.clone());
            }
        }
    }
}

impl ChatsDeckRef {
    pub fn sync_bot_contexts(&mut self, cx: &mut Cx, scope: &mut Scope) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.sync_bot_contexts(cx, scope);
        }
    }
}
