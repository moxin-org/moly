use makepad_widgets::*;
use crate::data::{chats::{Provider, ServerConnectionStatus}, store::{ProviderType, Store}};

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

        let mut all_providers: Vec<Provider> = store.chats.providers.values().cloned().collect();
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

        // TODO(Julian): this will be replaced by a modal that allows for custom providers
        // most providers will be already listed by default from a JSON file (and synced with the preferences, store, and remote server)
        if add_server_button.clicked(actions) {
            let provider_type = ProviderType::from_usize(provider_type.selected_item());
            let provider = match provider_type {
                ProviderType::OpenAIAPI => {
                    Provider {
                        name: "OpenAI".to_string(),
                        url: address.clone(),
                        api_key: Some(api_key_input.text()),
                        provider_type: ProviderType::OpenAIAPI,
                        connection_status: ServerConnectionStatus::Disconnected,
                        models: vec![],
                    }
                }
                ProviderType::MoFa => {
                    Provider {
                        name: "MoFa".to_string(),
                        url: address.clone(),
                        api_key: None,
                        provider_type: ProviderType::MoFa,
                        connection_status: ServerConnectionStatus::Disconnected,
                        models: vec![],
                    }
                }
            };
            // Add to memory
            store.chats.register_provider(provider.clone());

            // Persist to preferences
            store.preferences.add_or_update_provider(provider);

            // Clear the form fields:
            self.view.text_input(id!(add_server_input)).set_text(cx, "");
            api_key_input.set_text(cx, "");

            self.redraw(cx);
        }
    }
}

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
                    store.preferences.remove_provider(&self.provider.url);
                }
                ProviderType::OpenAIAPI => {
                    store.chats.remove_openai_server(&self.provider.url);
                    store.preferences.remove_provider(&self.provider.url);
                }
            }
            self.redraw(cx);
        }

        // Re-test connection if the user clicks on the "failure" status
        if let Some(_) = self.view(id!(connection_status_failure)).finger_down(actions) {
            store.chats.test_provider_and_fetch_models(&self.provider.url);
            self.update_connection_status(cx, &ServerConnectionStatus::Connecting);
            self.redraw(cx);
        }

        let was_item_clicked = self.view(id!(main_view)).finger_up(actions).is_some();
        if was_item_clicked {
            cx.action(ConnectionSettingsAction::ProviderSelected(self.provider.url.clone()));
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
