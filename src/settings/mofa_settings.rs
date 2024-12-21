use makepad_widgets::*;
use moly_mofa::MofaServerId;

use crate::data::chats::{MofaServer, MofaServerConnectionStatus};
use crate::data::store::Store;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;

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

    MofaServerItem = {{MofaServerItem}} {
        width: Fill, height: 60
        show_bg: true
        draw_bg: {
            color: #f
        }
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

            address_editable = <View> {
                width: Fill, height: Fill
                spacing: 10
                align: {x: 0.0, y: 0.5}

                mofa_address_label = <Label> {
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

    MofaServers = {{MofaServers}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 20
        <RoundedView> {
            width: Fill
            height: Fit
            padding: {left: 30, right: 30, top: 10, bottom: 10}
            show_bg: true
            draw_bg: {
                color: #f
                radius: 3
            }

            add_server_input = <MolyTextInput> {
                width: Fill
                height: Fit
                empty_message: "Add a new server"
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

            severs_list = <PortalList> {
                width: Fill, height: Fill
                mofa_server_item = <MofaServerItem> {}
            }
        }
    }

    pub MofaSettings = {{MofaSettings}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 20

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 16}
                color: #000
            }
            text: "MoFa Settings"
        }

        <HorizontalFiller> { height: 10 }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12}
                color: #000
            }
            text: "MoFa Servers"
        }

        <MofaServers> {}
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct MofaSettings {
    #[deref]
    view: View,
}

impl Widget for MofaSettings {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MofaSettings {}

#[derive(Widget, LiveHook, Live)]
struct MofaServers {
    #[deref]
    view: View,
}

impl Widget for MofaServers {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let mut servers: Vec<_> = store.chats.mofa_servers.values().cloned().collect();
        let entries_count = servers.len();
        let last_item_id = if entries_count > 0 { entries_count } else { 0 };
        servers.sort_by(|a, b| a.address.cmp(&b.address));
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, last_item_id);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < last_item_id {
                        let template = live_id!(mofa_server_item);
                        let item = list.item(cx, item_id, template);

                        // hide the separator for the first item
                        if item_id == 0 {
                            item.view(id!(separator)).set_visible(false);
                        }

                        let server = &servers[item_id].clone();
                        let mut item_scope = Scope::with_props(server);
                        item.draw_all(cx, &mut item_scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for MofaServers {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let add_server_input = self.view.text_input(id!(add_server_input));
        if let Some(address) = add_server_input.returned(actions) {
            store.chats.register_mofa_server(address);
            add_server_input.set_text("");

            self.redraw(cx);
        }
    }
}

#[derive(Widget, LiveHook, Live)]
struct MofaServerItem {
    #[deref]
    view: View,

    #[rust]
    server_id: MofaServerId,
}

impl Widget for MofaServerItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let server = scope.props.get::<MofaServer>().unwrap();
        self.server_id = MofaServerId(server.address.clone());

        self.update_connection_status(&server.connection_status);

        self.label(id!(address_editable.mofa_address_label))
            .set_text(&server.address);

        if server.is_local() {
            self.view.view(id!(icon_local)).set_visible(true);
            self.view.view(id!(icon_remote)).set_visible(false);
        } else {
            self.view.view(id!(icon_local)).set_visible(false);
            self.view.view(id!(icon_remote)).set_visible(true);
        };

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MofaServerItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if self.button(id!(remove_server)).clicked(actions) {
            store.chats.remove_mofa_server(&self.server_id.0);
            self.redraw(cx);
        }

        if let Some(_) = self
            .view(id!(connection_status_failure))
            .finger_down(actions)
        {
            store.chats.test_mofa_server_and_fetch_agents(&self.server_id.0);
            self.update_connection_status(&MofaServerConnectionStatus::Connecting);
            self.redraw(cx);
        }
    }
}

impl MofaServerItem {
    /// Toggles the visibility of the connection status icons based on the connection status
    fn update_connection_status(&mut self, connection_status: &MofaServerConnectionStatus) {
        self.view(id!(connection_status_success))
            .set_visible(*connection_status == MofaServerConnectionStatus::Connected);
        self.view(id!(connection_status_failure))
            .set_visible(*connection_status == MofaServerConnectionStatus::Disconnected);
        self.view(id!(connection_status_loading))
            .set_visible(*connection_status == MofaServerConnectionStatus::Connecting);
    }
}
