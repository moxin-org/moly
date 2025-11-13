use std::collections::{HashMap, VecDeque};

use makepad_widgets::*;
use moly_kit::utils::vec::VecMutation;
use moly_kit::*;

use super::chat_view::ChatViewRef;
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
        padding: {top: 18, bottom: 0, right: 28, left: 28},

        chat_view_template: <ChatView> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatsDeck {
    #[deref]
    view: View,

    /// All currently active chat instances, keyed by their corresponding ChatID.
    /// Each chat maintains its own instance to keep background streaming alive.
    #[rust]
    chat_view_refs: HashMap<ChatID, ChatViewRef>,

    /// LRU tracking for memory management.
    /// When we exceed MAX_CHAT_VIEWS, we evict the oldest chat (unless it's streaming).
    #[rust]
    chat_view_accessed_order: VecDeque<ChatID>,

    /// The currently visible/focused chat id.
    #[rust]
    currently_visible_chat_id: Option<ChatID>,

    /// The template for creating new chat views.
    #[live]
    chat_view_template: Option<LivePtr>,
}

/// The maximum number of chat views that can be kept alive at once.
/// Prevents unbounded memory growth in long-running sessions.
const MAX_CHAT_VIEWS: usize = 10;

impl Widget for ChatsDeck {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // Handle events for ALL instances to keep background activity (streaming, etc.) alive
        for (_, chat_view) in self.chat_view_refs.iter_mut() {
            chat_view.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Because chats_deck is being cached, overriding its properties in the DSL does not take effect.
        // For now we'll override them through apply_over.
        // TODO: Do not use CachedWidget, create a shared structure of chat instances that is shared across layouts.
        if cx.display_context.is_desktop() {
            self.view.apply_over(
                cx,
                live! {padding: {top: 18, bottom: 0, right: 28, left: 28} },
            );
        } else {
            self.view.apply_over(
                cx,
                live! { padding: {top: 55, left: 0, right: 0, bottom: 0} },
            );
        }

        cx.begin_turtle(walk, self.layout);

        // Draw only the currently visible chat
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
                        self.create_or_update_chat_view(cx, &chat.borrow());
                    }
                }
                ChatAction::StartWithoutEntity => {
                    let chat_id = store.chats.create_empty_chat(None);
                    let chat = store.chats.get_chat_by_id(chat_id);
                    if let Some(chat) = chat {
                        self.create_or_update_chat_view(cx, &chat.borrow());
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

                        self.create_or_update_chat_view(cx, &chat.borrow());
                    }
                }
                _ => {}
            }

            // Handle Context Capture
            if let CaptureAction::Capture { event } = action.cast() {
                // Paste the captured text into the currently visible chat
                if let Some(chat_id) = self.currently_visible_chat_id {
                    if let Some(chat_view) = self.chat_view_refs.get_mut(&chat_id) {
                        chat_view
                            .prompt_input(ids!(prompt))
                            .write()
                            .set_text(cx, event.contents());
                    }
                }
            }
        }
    }
}

impl ChatsDeck {
    pub fn create_or_update_chat_view(&mut self, cx: &mut Cx, chat_data: &ChatData) {
        // Check if an instance already exists for this chat
        if let Some(existing_view) = self.chat_view_refs.get_mut(&chat_data.id) {
            // Instance exists, just make it visible and focused
            self.currently_visible_chat_id = Some(chat_data.id);

            // Update focus states
            existing_view.set_focused(true);
            for (id, chat_view) in self.chat_view_refs.iter_mut() {
                if *id != chat_data.id {
                    chat_view.set_focused(false);
                }
            }

            // Update LRU access order
            self.chat_view_accessed_order
                .retain(|id| *id != chat_data.id);
            self.chat_view_accessed_order.push_back(chat_data.id);

            return; // EARLY RETURN, don't recreate!
        }

        // No existing instance, create a new one
        let chat_view = WidgetRef::new_from_ptr(cx, self.chat_view_template);
        let mut chat_view = chat_view.as_chat_view();

        // Initialize new instance
        chat_view.set_chat_id(chat_data.id);

        // Load messages into the controller
        chat_view
            .borrow()
            .unwrap()
            .chat_controller()
            .lock()
            .unwrap()
            .dispatch_mutation(VecMutation::Set(chat_data.messages.clone()));

        // Sync associated_bot from Store to ChatController
        if let Some(bot_id) = &chat_data.associated_bot {
            chat_view.set_bot_id(Some(bot_id.clone()));
        }

        // Set as focused
        chat_view.set_focused(true);

        // Insert into HashMap
        self.chat_view_refs.insert(chat_data.id, chat_view);
        self.currently_visible_chat_id = Some(chat_data.id);

        // Defocus other chats
        for (id, cv) in self.chat_view_refs.iter_mut() {
            if *id != chat_data.id {
                cv.set_focused(false);
            }
        }

        // Add to LRU tracking
        self.chat_view_accessed_order.push_back(chat_data.id);

        // Evict oldest instance if we exceed max
        if self.chat_view_accessed_order.len() > MAX_CHAT_VIEWS {
            let oldest_id = self.chat_view_accessed_order.pop_front().unwrap();
            if let Some(oldest_view) = self.chat_view_refs.get_mut(&oldest_id) {
                // Don't evict if currently streaming
                if !oldest_view.chat(ids!(chat)).read().is_streaming() {
                    self.chat_view_refs.remove(&oldest_id);
                } else {
                    // Put back in queue if streaming
                    self.chat_view_accessed_order.push_front(oldest_id);
                }
            }
        }

        // TODO: Focus on prompt input
    }
}
