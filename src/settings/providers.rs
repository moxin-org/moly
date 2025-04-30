use makepad_widgets::*;
use crate::data::{providers::{Provider, ProviderConnectionStatus}, store::Store};
use crate::shared::modal::ModalWidgetExt;

use super::{add_provider_modal::AddProviderModalAction, provider_view::ProviderViewAction};

live_design! {
    use link::widgets::*;
    use link::theme::*;
    use link::shaders::*;
    
    use crate::shared::widgets::*;
    use crate::shared::styles::*;
    use crate::settings::add_provider_modal::*;
    use crate::shared::modal::*;

    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_TRASH = dep("crate://self/resources/images/trash_icon.png")
    ICON_REMOTE = dep("crate://self/resources/images/globe_icon.png")
    ICON_LOCAL = dep("crate://self/resources/images/laptop_icon.png")
    ICON_SETTINGS = dep("crate://self/resources/images/settings_icon.png")

    ICON_SUCCESS = dep("crate://self/resources/images/circle_check_icon.png")
    ICON_LOADER = dep("crate://self/resources/images/loader_icon.png")
    ICON_FAILURE = dep("crate://self/resources/images/refresh_error_icon.png")

    // Provider icons
    ICON_OPENAI = dep("crate://self/resources/images/providers/openai.png")
    ICON_GEMINI = dep("crate://self/resources/images/providers/gemini.png")
    ICON_MOFA = dep("crate://self/resources/images/providers/mofa.png")
    ICON_SILICONFLOW = dep("crate://self/resources/images/providers/siliconflow.png")
    ICON_OPENROUTER = dep("crate://self/resources/images/providers/openrouter.png")
    ICON_DEEPINQUIRE = dep("crate://self/resources/images/providers/deepinquire.png")
    ICON_MOLYSERVER = dep("crate://self/resources/images/providers/molyserver.png")
    ICON_CUSTOM_PROVIDER = dep("crate://self/resources/images/providers/custom.png")

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

    ProviderItem = {{ProviderItem}}<RoundedView> {
        width: Fill, height: 40
        flow: Overlay
        show_bg: true
        draw_bg: {
            border_radius: 5
        }
        padding: {left: 30}
        align: {x: 0.0, y: 0.5}

        main_view = <View> {
            cursor: Hand
            padding: 8
            align: {x: 0.0, y: 0.5}
            spacing: 20
            flow: Right

            provider_icon = <View> {
                width: Fit, height: Fit
                visible: true
                provider_icon_image = <Image> {
                    width: 25, height: 25
                }
            }

    
            <View> {
                flow: Right
                width: Fill, height: Fill
                spacing: 20
                align: {x: 0.0, y: 0.5}

                provider_name_label = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 11}
                        color: #000
                    }
                }

                filler = <View> { width: Fill, height: Fill }
            
                status_view = <RoundedView> {
                    align: {x: 0.5, y: 0.5}
                    show_bg: true
                    width: Fit, height: Fit
                    padding: {left: 8, right: 8, bottom: 5, top: 5}
                    margin: {right: 10}
                    draw_bg: {
                        border_radius: 5
                        color: #9FD5C7
                        border_color: #357852
                        border_size: 1.2
                    }
                    status_label = <Label> {
                        text: "ON"
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 7},
                            color: #043b1c
                        }
                    }
                }
            }

        }

    }

    pub Providers = {{Providers}} {
        width: 300, height: Fill
        flow: Down, spacing: 20
        padding: {left: 10, right: 10}
        providers_list = <PortalList> {
            width: Fill, height: Fill
            provider_item = <ProviderItem> {}
        }

        add_provider_button = <RoundedShadowView> {
            cursor: Hand
            margin: {left: 10, right: 10, bottom: 5, top: 10}
            width: Fill, height: Fit
            align: {x: 0.5, y: 0.5}
            padding: {left: 30, right: 30, bottom: 10, top: 10}
            draw_bg: { 
                color: (MAIN_BG_COLOR)
                border_radius: 4.5,
                uniform shadow_color: #0002
                shadow_radius: 8.0,
                shadow_offset: vec2(0.0,-1.5)
            }
            <Label> {
                text: "+"
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 15}
                    color: #000
                }
            }
        }

        provider_icons: [
            (ICON_CUSTOM_PROVIDER),
            (ICON_OPENAI),
            (ICON_GEMINI),
            (ICON_MOFA),
            (ICON_SILICONFLOW),
            (ICON_OPENROUTER),
            (ICON_DEEPINQUIRE),
            (ICON_MOLYSERVER),
        ]

        <View> {
            width: Fill, height: Fit
            flow: Overlay

            add_provider_modal = <Modal> {
                content: {
                    add_provider_modal_inner = <AddProviderModal> {}
                }
            }
        }
    }   
}

#[derive(Widget, Live)]
struct Providers {
    #[deref]
    view: View,

    #[live]
    provider_icons: Vec<LiveDependency>,
    #[rust]
    selected_provider: Option<String>,
}

impl LiveHook for Providers {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        let default_provider_url = "https://api.siliconflow.cn/v1".to_string();
        self.selected_provider = Some(default_provider_url.clone());
        cx.action(ConnectionSettingsAction::ProviderSelected(default_provider_url));
    }
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
                        let icon = self.get_provider_icon(&provider);
                        let is_selected = self.selected_provider == Some(provider.url.clone());
                        item.as_provider_item().set_provider(cx, provider, icon, is_selected);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl Providers {
    fn get_provider_icon(&self, provider: &Provider) -> LiveDependency {
        // TODO: a more robust, less horrible way to programatically swap icons that are loaded as live dependencies
        // Find a path that contains the provider name
        self.provider_icons.iter().find(|icon| icon.as_str().to_lowercase().contains(&provider.name.to_lowercase()))
        .unwrap_or(&self.provider_icons[0]).clone()
    }
}

impl WidgetMatchEvent for Providers {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // Handle modal open
        if self.view(id!(add_provider_button)).finger_up(actions).is_some() {
            let modal = self.modal(id!(add_provider_modal));
            modal.open(cx);
        }

        for action in actions {
            // Handle selected provider
            if let ConnectionSettingsAction::ProviderSelected(provider_url) = action.cast() {
                self.selected_provider = Some(provider_url);
            }

            // Handle modal actions
            if let AddProviderModalAction::ModalDismissed = action.cast() {
                self.modal(id!(add_provider_modal)).close(cx);
                self.redraw(cx);
            }

            // Handle provider removed
            if let ProviderViewAction::ProviderRemoved = action.cast() {
                // Select another provider
                let store = scope.data.get::<Store>().unwrap();
                if let Some(first_provider) = store.chats.providers.values().next() {
                    self.selected_provider = Some(first_provider.url.clone());
                    cx.action(ConnectionSettingsAction::ProviderSelected(first_provider.url.clone()));
                }
                self.redraw(cx);
            }
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

        self.view(id!(status_view))
            .set_visible(cx, connection_status == ProviderConnectionStatus::Connected && self.provider.enabled);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ProviderItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
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
        connection_status: &ProviderConnectionStatus
    ) {
        self.view(id!(connection_status_success))
            .set_visible(cx, *connection_status == ProviderConnectionStatus::Connected);
        self.view(id!(connection_status_failure))
            .set_visible(cx, *connection_status == ProviderConnectionStatus::Disconnected);
        self.view(id!(connection_status_loading))
            .set_visible(cx, *connection_status == ProviderConnectionStatus::Connecting);
    }
}

impl ProviderItemRef {
    fn set_provider(&mut self, cx: &mut Cx, provider: Provider, icon_path: LiveDependency, is_selected: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.provider = provider;
        let _ = inner.image(id!(provider_icon_image))
        .load_image_dep_by_path(cx, icon_path.as_str());

        if is_selected {
            inner.view.apply_over(cx, live! {
                draw_bg: { color: #EAECEF }
            });
        } else {
            inner.view.apply_over(cx, live! {
                draw_bg: { color: #f9f9f9 }
            });
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ConnectionSettingsAction {
    None,
    ProviderSelected(String),
}
