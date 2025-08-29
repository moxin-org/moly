use crate::shared::modal::ModalWidgetExt;
use crate::{
    data::{
        providers::{Provider, ProviderConnectionStatus},
        store::Store,
    },
    settings::sync_modal::{SyncModalAction, SyncModalWidgetExt},
};
use makepad_widgets::*;

use super::{add_provider_modal::AddProviderModalAction, provider_view::ProviderViewAction};

live_design! {
    use link::widgets::*;
    use link::theme::*;
    use link::shaders::*;

    use crate::shared::widgets::*;
    use crate::shared::styles::*;
    use crate::settings::add_provider_modal::*;
    use crate::settings::sync_modal::SyncModal;
    use crate::shared::modal::*;

    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_TRASH = dep("crate://self/resources/images/trash_icon.png")
    ICON_REMOTE = dep("crate://self/resources/images/globe_icon.png")
    ICON_LOCAL = dep("crate://self/resources/images/laptop_icon.png")

    ICON_SUCCESS = dep("crate://self/resources/images/circle_check_icon.png")
    ICON_LOADER = dep("crate://self/resources/images/loader_icon.png")
    ICON_FAILURE = dep("crate://self/resources/images/refresh_error_icon.png")

    // Provider icons
    ICON_OPENAI = dep("crate://self/resources/images/providers/openai.png")
    ICON_GEMINI = dep("crate://self/resources/images/providers/gemini.png")
    ICON_SILICONFLOW = dep("crate://self/resources/images/providers/siliconflow.png")
    ICON_OPENROUTER = dep("crate://self/resources/images/providers/openrouter.png")
    ICON_MOLYSERVER = dep("crate://self/resources/images/providers/molyserver.png")
    ICON_DEEPSEEK = dep("crate://self/resources/images/providers/deepseek.png")
    ICON_OLLAMA = dep("crate://self/resources/images/providers/ollama.png")
    ICON_ANTHROPIC = dep("crate://self/resources/images/providers/anthropic.png")

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
        padding: {left: 20}
        align: {x: 0.0, y: 0.5}

        main_view = <View> {
            cursor: Hand
            padding: 8
            align: {x: 0.0, y: 0.5}
            spacing: 20
            flow: Right

            provider_icon = <View> {
                width: Fit, height: Fit
                image_wrapper = <View> {
                    width: Fit, height: Fit
                    provider_icon_image = <Image> {
                        width: 25, height: 25
                    }
                    visible: true
                }

                label_wrapper = <RoundedView> {
                    width: 25, height: 25
                    visible: false
                    show_bg: true
                    draw_bg: {
                        color: #344054
                        border_radius: 6
                    }
                    align: {x: 0.5, y: 0.5}

                    initial_label = <Label> {
                        draw_text:{
                            text_style: <BOLD_FONT>{font_size: 12}
                            color: #f
                        }
                    }
                }
            }


            <View> {
                flow: Right
                width: Fill, height: Fill
                spacing: 20
                align: {x: 0.0, y: 0.5}

                provider_name_label = <Label> {
                    flow: Right,
                    width: Fill,
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 11}
                        color: #000
                    }
                }

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
        flow: Down, spacing: 10
        padding: {left: 10, right: 10}
        providers_list = <PortalList> {
            width: Fill, height: Fill
            provider_item = <ProviderItem> {}
        }

        add_provider_button = <RoundedShadowView> {
            cursor: Hand
            margin: {left: 10, right: 10, bottom: 0, top: 10}
            width: Fill, height: Fit
            align: {x: 0.5, y: 0.5}
            padding: {left: 30, right: 30, bottom: 15, top: 15}
            draw_bg: {
                color: (MAIN_BG_COLOR)
                border_radius: 4.5,
                uniform shadow_color: #0002
                shadow_radius: 8.0,
                shadow_offset: vec2(0.0,-1.5)
            }
            <Label> {
                text: "+ Add a Custom Provider"
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 11}
                    color: #000
                }
            }
        }

        open_sync_button = <RoundedShadowView> {
            cursor: Hand
            margin: {left: 10, right: 10, bottom: 0}
            width: Fill, height: Fit
            align: {x: 0.5, y: 0.5}
            padding: {left: 30, right: 30, bottom: 15, top: 15}
            draw_bg: {
                color: (MAIN_BG_COLOR)
                border_radius: 4.5,
                uniform shadow_color: #0002
                shadow_radius: 8.0,
                shadow_offset: vec2(0.0,-1.5)
            }
            <Label> {
                text: "Sync Settings"
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 11}
                    color: #000
                }
            }
        }

        provider_icons: [
            (ICON_OPENAI),
            (ICON_GEMINI),
            (ICON_SILICONFLOW),
            (ICON_OPENROUTER),
            (ICON_MOLYSERVER),
            (ICON_DEEPSEEK),
            (ICON_OLLAMA),
            (ICON_ANTHROPIC),
        ]

        <View> {
            width: Fill, height: Fit
            flow: Overlay

            add_provider_modal = <Modal> {
                content: {
                    add_provider_modal_inner = <AddProviderModal> {}
                }
            }

            sync_modal = <Modal> {
                content: {
                    sync_modal_inner = <SyncModal> {}
                }
            }
        }
    }
}

#[derive(Widget, Live, LiveHook)]
struct Providers {
    #[deref]
    view: View,

    #[live]
    provider_icons: Vec<LiveDependency>,
    #[rust]
    selected_provider_id: Option<String>,

    #[rust]
    initialized: bool,
}

impl Widget for Providers {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // `after_new_from_doc` will run although the whole app is set to invisible
        // so the event will not be received.
        //
        // I think this demostrates that `after_new_from_doc != initialize`.
        if !self.initialized {
            if cx.display_context.is_desktop() {
                self.initialized = true;
                let default_provider_id = "anthropic".to_string();
                self.selected_provider_id = Some(default_provider_id.clone());

                cx.action(ConnectionSettingsAction::ProviderSelected(
                    default_provider_id,
                ));
            }
        }

        let store = scope.data.get_mut::<Store>().unwrap();
        if store.provider_icons.is_empty() {
            store.provider_icons = self.provider_icons.clone();
        }
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
                        let is_selected = self.selected_provider_id == Some(provider.id.clone());
                        item.as_provider_item()
                            .set_provider(cx, provider, icon, is_selected);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl Providers {
    fn get_provider_icon(&self, provider: &Provider) -> Option<LiveDependency> {
        // TODO: a more robust, less horrible way to programatically swap icons that are loaded as live dependencies
        // Find a path that contains the provider name
        self.provider_icons
            .iter()
            .find(|icon| {
                icon.as_str()
                    .to_lowercase()
                    .contains(&provider.name.to_lowercase())
            })
            .cloned()
    }
}

impl WidgetMatchEvent for Providers {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // Handle modal open
        if self
            .view(id!(add_provider_button))
            .finger_up(actions)
            .is_some()
        {
            let modal = self.modal(id!(add_provider_modal));
            modal.open(cx);
        }

        if self
            .view(id!(open_sync_button))
            .finger_up(actions)
            .is_some()
        {
            let modal = self.modal(id!(sync_modal));
            modal.open(cx);
        }

        for action in actions {
            // Handle selected provider
            if let ConnectionSettingsAction::ProviderSelected(provider_id) = action.cast() {
                self.selected_provider_id = Some(provider_id);
            }

            // Handle modal actions
            if let AddProviderModalAction::ModalDismissed = action.cast() {
                self.modal(id!(add_provider_modal)).close(cx);
                self.redraw(cx);
            }

            if let SyncModalAction::ModalDismissed = action.cast() {
                self.modal(id!(sync_modal)).close(cx);
                self.redraw(cx);
            }

            // Handle the case where the modal is dismissed by the user clicking outside the modal
            // This is a hacky way to reset the modal state because the inner content never gets to
            // hear if it was dismissed from outside.
            if self.modal(id!(sync_modal)).dismissed(actions) {
                self.sync_modal(id!(sync_modal_inner)).reset_state(cx);
            }

            // Handle provider removed
            if let ProviderViewAction::ProviderRemoved = action.cast() {
                // Select another provider
                let store = scope.data.get::<Store>().unwrap();
                if let Some(first_provider) = store.chats.providers.values().next() {
                    self.selected_provider_id = Some(first_provider.id.clone());
                    cx.action(ConnectionSettingsAction::ProviderSelected(
                        first_provider.id.clone(),
                    ));
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

        self.view(id!(status_view)).set_visible(
            cx,
            connection_status == ProviderConnectionStatus::Connected && self.provider.enabled,
        );

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ProviderItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let was_item_clicked = self.view(id!(main_view)).finger_up(actions).is_some();
        if was_item_clicked {
            cx.action(ConnectionSettingsAction::ProviderSelected(
                self.provider.id.clone(),
            ));
        }
    }
}

impl ProviderItem {
    /// Toggles the visibility of the connection status icons
    fn update_connection_status(
        &mut self,
        cx: &mut Cx,
        connection_status: &ProviderConnectionStatus,
    ) {
        self.view(id!(connection_status_success)).set_visible(
            cx,
            *connection_status == ProviderConnectionStatus::Connected,
        );
        self.view(id!(connection_status_failure)).set_visible(
            cx,
            *connection_status == ProviderConnectionStatus::Disconnected,
        );
        self.view(id!(connection_status_loading)).set_visible(
            cx,
            *connection_status == ProviderConnectionStatus::Connecting,
        );
    }
}

impl ProviderItemRef {
    fn set_provider(
        &mut self,
        cx: &mut Cx,
        provider: Provider,
        icon_path: Option<LiveDependency>,
        is_selected: bool,
    ) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.provider = provider.clone();

        // Determine whether to show image or label
        if let Some(icon) = icon_path {
            // Show the image
            inner.view(id!(image_wrapper)).set_visible(cx, true);
            let image = inner.image(id!(provider_icon_image));
            let _ = image.load_image_dep_by_path(cx, icon.as_str());

            // Hide the label
            let label_view = inner.view(id!(provider_icon_label));
            label_view.set_visible(cx, false);
        } else {
            // Hide the image
            inner.view(id!(image_wrapper)).set_visible(cx, false);

            // Show the label
            let label_view = inner.view(id!(label_wrapper));
            label_view.set_visible(cx, true);

            // Get first character of the provider name
            let first_char = provider
                .name
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default();

            label_view
                .label(id!(initial_label))
                .set_text(cx, &first_char);
        }

        if is_selected && cx.display_context.is_desktop() {
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { color: #EAECEF }
                },
            );
        } else {
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { color: #f9f9f9 }
                },
            );
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ConnectionSettingsAction {
    None,
    ProviderSelected(String),
}
