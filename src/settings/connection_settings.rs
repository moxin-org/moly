use makepad_widgets::*;
use serde::{Deserialize, Serialize};

use crate::data::chats::{MofaServer, ServerConnectionStatus, ServerType};
use crate::data::store::Store;
use crate::data::remote_servers::OpenAIClient;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::modal::*;
    use crate::settings::delete_server_modal::DeleteServerModal;

    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")
    ICON_REMOTE = dep("crate://self/resources/images/globe_icon.png")
    ICON_LOCAL = dep("crate://self/resources/images/laptop_icon.png")

    ICON_SUCCESS = dep("crate://self/resources/images/circle_check_icon.png")
    ICON_LOADER = dep("crate://self/resources/images/loader_icon.png")
    ICON_FAILURE = dep("crate://self/resources/images/refresh_error_icon.png")

    // Not making this based on <Icon> because button does not support images
    // (and these SVGs are too complex for Makepad's SVG support)
    ConnectionStatusButton = <View> {
        visible: false
        cursor: Hand
        width: Fit, height: Fit
        
        icon = <Image> {
            width: 18, height: 18
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

    ProviderDropDown = <DropDownFlat> {
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 10}
            fn get_color(self) -> vec4 {
                return mix(
                    #2,
                    #x0,
                    self.pressed
                )
            }
        }

        popup_menu: {
            width: 300, height: Fit,
            flow: Down,
            padding: <THEME_MSPACE_1> {}
            
            menu_item: <PopupMenuItem> {
                width: Fill, height: Fit,
                align: { y: 0.5 }
                padding: {left: 15, right: 15, top: 10, bottom: 10}
                
                draw_name: {
                    fn get_color(self) -> vec4 {
                        return mix(
                            mix(
                                #3,
                                #x0,
                                self.selected
                            ),
                            #x0,
                            self.hover
                        )
                    }
                }
                
                draw_bg: {
                    instance color: #f //(THEME_COLOR_FLOATING_BG)
                    instance color_selected: #f2 //(THEME_COLOR_CTRL_HOVER)
                }
            }
            
            draw_bg: {
                instance color: #f9 //(THEME_COLOR_FLOATING_BG)
            }
        }
    }

    AiServerItem = {{AiServerItem}} {
        width: Fill
        height: 50
        flow: Down

        separator = <View> {
            margin: {left: 20, right: 20, top: 0, bottom: 10}
            height: 1,
            show_bg: true,
            draw_bg: {
                color: #D9D9D9
            }
        }
    
        <View> {
            padding: {left: 30, right: 30, top: 0, bottom: 10}
            align: {x: 0.0, y: 0.5}
            spacing: 20
            flow: Right

            icon_remote = <View> {
                width: Fit, height: Fit
                visible: true
                <Image> {
                    source: (ICON_REMOTE)
                    width: 18, height: 18
                }
            }

            icon_local = <View> {
                width: Fit, height: Fit
                visible: false
                <Image> {
                    source: (ICON_LOCAL)
                    width: 18, height: 18
                }
            }
    
            <View> {
                width: Fill, height: Fill
                spacing: 10
                align: {x: 0.0, y: 0.5}
                server_address_label = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 12}
                        color: #000
                    }
                }

                <VerticalFiller> {}

                connection_status_success = <ConnectionStatusButton> {
                    icon = {
                        source: (ICON_SUCCESS)
                        draw_bg: {
                            tint_color: #099250
                        }
                    }
                }

                connection_status_failure = <ConnectionStatusButton> {
                    icon = {
                        source: (ICON_FAILURE)
                        draw_bg: {
                            tint_color: #B42318
                        }
                    }
                }

                connection_status_loading = <ConnectionStatusButton> {
                    visible: true
                    icon = {
                        source: (ICON_LOADER)
                        draw_bg: {
                            tint_color: #FF8C00
                        }
                    }
                }

                remove_server = <MolyButton> {
                    width: Fit
                    height: Fit

                    draw_bg: {
                        border_width: 1,
                        radius: 3
                    }

                    icon_walk: {width: 14, height: 14}
                    draw_icon: {
                        svg_file: (ICON_DELETE),
                        fn get_color(self) -> vec4 {
                            return #B42318;
                        }
                    }
                }
            }
        }
    }

    delete_modal = <Modal> {
        content: {
            <DeleteServerModal> {}
        }
    }

    HorizontalSeparator =  <RoundedView> {
        width: 2, height: Fill
        show_bg: true
        draw_bg: {
            color: #d3d3d3
        }
    }

    AiServers = {{AiServers}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 20
        
        <RoundedView> {
            flow: Right
            width: Fill, height: 55
            spacing: 10
            align: {x: 0.0, y: 0.5}
            padding: {left: 20, right: 20, top: 10, bottom: 10}
            show_bg: true
            draw_bg: {
                color: #f
                radius: 3
            }

            add_server_input = <MolyTextInput> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 10},
                    fn get_color(self) -> vec4 {
                        if self.is_empty > 0.5 {
                            return #475467;
                        }
                        return #000;
                    }
                }
                width: 400, height: Fit
                empty_message: "provider URL (e.g. https://api.openai.com/v1)"
            }
            
            <HorizontalSeparator> {}

            api_key = <MolyTextInput> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 10},
                    fn get_color(self) -> vec4 {
                        if self.is_empty > 0.5 {
                            return #475467;
                        }
                        return #000;
                    }
                }
                width: Fill
                height: Fit
                empty_message: "API key (optional)"
            }

            <HorizontalSeparator> {}

            provider_type = <ProviderDropDown> {
                width: Fill

                labels: ["OpenAI API", "MoFa"]
                values: [OpenAIAPI, MoFa]
            }

            add_server_button = <MolyButton> {
                width: Fit
                height: Fill
                padding: {left: 20, right: 20, top: 0, bottom: 0}
                text: "Add Server"
                draw_bg: { color: #099250, border_color: #099250 }
            }
        }

        <RoundedView> {
            width: Fill
            height: Fill
            show_bg: true
            draw_bg: {
                color: #f
                radius: 3
            }
            padding: 10

            servers_list = <PortalList> {
                width: Fill, height: Fill
                ai_server_item = <AiServerItem> {}
            }
        }
    }

    pub ConnectionSettings = {{ConnectionSettings}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 20

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 16}
                color: #000
            }
            text: "AI Provider Settings"
        }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12}
                color: #000
            }
            text: "Manage Connections"
        }

        <AiServers> {}
    }
}

#[derive(Live, LiveHook, PartialEq, Debug, LiveRead, Serialize, Deserialize, Clone)]
pub enum ProviderType {
    #[pick]
    OpenAIAPI,
    MoFa,
}

impl ProviderType {
    fn from_usize(value: usize) -> Self {
        match value {
            0 => ProviderType::OpenAIAPI,
            1 => ProviderType::MoFa,
            _ => panic!("Invalid provider type"),
        }
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct ConnectionSettings {
    #[deref]
    view: View,
}

impl Widget for ConnectionSettings {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ConnectionSettings {}

#[derive(Widget, LiveHook, Live)]
struct AiServers {
    #[deref]
    view: View,
}

impl Widget for AiServers {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();

        // Collect Mofa servers from store
        let mut mofa_servers: Vec<_> = store.chats.mofa_servers.values().cloned().map(AiServerEntry::Mofa).collect();
        // Collect OpenAI servers from store
        let mut openai_servers: Vec<_> = store.chats.openai_servers.values().cloned().map(AiServerEntry::OpenAI).collect();

        // Combine them
        let mut all_servers = Vec::new();
        all_servers.append(&mut mofa_servers);
        all_servers.append(&mut openai_servers);

        // Sort them by address
        all_servers.sort_by(|a, b| a.address().cmp(&b.address()));

        let entries_count = all_servers.len();

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, entries_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < entries_count {
                        let template = live_id!(ai_server_item);
                        let item = list.item(cx, item_id, template);

                        // hide the separator for the first item
                        if item_id == 0 {
                            item.view(id!(separator)).set_visible(cx, false);
                        }

                        let server_data = all_servers[item_id].clone();
                        item.as_ai_server_item().set_server_entry(server_data);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for AiServers {
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

/// A small enum to unify the idea of either a Mofa server or an OpenAI server.
#[derive(Clone, Debug)]
pub enum AiServerEntry {
    Mofa(MofaServer),
    OpenAI(OpenAIClient),
}

impl AiServerEntry {
    fn address(&self) -> String {
        match self {
            AiServerEntry::Mofa(m) => m.client.address.clone(),
            AiServerEntry::OpenAI(o) => o.address.clone(),
        }
    }
}

#[derive(Widget, LiveHook, Live)]
struct AiServerItem {
    #[deref]
    view: View,

    #[rust]
    server_address: String,

    #[rust]
    server_entry: Option<AiServerEntry>,
}

impl Widget for AiServerItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Extract address and status
        let (address, connection_status, is_local, _is_mofa) = match &self.server_entry {
            Some(AiServerEntry::Mofa(ref m)) => {
                (m.client.address.clone(), m.connection_status.clone(), m.is_local(), true)
            }
            Some(AiServerEntry::OpenAI(ref o)) => {
                (o.address.clone(), o.connection_status.clone(), false, false)
            }
            None => {
                return DrawStep::done();
            }
        };
        self.server_address = address.clone();

        // Show/hide local icon
        if is_local {
            self.view.view(id!(icon_local)).set_visible(cx, true);
            self.view.view(id!(icon_remote)).set_visible(cx, false);
        } else {
            self.view.view(id!(icon_local)).set_visible(cx, false);
            self.view.view(id!(icon_remote)).set_visible(cx, true);
        }

        // Update the label
        self.label(id!(server_address_label))
            .set_text(cx, &address);

        // Show connection status icons
        self.update_connection_status(cx, &connection_status);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for AiServerItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        // "Remove server" click
        // TODO: Use the modal instead. Currently broken and needs refactoring.
        if self.button(id!(remove_server)).clicked(actions) {
            match &self.server_entry {
                Some(AiServerEntry::Mofa(ref m)) => {
                    store.chats.remove_mofa_server(&m.client.address);
                    store.preferences.remove_server_connection(&m.client.address);
                }
                Some(AiServerEntry::OpenAI(ref o)) => {
                    store.chats.openai_servers.remove(&o.address);
                    store.preferences.remove_server_connection(&o.address);
                }
                None => {}
            }
            self.redraw(cx);
        }

        // Re-test connection if the user clicks on the "failure" status
        if let Some(_) = self.view(id!(connection_status_failure)).finger_down(actions) {
            match &self.server_entry {
                Some(AiServerEntry::Mofa(_)) => {
                    store.chats.test_mofa_server_and_fetch_agents(&self.server_address);
                    self.update_connection_status(cx, &ServerConnectionStatus::Connecting);
                }
                Some(AiServerEntry::OpenAI { .. }) => {
                    // For OpenAI servers:
                    store.chats.test_openai_server_and_fetch_models(&self.server_address);
                    self.update_connection_status(cx, &ServerConnectionStatus::Connecting);
                }
                None => {}
            }
            self.redraw(cx);
        }
    }
}

impl AiServerItem {
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

impl AiServerItemRef {
    fn set_server_entry(&mut self, entry: AiServerEntry) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.server_entry = Some(entry);
    }
}
