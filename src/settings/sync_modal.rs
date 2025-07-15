use std::net::UdpSocket;

use anyhow::Error;
use makepad_widgets::*;
use moly_kit::utils::asynchronous::spawn;
use moly_sync::fetch_json;

#[cfg(not(target_arch = "wasm32"))]
use moly_sync::{ServerHandle, start_server};

use crate::data::store::Store;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::widgets::MolyButton;
    use crate::shared::resource_imports::*;

    ShadowButton = <RoundedShadowView> {
        cursor: Hand
        width: Fill, height: Fit
        align: {x: 0.5, y: 0.5}
        padding: {left: 10, right: 10, bottom: 8, top: 8}
        draw_bg: {
            color: (MAIN_BG_COLOR)
            border_radius: 4.5,
            uniform shadow_color: #0002
            shadow_radius: 8.0,
            shadow_offset: vec2(0.0,-2.0)
        }
        label = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 11}
                color: #000
            }
        }
    }

    FormGroup = <View> {
        flow: Down
        height: Fit
        spacing: 10
        align: {x: 0.0, y: 0.5}
    }

    ModalTextInput = <MolyTextInput> {
        padding: 10
        draw_bg: {
            border_size: 1.0
            border_color: #ddd
        }
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 11},
            color: #000
            color_hover: #000
            color_focus: #000
            color_empty: #98A2B3
            color_empty_focus: #98A2B3
        }
        width: Fill, height: Fit
    }

    ModalLabel = <Label> {
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 11},
            color: #000
        }
    }

    CustomProviderRadio = <RadioButton> {
        draw_text: {
            color: #000
            color_hover: #000
            color_active: #000
            color_focus: #000
            text_style: <REGULAR_FONT>{font_size: 11},
        }

        draw_bg: {
            color: (TRANSPARENT)
            color_hover: (TRANSPARENT)
            color_down: (TRANSPARENT)
            color_active: (TRANSPARENT)
            color_focus: (TRANSPARENT)
            color_disabled: (TRANSPARENT)

            border_color: #ddd
            border_color_hover: #ddd
            border_color_down: #ddd
            border_color_active: #ddd
            border_color_focus: #ddd
            border_color_disabled: #ddd

            border_color_2: #ddd
            border_color_2_hover: #ddd
            border_color_2_down: #ddd
            border_color_2_active: #ddd
            border_color_2_focus: #ddd
            border_color_2_disabled: #ddd

            mark_color: (TRANSPARENT)
            mark_color_active: (PRIMARY_COLOR)
            mark_color_disabled: (TRANSPARENT)
        }
    }

    ImportView = <View> {
        width: Fill, height: Fit
        visible: false
        flow: Down
        spacing: 10
        align: {x: 0.0, y: 0.5}
        padding: 10

        <FormGroup> {
            <ModalLabel> {
                text: "Serving sync address:"
            }
            import_url = <ModalTextInput> {
                empty_text: "http://localhost:8080"
                // text: "http://192.168.1.4:8080"
            }
        }

        <FormGroup> {
            <ModalLabel> {
                text: "Access PIN:"
            }
            import_pin = <ModalTextInput> {
                empty_text: "1234"
                width: Fit
                // text: "http://192.168.1.4:8080"
            }
        }

        <FormGroup> {
            <ModalLabel> {
                text: "How do you want to sync?"
            }

            radios = <View> {
                flow: Down, spacing: 10
                width: Fill, height: Fit,
                radio_merge = <CustomProviderRadio> { text: "Merge with existing providers" }
                radio_replace = <CustomProviderRadio> { text: "Replace existing providers" }
            }
        }

        import = <ShadowButton> {
            label = { text: "Import" }
            width: Fill
        }
    }

    ExportView = <View> {
        width: Fill, height: Fit
        visible: false
        padding: 10
        flow: Down
        spacing: 10

        <FormGroup> {
            <ModalLabel> {
                text: "Serving sync address:"
            }
            serving_url = <ModalLabel> {
                text: "http://localhost:8080"
            }
        }
        <FormGroup> {
            <ModalLabel> {
                text: "Access PIN:"
            }
            sync_pin = <ModalLabel> {
                text: "1234"
            }
        }
        stop_server = <ShadowButton> {
            label = { text: "Stop sharing" }
            width: Fill
        }
    }

    pub SyncModal = {{SyncModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 420
            height: Fit
            padding: 25
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                border_radius: 3
            }

            header = <View> {
                width: Fill,
                height: Fit,
                flow: Right

                padding: {top: 8, bottom: 20}

                title = <View> {
                    width: Fit,
                    height: Fit,

                    model_name = <Label> {
                        text: "Sync provider settings",
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 13},
                            color: #000
                        }
                    }
                }

                filler_x = <View> {width: Fill, height: Fit}

                close_button = <MolyButton> {
                    width: Fit,
                    height: Fit,

                    margin: {top: -8}

                    draw_icon: {
                        svg_file: (ICON_CLOSE),
                        fn get_color(self) -> vec4 {
                            return #000;
                        }
                    }
                    icon_walk: {width: 12, height: 12}
                }
            }

            body = <View> {
                flow: Down
                width: Fill, height: Fit
                spacing: 20
                align: {x: 0.0, y: 0.5}

                <View> {
                    width: Fill, height: Fit
                    spacing: 10, flow: Down

                    hint = <ModalLabel> {
                        text: "Sync settings between devices"
                        draw_text: {
                            text_style: {font_size: 11},
                        }
                    }

                    sync_buttons = <View> {
                        width: Fill, height: Fit
                        padding: 10
                        spacing: 10
                        align: {x: 0.5, y: 0.5}

                        serve = <ShadowButton> {
                            label = { text: "Share from this device" }
                        }

                        show_import = <ShadowButton> {
                            label = { text: "Import from another" }
                        }
                    }
                }

                import_view = <ImportView> {}
                export_view = <ExportView> {}

                status_view = <View> {
                    visible: false
                    width: Fill, height: Fit
                    status_message = <Label> {
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 11},
                            color: #000
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum SyncModalAction {
    None,
    ModalDismissed,
}

#[derive(Live, LiveHook, Widget)]
pub struct SyncModal {
    #[deref]
    view: View,

    #[rust]
    should_merge: bool,

    #[rust]
    sync_status: SyncStatus,

    #[cfg(not(target_arch = "wasm32"))]
    #[rust]
    server_handle: Option<ServerHandle>,
}

#[derive(Clone, Debug, DefaultNone)]
pub enum SyncStatus {
    None,
    Serving,
    Importing,
}

impl Widget for SyncModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        #[cfg(target_arch = "wasm32")]
        {
            self.view(id!(sync_buttons)).set_visible(cx, false);
            self.view(id!(import_view)).set_visible(cx, true);
            self.label(id!(hint))
                .set_text(cx, "Import your settings from another Moly instance");
        }

        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetMatchEvent for SyncModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.button(id!(close_button)).clicked(actions) {
            self.reset_state(cx);
            cx.action(SyncModalAction::ModalDismissed);
        }

        if self.view(id!(serve)).finger_down(actions).is_some() {
            self.show_export(cx);
            if let SyncStatus::None = self.sync_status {
                self.serve(cx, scope);
            }
        }

        if self.view(id!(stop_server)).finger_down(actions).is_some() {
            self.stop_server(cx);
        }

        if self.view(id!(show_import)).finger_down(actions).is_some() {
            self.show_import(cx);
        }

        if self.view(id!(import)).finger_down(actions).is_some() {
            if let SyncStatus::None = self.sync_status {
                self.import();
            }
        }

        if let Some(selected_sync_mode) = self
            .radio_button_set(ids!(radios.radio_merge, radios.radio_replace))
            .selected(cx, actions)
        {
            self.should_merge = match selected_sync_mode {
                0 => true,
                1 => false,
                _ => true,
            };
        } else {
            self.radio_button(id!(radios.radio_merge)).select(cx, scope);
        }
    }
}

impl SyncModal {
    #[cfg(not(target_arch = "wasm32"))]
    fn serve(&mut self, _cx: &mut Cx, scope: &mut Scope) {
        let json_file = scope.data.get_mut::<Store>().unwrap().preferences.as_json();

        let ui = self.ui_runner();
        spawn(async move {
            // Start moly-sync server
            let server_result = start_server(json_file, None).await;
            match server_result {
                Ok(server_handle) => {
                    let addr = server_handle.addr;
                    ::log::info!("Sync server started at {:?}", addr);
                    ui.defer_with_redraw(move |me, cx, _| {
                        me.sync_status = SyncStatus::Serving;
                        me.label(id!(sync_pin)).set_text(cx, &server_handle.pin);
                        me.server_handle = Some(server_handle);

                        let full_server_url =
                            format!("http://{}:{}", get_local_ip_address(), addr.port());
                        me.label(id!(serving_url)).set_text(cx, &full_server_url);
                    });
                }
                Err(e) => {
                    ::log::error!("Failed to start sync server: {}", e);
                    ui.defer_with_redraw(move |me, cx, _| {
                        me.sync_status = SyncStatus::None;
                        me.label(id!(status_message))
                            .set_text(cx, &format!("Failed to start server: {}", e));
                        me.view(id!(status_view)).set_visible(cx, true);
                    });
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn stop_server(&mut self, cx: &mut Cx) {
        if let Some(server_handle) = self.server_handle.take() {
            server_handle.stop();
            ::log::info!("Sync server stopped");
        }
        self.sync_status = SyncStatus::None;
        self.view(id!(export_view)).set_visible(cx, false);
    }

    #[cfg(target_arch = "wasm32")]
    fn serve(&mut self, _cx: &mut Cx, _scope: &mut Scope) {
        ::log::error!("Sync server is not supported on wasm32");
    }

    #[cfg(target_arch = "wasm32")]
    fn stop_server(&mut self, _cx: &mut Cx) {
        ::log::error!("Sync server is not supported on wasm32");
    }

    fn show_export(&mut self, cx: &mut Cx) {
        self.view(id!(import_view)).set_visible(cx, false);
        self.view(id!(export_view)).set_visible(cx, true);
    }

    fn show_import(&mut self, cx: &mut Cx) {
        self.view(id!(import_view)).set_visible(cx, true);
        self.view(id!(export_view)).set_visible(cx, false);
    }

    fn import(&mut self) {
        let url = self.text_input(id!(import_view.import_url)).text();
        let pin = self.text_input(id!(import_view.import_pin)).text();

        let ui = self.ui_runner();
        self.sync_status = SyncStatus::Importing;

        spawn(async move {
            match fetch_json(&url, &pin).await {
                Ok(json) => {
                    ui.defer_with_redraw(move |me, cx, scope| {
                        me.handle_import_success(cx, &json, scope);
                    });
                }
                Err(e) => {
                    ui.defer_with_redraw(move |me, cx, _| {
                        me.handle_import_error(cx, e);
                    });
                }
            }
        });
    }

    fn handle_import_success(&mut self, cx: &mut Cx, json: &str, scope: &mut Scope) {
        self.view(id!(status_view)).set_visible(cx, true);
        self.sync_status = SyncStatus::None;

        let store = scope.data.get_mut::<Store>().unwrap();
        match store.preferences.import_from_json(json, self.should_merge) {
            Ok(_) => {
                self.label(id!(status_message))
                    .set_text(cx, "Import successful");
                store.bot_context = None;
                store.load_preference_connections();
                ::log::info!("Import of settings successful");
            }
            Err(e) => {
                ::log::error!("Failed to import: {}", e);
                self.label(id!(status_message))
                    .set_text(cx, "Failed to import");
            }
        }
    }

    fn handle_import_error(&mut self, cx: &mut Cx, error: Error) {
        ::log::error!("Failed to fetch settings: {:?}", error);
        self.view(id!(status_view)).set_visible(cx, true);
        self.label(id!(status_message))
            .set_text(cx, &format!("Failed to fetch settings: {:?}", error));
    }

    fn reset_state(&mut self, cx: &mut Cx) {
        self.view(id!(export_view)).set_visible(cx, false);
        self.view(id!(import_view)).set_visible(cx, false);
        self.view(id!(status_view)).set_visible(cx, false);
        self.label(id!(status_message)).set_text(cx, "");
        self.sync_status = SyncStatus::None;
        self.stop_server(cx);
    }
}

impl SyncModalRef {
    pub fn reset_state(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.reset_state(cx);
        }
    }
}

fn get_local_ip_address() -> String {
    // Use a dummy address to get the local IP for outbound traffic
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    // This address doesn't need to be reachable — it's just to force the OS to assign a local IP
    if socket.connect("8.8.8.8:80").is_ok() {
        if let Ok(local_addr) = socket.local_addr() {
            return local_addr.ip().to_string();
        }
    }

    // Fallback if all else fails
    "localhost".to_string()
}
