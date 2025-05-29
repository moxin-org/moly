use std::collections::{HashMap, VecDeque};

use makepad_widgets::*;
use moly_kit::protocol::Picture;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use super::chat_view::ChatViewRef;
use super::model_selector::ModelSelectorWidgetRefExt;
use crate::app::NavigationAction;
use crate::chat::chat_view::ChatViewWidgetRefExt;
use crate::data::capture::CaptureAction;
use crate::data::chats::chat::Chat as ChatData;
use crate::data::chats::chat::ChatID;
use crate::data::providers::ProviderType;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;
use crate::shared::modal::ModalWidgetExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chat_view::ChatView;
    use crate::shared::modal::*;
    use moly_kit::widgets::chat::Chat;

    ICON_MENU = dep("crate://self/resources/images/hamburger_menu_icon.png")

    ChatsDeck = {{ChatsDeck}} {
        width: Fill, height: Fill
        padding: {top: 38, bottom: 10, right: 28, left: 28},
        spacing: 10
        flow: Right

        chat_view_template: <ChatView> {}
    }

    ChatScreenMobile = {{ChatScreenMobile}} {
        width: Fill, height: Fill
        flow: Overlay

        menu_toggle = <View> {
            margin: {top: 10, left: 15}
            width: Fit, height: Fit
            cursor: Hand
            <IconSet> {
                text: "" // FontAwesome f0c9
                draw_text: {
                    color: #x0
                    text_style: { font_size: 18.0 }
                }
            }
        }

        <CachedWidget> {
            chats_deck = <ChatsDeck> {
                margin: {top: 55}
                padding: 0
            }
        }

        chat_history_modal = <Modal> {
            align: {x: 0.0, y: 0.0}
            bg_view: {
                width: Fill, height: Fill
                visible: true
            }
            content: {
                width: Fit, height: Fill

                <View> {
                    show_bg: true
                    draw_bg: {
                        color: (MAIN_BG_COLOR)
                    }
                    width: Fit, height: Fill
                    flow: Down
                    spacing: 10

                    chat_history = <ChatHistory> {}
                    <View> {
                        width: Fill, height: Fit

                        <View> { width: Fill, height: 1}
                        settings_button = <View> {
                            width: Fit, height: Fit
                            padding: {right: 12, bottom: 5}
                            cursor: Hand
                            <IconSet> {
                                text: "" // FontAwesome 141
                                draw_text: {
                                    color: #x0
                                    text_style: { font_size: 18.0 }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub ChatScreen = {{ChatScreen}} {
        width: Fill, height: Fill
        spacing: 10

        adaptive_view = <AdaptiveView> {
            Mobile = {
                <ChatScreenMobile> {}
            }

            Desktop = {
                <View> {
                    width: Fit, height: Fill
                    chat_history = <ChatHistory> {}
                }

                <CachedWidget> {
                    chats_deck = <ChatsDeck> {}
                }
            }
        }

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
    creating_bot_context: bool,
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

        let should_recreate_bot_context = store.bot_context.is_none();

        if (self.first_render || should_recreate_bot_context) && !self.creating_bot_context {
            self.create_bot_context(cx, scope);
            self.first_render = false;
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatScreen {
    fn create_bot_context(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let multi_client = {
            let mut multi_client = MultiClient::new();

            for (_key, provider) in store.chats.providers.iter() {
                match provider.provider_type {
                    ProviderType::OpenAI | ProviderType::MolyServer => {
                        if provider.enabled
                            && (provider.api_key.is_some()
                                || provider.url.starts_with("http://localhost"))
                        {
                            let mut new_client = OpenAIClient::new(provider.url.clone());
                            if let Some(key) = provider.api_key.as_ref() {
                                let _ = new_client.set_key(&key);
                            }

                            if let Some(icon) = store.get_provider_icon(&provider.name) {
                                new_client.set_provider_avatar(Picture::Dependency(icon));
                            }

                            multi_client.add_client(Box::new(new_client));
                        }
                    }
                    ProviderType::MoFa => {
                        // For MoFa we don't require an API key
                        if provider.enabled {
                            let mut new_client = OpenAIClient::new(provider.url.clone());
                            if let Some(key) = provider.api_key.as_ref() {
                                let _ = new_client.set_key(&key);
                            }

                            if let Some(icon) = store.get_provider_icon(&provider.name) {
                                new_client.set_provider_avatar(Picture::Dependency(icon));
                            }

                            multi_client.add_client(Box::new(new_client));
                        }
                    }
                    ProviderType::DeepInquire => {
                        let mut new_client = DeepInquireClient::new(provider.url.clone());
                        if let Some(key) = provider.api_key.as_ref() {
                            let _ = new_client.set_key(&key);
                        }

                        if let Some(icon) = store.get_provider_icon(&provider.name) {
                            new_client.set_provider_avatar(Picture::Dependency(icon));
                        }

                        multi_client.add_client(Box::new(new_client));
                    }
                }
            }

            multi_client
        };

        let mut context: BotContext = multi_client.into();
        store.bot_context = Some(context.clone());
        self.chats_deck(id!(chats_deck)).sync_bot_contexts(scope);

        self.creating_bot_context = true;

        let ui = self.ui_runner();
        spawn(async move {
            context.load().await;
            ui.defer_with_redraw(move |me, _cx, _scope| {
                me.creating_bot_context = false;
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
            if !chat_view
                .messages(id!(chat.messages))
                .read()
                .messages
                .last()
                .unwrap()
                .is_writing
            {
                chat_view.chat(id!(chat)).write().bot_context = store.bot_context.clone();
            }
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
        bot_context: Option<BotContext>,
    ) {
        let mut chat_view_to_update;
        if let Some(chat_view) = self.chat_view_refs.get_mut(&chat.id) {
            chat_view.set_chat_id(chat.id);
            self.currently_visible_chat_id = Some(chat.id);

            chat_view_to_update = chat_view.clone();
        } else {
            let chat_view = WidgetRef::new_from_ptr(cx, self.chat_view_template);
            chat_view.chat(id!(chat)).write().bot_context = bot_context;
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

    fn sync_bot_contexts(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        for (_, chat_view) in self.chat_view_refs.iter_mut() {
            // Only set the BotContext if the chat is not currently streaming, otherwise it will be interrumpted.
            if !chat_view.chat(id!(chat)).read().is_streaming() {
                chat_view.chat(id!(chat)).write().bot_context = store.bot_context.clone();
            } else {
                self.chats_views_pending_sync.push(chat_view.clone());
            }
        }
    }
}

impl ChatsDeckRef {
    pub fn sync_bot_contexts(&mut self, scope: &mut Scope) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.sync_bot_contexts(scope);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreenMobile {
    #[deref]
    view: View,
}

impl Widget for ChatScreenMobile {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatScreenMobile {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(_evt) = self.view(id!(menu_toggle)).finger_down(actions) {
            self.modal(id!(chat_history_modal)).open(cx);
        }

        if let Some(_evt) = self.view(id!(settings_button)).finger_down(actions) {
            cx.action(NavigationAction::NavigateToProviders);
        }
    }
}
