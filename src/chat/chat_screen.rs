use makepad_widgets::*;
use moly_kit::protocol::Picture;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use crate::data::bot_fetcher::should_include_model;
use crate::data::providers::ProviderType;
use crate::data::store::Store;
use crate::data::supported_providers;
use crate::settings::provider_view::ProviderViewWidgetExt;
use crate::settings::providers::ConnectionSettingsAction;
use crate::shared::actions::ChatAction;
use crate::shared::bot_context::BotContext;

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
        if self.button(ids!(new_chat_button)).clicked(&actions) {
            cx.action(ChatAction::StartWithoutEntity);
            self.stack_navigation(ids!(navigation)).pop_to_root(cx);
            self.redraw(cx);
        }

        for action in actions {
            if let ChatAction::ChatSelected(_chat_id) = action.cast() {
                self.stack_navigation(ids!(navigation)).pop_to_root(cx);
                self.redraw(cx);
            }

            if let ConnectionSettingsAction::ProviderSelected(provider_id) = action.cast() {
                self.stack_navigation(ids!(navigation))
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
                        .provider_view(ids!(provider_view))
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
    fn create_bot_context(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let multi_client = {
            let mut multi_client = MultiClient::new();
            let supported_providers_list = supported_providers::load_supported_providers();

            // Clone store data for use in MapClient closures to check enabled status
            let available_bots = store.chats.available_bots.clone();
            let providers = store.chats.providers.clone();

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
                            client.set_tools_enabled(provider.tools_enabled);

                            let mut client = MapClient::from(client);

                            // Clone supported models for this provider (if any)
                            let supported_models = supported_providers_list
                                .iter()
                                .find(|sp| sp.id == provider.id)
                                .and_then(|sp| sp.supported_models.clone());

                            let icon_opt = store.get_provider_icon(&provider.name);
                            let available_bots_clone = available_bots.clone();
                            let providers_clone = providers.clone();
                            client.set_map_bots(move |mut bots| {
                                // Filter by provider enabled status only
                                // Keep all bots (including disabled ones) so historical messages can display bot names
                                if !available_bots_clone.is_empty() {
                                    bots.retain(|bot| {
                                        if let Some(provider_bot) =
                                            available_bots_clone.get(&bot.id)
                                        {
                                            providers_clone
                                                .get(&provider_bot.provider_id)
                                                .map_or(false, |p| p.enabled)
                                        } else {
                                            // Bot not in available_bots yet, let it through
                                            true
                                        }
                                    });
                                }

                                // Apply basic filter (non-chat models)
                                bots.retain(|bot| should_include_model(&bot.name));

                                // Apply supported models whitelist if available
                                if let Some(ref models) = supported_models {
                                    bots.retain(|bot| models.contains(&bot.name));
                                }

                                // Set icon if available
                                if let Some(ref icon) = icon_opt {
                                    for bot in bots.iter_mut() {
                                        bot.avatar = Picture::Dependency(icon.clone());
                                    }
                                }
                                bots
                            });

                            multi_client.add_client(Box::new(client));
                        }
                    }
                    ProviderType::OpenAIImage => {
                        let client_url = provider.url.trim_start_matches('#').to_string();
                        let mut client = OpenAIImageClient::new(client_url);
                        if let Some(key) = provider.api_key.as_ref() {
                            let _ = client.set_key(&key);
                        }

                        let mut client = MapClient::from(client);

                        let icon_opt = store.get_provider_icon(&provider.name);
                        let available_bots_clone = available_bots.clone();
                        let providers_clone = providers.clone();
                        client.set_map_bots(move |mut bots| {
                            // Filter by enabled status only if bot exists in available_bots
                            // If available_bots is empty (initial load), let bots through
                            if !available_bots_clone.is_empty() {
                                bots.retain(|bot| {
                                    if let Some(provider_bot) = available_bots_clone.get(&bot.id) {
                                        provider_bot.enabled
                                            && providers_clone
                                                .get(&provider_bot.provider_id)
                                                .map_or(false, |p| p.enabled)
                                    } else {
                                        // Bot not in available_bots yet, let it through
                                        true
                                    }
                                });
                            }

                            // Set icon if available
                            if let Some(ref icon) = icon_opt {
                                for bot in bots.iter_mut() {
                                    bot.avatar = Picture::Dependency(icon.clone());
                                }
                            }
                            bots
                        });

                        multi_client.add_client(Box::new(client));
                    }
                    ProviderType::OpenAIRealtime => {
                        let is_local = provider.url.contains("127.0.0.1")
                            || provider.url.contains("localhost");
                        if provider.enabled && (is_local || provider.api_key.is_some()) {
                            let client_url = provider.url.trim_start_matches('#').to_string();
                            let mut client = OpenAIRealtimeClient::new(client_url);
                            if let Some(key) = provider.api_key.as_ref() {
                                let _ = client.set_key(&key);
                            }
                            if let Some(prompt) = provider.system_prompt.as_ref() {
                                let _ = client.set_system_prompt(&prompt);
                            }
                            client.set_tools_enabled(provider.tools_enabled);

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
                            client.set_tools_enabled(provider.tools_enabled);

                            let mut client = MapClient::from(client);

                            // Clone supported models for this provider (if any)
                            let supported_models = supported_providers_list
                                .iter()
                                .find(|sp| sp.id == provider.id)
                                .and_then(|sp| sp.supported_models.clone());

                            let icon_opt = store.get_provider_icon(&provider.name);
                            let available_bots_clone = available_bots.clone();
                            let providers_clone = providers.clone();
                            client.set_map_bots(move |mut bots| {
                                // Filter by provider enabled status only
                                // Keep all bots (including disabled ones) so historical messages can display bot names
                                if !available_bots_clone.is_empty() {
                                    bots.retain(|bot| {
                                        if let Some(provider_bot) =
                                            available_bots_clone.get(&bot.id)
                                        {
                                            providers_clone
                                                .get(&provider_bot.provider_id)
                                                .map_or(false, |p| p.enabled)
                                        } else {
                                            // Bot not in available_bots yet, let it through
                                            true
                                        }
                                    });
                                }

                                // Apply basic filter (non-chat models)
                                bots.retain(|bot| should_include_model(&bot.name));

                                // Apply supported models whitelist if available
                                if let Some(ref models) = supported_models {
                                    bots.retain(|bot| models.contains(&bot.name));
                                }

                                // Set icon if available
                                if let Some(ref icon) = icon_opt {
                                    for bot in bots.iter_mut() {
                                        bot.avatar = Picture::Dependency(icon.clone());
                                    }
                                }
                                bots
                            });

                            multi_client.add_client(Box::new(client));
                        }
                    }
                    ProviderType::DeepInquire => {
                        let mut client = DeepInquireClient::new(provider.url.clone());
                        if let Some(key) = provider.api_key.as_ref() {
                            let _ = client.set_key(&key);
                        }

                        let mut client = MapClient::from(client);

                        // Clone supported models for this provider (if any)
                        let supported_models = supported_providers_list
                            .iter()
                            .find(|sp| sp.id == provider.id)
                            .and_then(|sp| sp.supported_models.clone());

                        let icon_opt = store.get_provider_icon(&provider.name);
                        let available_bots_clone = available_bots.clone();
                        let providers_clone = providers.clone();
                        client.set_map_bots(move |mut bots| {
                            // Filter by provider enabled status only
                            // Keep all bots (including disabled ones) so historical messages can display bot names
                            if !available_bots_clone.is_empty() {
                                bots.retain(|bot| {
                                    if let Some(provider_bot) = available_bots_clone.get(&bot.id) {
                                        providers_clone
                                            .get(&provider_bot.provider_id)
                                            .map_or(false, |p| p.enabled)
                                    } else {
                                        // Bot not in available_bots yet, let it through
                                        true
                                    }
                                });
                            }

                            // No basic filter for DeepInquire, just apply supported models whitelist
                            if let Some(ref models) = supported_models {
                                bots.retain(|bot| models.contains(&bot.name));
                            }

                            // Set icon if available
                            if let Some(ref icon) = icon_opt {
                                for bot in bots.iter_mut() {
                                    bot.avatar = Picture::Dependency(icon.clone());
                                }
                            }
                            bots
                        });

                        multi_client.add_client(Box::new(client));
                    }
                }
            }

            multi_client
        };

        let mut context: BotContext = multi_client.into();
        let tool_manager = store.create_and_load_mcp_tool_manager();
        tool_manager
            .set_dangerous_mode_enabled(store.preferences.get_mcp_servers_dangerous_mode_enabled());
        context.set_tool_manager(tool_manager);

        store.bot_context = Some(context.clone());

        self.creating_bot_context = true;

        let ui = self.ui_runner();
        spawn(async move {
            let _ = context.load().await;
            ui.defer_with_redraw(move |me, _cx, _scope| {
                me.creating_bot_context = false;
            });
        });
    }
}
