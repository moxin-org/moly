use std::collections::{HashMap, VecDeque};

use makepad_widgets::*;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use crate::chat::chat_view::ChatViewWidgetRefExt;
use crate::data::capture::CaptureAction;
use crate::data::chats::chat::Chat as ChatData;
use crate::data::chats::chat::ChatID;
use crate::data::providers::ProviderType;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;

use super::chat_view::ChatViewRef;
use super::model_selector::ModelSelectorWidgetRefExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chat_view::ChatView;
    use moly_kit::widgets::chat::Chat;

    ChatsDeck = {{ChatsDeck}} {
        width: Fill,
        height: Fill,
        spacing: 10,
        flow: Right

        chat_view_template: <ChatView> {}
    }

    pub ChatScreen = {{ChatScreen}} {
        width: Fill,
        height: Fill,
        spacing: 10,

        <View> {
            width: Fit,
            height: Fill,

            chat_history = <ChatHistory> {}
        }

        chats_deck = <ChatsDeck> {}

        // TODO: Add chat params back in, only when the model is a local model (MolyServer)
        // currenlty MolyKit does not support chat params
        //
        // <View> {
        //     width: Fit,
        //     height: Fill,
        //
        //     chat_params = <ChatParams> {}
        // }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreen {
    #[deref]
    view: View,

    #[rust(true)]
    first_render: bool,

    #[rust]
    should_load_repo_to_store: bool,

    #[rust]
    creating_bot_repo: bool,
}

impl Widget for ChatScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);

        // TODO This check is actually copied from Makepad view.rs file
        // It's not clear why it's needed here, but without this line
        // the "View all files" link in Discover section does not work after visiting the chat screen
        if self.visible || !event.requires_visibility() {
            self.view.handle_event(cx, event, scope);
        }

        let store = scope.data.get_mut::<Store>().unwrap();

        let should_recreate_bot_repo = store.bot_repo.is_none();

        if self.should_load_repo_to_store {
            self.should_load_repo_to_store = false;
        } else if (self.first_render || should_recreate_bot_repo) && !self.creating_bot_repo {
            self.create_bot_repo(cx, scope);
            self.first_render = false;
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatScreen {
    fn create_bot_repo(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let multi_client = {
            let mut multi_client = MultiClient::new();

            for provider in store.chats.providers.iter() {
                match provider.1.provider_type {
                    ProviderType::OpenAI | ProviderType::MolyServer => {
                        if provider.1.enabled
                            && (provider.1.api_key.is_some()
                                || provider.1.url.starts_with("http://localhost"))
                        {
                            let mut new_client = OpenAIClient::new(provider.1.url.clone());
                            if let Some(key) = provider.1.api_key.as_ref() {
                                let _ = new_client.set_key(&key);
                            }
                            multi_client.add_client(Box::new(new_client));
                        }
                    }
                    ProviderType::MoFa => {
                        // For MoFa we don't require an API key
                        if provider.1.enabled {
                            let mut new_client = OpenAIClient::new(provider.1.url.clone());
                            if let Some(key) = provider.1.api_key.as_ref() {
                                let _ = new_client.set_key(&key);
                            }
                            multi_client.add_client(Box::new(new_client));
                        }
                    }
                    ProviderType::DeepInquire => {
                        let mut new_client = DeepInquireClient::new(provider.1.url.clone());
                        if let Some(key) = provider.1.api_key.as_ref() {
                            let _ = new_client.set_key(&key);
                        }
                        multi_client.add_client(Box::new(new_client));
                    }
                }
            }

            multi_client
        };

        let mut repo: BotRepo = multi_client.into();
        store.bot_repo = Some(repo.clone());

        self.creating_bot_repo = true;

        let ui = self.ui_runner();
        spawn(async move {
            repo.load().await;
            ui.defer_with_redraw(move |me, _cx, _scope| {
                me.should_load_repo_to_store = true;
                me.creating_bot_repo = false;
            });
        });
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

    #[rust]
    currently_visible_chat_id: Option<ChatID>,

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
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
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
                        self.create_or_update_chat_view(cx, &chat.borrow(), store.bot_repo.clone());
                    }
                }
                ChatAction::StartWithoutEntity => {
                    let chat_id = store.chats.create_empty_chat(None);
                    let chat = store.chats.get_chat_by_id(chat_id);
                    if let Some(chat) = chat {
                        self.create_or_update_chat_view(cx, &chat.borrow(), store.bot_repo.clone());
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

                        self.create_or_update_chat_view(cx, &chat.borrow(), store.bot_repo.clone());

                        self.redraw(cx);
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
        bot_repo: Option<BotRepo>,
    ) {
        let chat_view_to_update;
        if let Some(chat_view) = self.chat_view_refs.get_mut(&chat.id) {
            chat_view.set_chat_id(chat.id);
            self.currently_visible_chat_id = Some(chat.id);

            chat_view_to_update = chat_view.clone();
        } else {
            let chat_view = WidgetRef::new_from_ptr(cx, self.chat_view_template);
            chat_view.chat(id!(chat)).write().bot_repo = bot_repo;
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
            chat_view_to_update.chat(id!(chat)).write().bot_id = Some(bot_id.clone());
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
                if let Some(latest_message) = chat_view
                    .messages(id!(chat.messages))
                    .read()
                    .messages
                    .last()
                {
                    if latest_message.is_writing {
                        // If the latest message is being streamed, do not remove the chat view
                        should_remove = false;
                    }
                }

                if should_remove {
                    self.chat_view_refs.remove(&least_recently_used_chat_id);
                    self.chat_view_accesed_order.remove(0);
                }
            }
        }

        // TODO: Focus on prompt input
    }
}
