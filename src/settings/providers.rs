use makepad_widgets::*;

use crate::data::{chats::{ServerConnectionStatus, ServerType}, store::{ProviderType, Store}};

use super::configure_connection_modal::ConfigureConnectionModalAction;

live_design! {
    use link::widgets::*;
    use link::theme::*;
    use link::shaders::*;
    
    use crate::shared::widgets::*;
    use crate::shared::styles::*;

    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_TRASH = dep("crate://self/resources/images/trash_icon.png")
    ICON_REMOTE = dep("crate://self/resources/images/globe_icon.png")
    ICON_LOCAL = dep("crate://self/resources/images/laptop_icon.png")
    ICON_SETTINGS = dep("crate://self/resources/images/settings_icon.png")

    ICON_OPENAI = dep("crate://self/resources/images/providers/openai.png")

    ICON_SUCCESS = dep("crate://self/resources/images/circle_check_icon.png")
    ICON_LOADER = dep("crate://self/resources/images/loader_icon.png")
    ICON_FAILURE = dep("crate://self/resources/images/refresh_error_icon.png")

    // Not making this based on <Icon> because button does not support images
    // (and these SVGs are too complex for Makepad's SVG support)
    ConnectionActionButton = <View> {
        visible: false
        cursor: Hand
        width: Fit, height: Fit
        
        icon = <Image> {
            width: 22, height: 22
            // Override the color of the icon
            draw_bg: {
                instance tint_color: #B42318

                fn get_color_scale_pan(self, scale: vec2, pan: vec2) -> vec4 {
                    let tex_color = sample2d(self.image, self.pos * scale + pan).xyzw;
                    // Use the alpha channel from the texture but replace RGB with our tint color
                    // Assuming the icon is black/white with transparency
                    return vec4(
                        self.tint_color.rgb * tex_color.a,
                        tex_color.a
                    );
                }
            }
        }
    }

    ProviderItem = {{ProviderItem}} {
        width: Fill, height: 55
        flow: Overlay
        show_bg: true
        draw_bg: {
            color: #f
        }
        align: {x: 0.0, y: 0.5}

        // separator = <View> {
        //     margin: {left: 20, right: 20, top: 0, bottom: 10}
        //     height: 1,
        //     show_bg: true,
        //     draw_bg: {
        //         color: #D9D9D9
        //     }
        // }
    
        main_view = <View> {
            cursor: Hand
            padding: 10
            align: {x: 0.0, y: 0.5}
            spacing: 20
            flow: Right

            provider_icon = <View> {
                width: Fit, height: Fit
                visible: true
                <Image> {
                    source: (ICON_OPENAI) // TODO: replace with the icon of the provider
                    width: 25, height: 25
                }
            }

            // icon_local = <View> {
            //     width: Fit, height: Fit
            //     visible: false
            //     <Image> {
            //         source: (ICON_LOCAL)
            //         width: 18, height: 18
            //     }
            // }
    
            <View> {
                width: Fill, height: Fill
                spacing: 20
                align: {x: 0.0, y: 0.5}
                // server_address_label = <Label> {
                //     draw_text:{
                //         text_style: <REGULAR_FONT>{font_size: 12}
                //         color: #000
                //     }
                // }

                provider_name_label = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 12}
                        color: #000
                    }
                }

                // <VerticalFiller> {}

                // connection_status_success = <ConnectionActionButton> {
                //     icon = {
                //         source: (ICON_SUCCESS)
                //         draw_bg: {
                //             tint_color: #099250
                //         }
                //     }
                // }

                // connection_status_failure = <ConnectionActionButton> {
                //     icon = {
                //         source: (ICON_FAILURE)
                //         draw_bg: {
                //             tint_color: #B42318
                //         }
                //     }
                // }

                // connection_status_loading = <ConnectionActionButton> {
                //     visible: true
                //     icon = {
                //         source: (ICON_LOADER)
                //         draw_bg: {
                //             tint_color: #FF8C00
                //         }
                //     }
                // }

                // configure_connection = <ConnectionActionButton> {
                //     visible: true
                //     icon = {
                //         source: (ICON_SETTINGS)
                //         draw_bg: {
                //             tint_color: #444
                //         }
                //     }
                // }

                // remove_server = <ConnectionActionButton> {
                //     visible: true
                //     icon = {
                //         source: (ICON_TRASH)
                //         draw_bg: {
                //             tint_color: #B42318
                //         }
                //     }
                // }
            }

        }

        // configure_connection_modal = <Modal> {
        //     content: {
        //         configure_connection_modal_inner = <ConfigureConnectionModal> {}
        //     }
        // }
    }

    // delete_modal = <Modal> {
    //     content: {
    //         <DeleteServerModal> {}
    //     }
    // }

    pub Providers = {{Providers}} {
        width: 400, height: Fill
        flow: Down, spacing: 20
        providers_list = <PortalList> {
            width: Fill, height: Fill
            provider_item = <ProviderItem> {}
        }
    }   
}

#[derive(Widget, LiveHook, Live)]
struct Providers {
    #[deref]
    view: View,
}

impl Widget for Providers {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();

        // We load the supported providers from the JSON file, and merge them with the servers from the store.
        // The supported providers are always shown in the list.
        // ConnectionView then shows a specific provider configuration, using the API key and model config from the store.
        // read the file in supported_providers.json
        // let supported_providers = include_str!("supported_providers.json");
        // let supported_providers: Vec<ProviderType> = serde_json::from_str(supported_providers).unwrap();
        // println!("Supported providers: {:?}", supported_providers);

        // Collect Mofa servers from store
        let mut mofa_servers: Vec<_> = store.chats.mofa_servers.values().cloned().map(|m| Provider {
            name: m.client.address.clone(),
            url: m.client.address.clone(),
            api_key: None,
            provider_type: ProviderType::MoFa,
            connection_status: m.connection_status.clone(),
        }).collect();

        // Collect OpenAI servers from store
        let mut openai_servers: Vec<_> = store.chats.openai_servers.values().cloned().map(|o| Provider {
            // name: o.address.clone(), // TODO: Fetch the actual name here
            name: "OpenAI".to_string(),
            url: o.address.clone(),
            api_key: o.api_key.clone(),
            provider_type: ProviderType::OpenAIAPI,
            connection_status: o.connection_status.clone(),
        }).collect();

        // Combine them
        let mut all_providers = Vec::new();
        all_providers.append(&mut mofa_servers);
        all_providers.append(&mut openai_servers);

        // Sort them by name
        all_providers.sort_by(|a, b| a.name.cmp(&b.name));

        let entries_count = all_providers.len();

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, entries_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < entries_count {
                        let template = live_id!(provider_item);
                        let item = list.item(cx, item_id, template);

                        // hide the separator for the first item
                        if item_id == 0 {
                            item.view(id!(separator)).set_visible(cx, false);
                        }

                        let provider = all_providers[item_id].clone();
                        item.as_provider_item().set_provider(provider);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for Providers {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let address = self.view.text_input(id!(add_server_input)).text();
        let api_key_input = self.view.text_input(id!(api_key));
        let provider_type = self.view.drop_down(id!(provider_type));
        let add_server_button = self.view.button(id!(add_server_button));

        if add_server_button.clicked(actions) {
            let provider = ProviderType::from_usize(provider_type.selected_item());
            match provider {
                ProviderType::OpenAIAPI => {
                    store.chats.register_server(ServerType::OpenAI {
                        address: address.clone(),
                        api_key: api_key_input.text(),
                    });
                }
                ProviderType::MoFa => {
                    store.chats.register_server(ServerType::Mofa(address.clone()));
                }
            }

            // Persist to preferences:
            store.preferences.add_or_update_server_connection(
                provider,
                address.clone(),
                Some(api_key_input.text()).filter(|s| !s.is_empty()),
            );

            // Clear the form fields:
            self.view.text_input(id!(add_server_input)).set_text(cx, "");
            api_key_input.set_text(cx, "");

            self.redraw(cx);
        }
    }
}

// /// A small enum to unify the idea of either a Mofa server or an OpenAI server.
// #[derive(Clone, Debug)]
// pub enum AiServerEntry {
//     Mofa(MofaServer),
//     OpenAI(OpenAIClient),
// }

#[derive(Clone, Debug, Default)]
pub struct Provider {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub provider_type: ProviderType,
    pub connection_status: ServerConnectionStatus,
}

// impl AiServerEntry {
//     fn address(&self) -> String {
//         match self {
//             AiServerEntry::Mofa(m) => m.client.address.clone(),
//             AiServerEntry::OpenAI(o) => o.address.clone(),
//         }
//     }
// }

#[derive(Widget, LiveHook, Live)]
struct ProviderItem {
    #[deref]
    view: View,

    #[rust]
    provider: Provider,
}

impl Widget for ProviderItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Extract address and status
        // let (address, connection_status, is_local, _is_mofa) = match &self.server_entry {
        //     Some(AiServerEntry::Mofa(ref m)) => {
        //         (m.client.address.clone(), m.connection_status.clone(), m.is_local(), true)
        //     }
        //     Some(AiServerEntry::OpenAI(ref o)) => {
        //         (o.address.clone(), o.connection_status.clone(), false, false)
        //     }
        //     None => {
        //         return DrawStep::done();
        //     }
        // };


        // self.provider.url = address.clone();

        // Show/hide local icon
        // if is_local {
        //     self.view.view(id!(icon_local)).set_visible(cx, true);
        //     self.view.view(id!(icon_remote)).set_visible(cx, false);
        // } else {
        //     self.view.view(id!(icon_local)).set_visible(cx, false);
        //     self.view.view(id!(icon_remote)).set_visible(cx, true);
        // }

        // Update the label
        self.label(id!(provider_name_label))
            .set_text(cx, &self.provider.name);

        let connection_status = self.provider.connection_status.clone();
        // Show connection status icons
        self.update_connection_status(cx, &connection_status);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ProviderItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        // "Remove server" click
        // TODO: Use the modal instead. Currently broken and needs refactoring.
        let remove_server_was_clicked = self.view(id!(remove_server)).finger_up(actions).is_some();
        if remove_server_was_clicked {
            match &self.provider.provider_type {
                ProviderType::MoFa => {
                    store.chats.remove_mofa_server(&self.provider.url);
                    store.preferences.remove_server_connection(&self.provider.url);
                }
                ProviderType::OpenAIAPI => {
                    store.chats.remove_openai_server(&self.provider.url);
                    store.preferences.remove_server_connection(&self.provider.url);
                }
            }
            self.redraw(cx);
        }

        // Re-test connection if the user clicks on the "failure" status
        if let Some(_) = self.view(id!(connection_status_failure)).finger_down(actions) {
            match &self.provider.provider_type {
                ProviderType::MoFa => {
                    store.chats.test_mofa_server_and_fetch_agents(&self.provider.url);
                    self.update_connection_status(cx, &ServerConnectionStatus::Connecting);
                }
                ProviderType::OpenAIAPI => {
                    // For OpenAI servers:
                    store.chats.test_openai_server_and_fetch_models(&self.provider.url);
                    self.update_connection_status(cx, &ServerConnectionStatus::Connecting);
                }
            }
            self.redraw(cx);
        }

        let was_item_clicked = self.view(id!(main_view)).finger_up(actions).is_some();
        // let was_configure_connection_clicked = self.view(id!(configure_connection)).finger_down(actions).is_some();
        if was_item_clicked {
            cx.action(ConnectionSettingsAction::ProviderSelected(self.provider.url.clone()));
        }
        // if was_configure_connection_clicked || was_item_clicked {
        //     if let Some(entry) = &self.server_entry {
        //         let (address, _provider) = match entry {
        //             AiServerEntry::Mofa(m) => (m.client.address.clone(), ProviderType::MoFa),
        //             AiServerEntry::OpenAI(o) => (o.address.clone(), ProviderType::OpenAIAPI),
        //         };

        //         self.view.configure_connection_modal(id!(configure_connection_modal_inner))
        //             .set_server_address(address);

        //         self.modal(id!(configure_connection_modal)).open(cx);
        //     }
        // }

        // Handle modal actions
        for action in actions {
            if matches!(
                action.cast(),
                ConfigureConnectionModalAction::ModalDismissed
            ) {
                self.modal(id!(configure_connection_modal)).close(cx);
                self.redraw(cx);
            }
        }
    }
}

impl ProviderItem {
    /// Toggles the visibility of the connection status icons
    fn update_connection_status(
        &mut self,
        cx: &mut Cx,
        connection_status: &ServerConnectionStatus
    ) {
        self.view(id!(connection_status_success))
            .set_visible(cx, *connection_status == ServerConnectionStatus::Connected);
        self.view(id!(connection_status_failure))
            .set_visible(cx, *connection_status == ServerConnectionStatus::Disconnected);
        self.view(id!(connection_status_loading))
            .set_visible(cx, *connection_status == ServerConnectionStatus::Connecting);
    }
}

impl ProviderItemRef {
    fn set_provider(&mut self, provider: Provider) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.provider = provider;
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ConnectionSettingsAction {
    None,
    ProviderSelected(String),
}
