use makepad_widgets::*;
use moly_kit::protocol::Picture;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use crate::chat::chats_deck::ChatsDeckWidgetExt;
use crate::data::providers::ProviderType;
use crate::data::store::Store;
use crate::settings::provider_view::ProviderViewWidgetExt;
use crate::settings::providers::ConnectionSettingsAction;
use crate::shared::actions::ChatAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::modal::*;
    use crate::shared::widgets::*;
    use crate::chat::chat_history_panel::ChatHistoryPanel;
    use crate::chat::chat_screen_mobile::ChatScreenMobile;
    use crate::chat::chats_deck::ChatsDeck;

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
                    chat_history_panel = <ChatHistoryPanel> {}
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

        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.button(id!(new_chat_button)).clicked(&actions) {
            cx.action(ChatAction::StartWithoutEntity);
            self.stack_navigation(id!(navigation)).pop_to_root(cx);
            self.redraw(cx);
        }

        for action in actions {
            if let ChatAction::ChatSelected(_chat_id) = action.cast() {
                self.stack_navigation(id!(navigation)).pop_to_root(cx);
                self.redraw(cx);
            }

            if let ConnectionSettingsAction::ProviderSelected(provider_id) = action.cast() {
                self.stack_navigation(id!(navigation))
                    .push(cx, live_id!(provider_navigation_view));

                let provider = scope
                    .data
                    .get_mut::<Store>()
                    .unwrap()
                    .chats
                    .providers
                    .get(&provider_id);
                if let Some(provider) = provider {
                    self.view
                        .provider_view(id!(provider_view))
                        .set_provider(cx, provider);
                } else {
                    eprintln!("Provider not found: {}", provider_id);
                }

                self.redraw(cx);
            }
        }
    }
}

impl ChatScreen {
    fn create_bot_context(&mut self, cx: &mut Cx, scope: &mut Scope) {
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
                            let mut client = OpenAIClient::new(provider.url.clone());
                            if let Some(key) = provider.api_key.as_ref() {
                                let _ = client.set_key(&key);
                            }

                            let mut client = MapClient::from(client);
                            if let Some(icon) = store.get_provider_icon(&provider.name) {
                                client.set_map_bots(move |mut bots| {
                                    for bot in bots.iter_mut() {
                                        bot.avatar = Picture::Dependency(icon.clone());
                                    }
                                    bots
                                });
                            }

                            multi_client.add_client(Box::new(client));
                        }
                    }
                    ProviderType::OpenAIImage => {
                        let client_url = provider.url.trim_start_matches('#').to_string();
                        let mut client = OpenAIImageClient::new(client_url);
                        if let Some(key) = provider.api_key.as_ref() {
                            let _ = client.set_key(&key);
                        }

                        multi_client.add_client(Box::new(client));
                    }
                    ProviderType::OpenAIRealtime => {
                        if provider.enabled && provider.api_key.is_some() {
                            let client_url = provider.url.trim_start_matches('#').to_string();
                            let mut client = OpenAIRealtimeClient::new(client_url);
                            if let Some(key) = provider.api_key.as_ref() {
                                let _ = client.set_key(&key);
                            }

                            multi_client.add_client(Box::new(client));
                        }
                    }
                    ProviderType::MoFa => {
                        // For MoFa we don't require an API key
                        if provider.enabled {
                            let mut client = OpenAIClient::new(provider.url.clone());
                            if let Some(key) = provider.api_key.as_ref() {
                                let _ = client.set_key(&key);
                            }

                            let mut client = MapClient::from(client);
                            if let Some(icon) = store.get_provider_icon(&provider.name) {
                                client.set_map_bots(move |mut bots| {
                                    for bot in bots.iter_mut() {
                                        bot.avatar = Picture::Dependency(icon.clone());
                                    }
                                    bots
                                });
                            }

                            multi_client.add_client(Box::new(client));
                        }
                    }
                    ProviderType::DeepInquire => {
                        let mut client = DeepInquireClient::new(provider.url.clone());
                        if let Some(key) = provider.api_key.as_ref() {
                            let _ = client.set_key(&key);
                        }

                        let mut client = MapClient::from(client);
                        if let Some(icon) = store.get_provider_icon(&provider.name) {
                            client.set_map_bots(move |mut bots| {
                                for bot in bots.iter_mut() {
                                    bot.avatar = Picture::Dependency(icon.clone());
                                }
                                bots
                            });
                        }

                        multi_client.add_client(Box::new(client));
                    }
                }
            }

            multi_client
        };

        let mut context: BotContext = multi_client.into();
        store.bot_context = Some(context.clone());
        self.chats_deck(id!(chats_deck))
            .sync_bot_contexts(cx, scope);

        self.creating_bot_context = true;

        let ui = self.ui_runner();
        spawn(async move {
            context.load().await;
        
            let mcp_manager = McpManagerClient::new();
            mcp_manager.add_server("test", McpTransport::Sse("http://localhost:8000/sse".to_string())).await.unwrap();
            context.set_tool_manager(mcp_manager);

            ui.defer_with_redraw(move |me, cx, scope| {
                me.creating_bot_context = false;

                // Update the bot_context with loaded bots and re-sync
                let store = scope.data.get_mut::<Store>().unwrap();
                store.bot_context = Some(context);
                me.chats_deck(id!(chats_deck)).sync_bot_contexts(cx, scope);
            });
        });
    }
}
