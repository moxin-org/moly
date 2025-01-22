use makepad_code_editor::code_view::CodeViewWidgetExt;
use makepad_widgets::*;

use crate::data::{
    chats::model_loader::{ModelLoaderStatus, ModelLoaderStatusChanged},
    store::Store,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use makepad_code_editor::code_view::CodeView;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::settings::mofa_settings::MofaSettings;

    BG_IMAGE = dep("crate://self/resources/images/my_models_bg_image.png")
    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")

    pub SettingsScreen = {{SettingsScreen}} {
        width: Fill
        height: Fill
        flow: Overlay

        <Image> {
            source: (BG_IMAGE),
            width: Fill,
            height: Fill,
        }

        <View> {
            width: Fill, height: Fill
            flow: Down
            padding: 60

            spacing: 60

            <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 30}
                    color: #000
                }
                text: "Settings"
            }

            <ScrollYView> {
                width: Fill, height: Fill
                spacing: 40

                local_server_options = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 20

                    <Label> {
                        draw_text:{
                            text_style: <BOLD_FONT>{font_size: 16}
                            color: #000
                        }
                        text: "Local inference server information"
                    }

                    no_model = <View> {
                        visible: false,
                        width: Fill, height: Fit
                        <Label> {
                            draw_text:{
                                text_style: <REGULAR_FONT>{font_size: 12}
                                color: #000
                            }
                            text: "Local inference options will appear once you have a model loaded."
                        }
                    }

                    main = <View> {
                        width: Fill, height: Fit
                        flow: Down
                        align: {x: 0.0, y: 0.0}

                        spacing: 10

                        <View> {
                            width: Fit, height: Fit
                            flow: Right
                            spacing: 10
                            align: {x: 0.0, y: 0.5}

                            <Label> {
                                draw_text:{
                                    text_style: <REGULAR_FONT>{font_size: 12}
                                    color: #000
                                }
                                text: "Port number:"
                            }

                            port_on_edit = <View> {
                                visible: false,
                                width: Fit, height: Fit

                                port_number_input = <MolyTextInput> {
                                    width: 100,
                                    height: Fit,
                                    draw_text: {
                                        text_style: <REGULAR_FONT>{font_size: 12}
                                        color: #000
                                    }
                                }
                            }

                            port_editable = <View> {
                                width: Fit, height: Fit
                                spacing: 10
                                align: {x: 0.0, y: 0.5}

                                port_number_label = <Label> {
                                    draw_text:{
                                        text_style: <REGULAR_FONT>{font_size: 12}
                                        color: #000
                                    }
                                }

                                edit_port_number = <MolyButton> {
                                    width: Fit
                                    height: Fit

                                    draw_bg: {
                                        border_width: 1,
                                        radius: 3
                                    }

                                    margin: {bottom: 4}

                                    icon_walk: {width: 14, height: 14}
                                    draw_icon: {
                                        svg_file: (ICON_EDIT),
                                        fn get_color(self) -> vec4 {
                                            return #000;
                                        }
                                    }
                                }
                            }
                        }

                        load_info_label = <View> {
                            visible: false,
                            width: Fit, height: Fit
                            <Label> {
                                draw_text:{
                                    text_style: <REGULAR_FONT>{font_size: 12}
                                    color: #000
                                }
                                text: "Something went wrong while loading the model using this port number. Please try another one."
                            }
                        }

                        <HorizontalFiller> { height: 10 }

                        <Label> {
                            draw_text:{
                                text_style: <BOLD_FONT>{font_size: 12}
                                color: #000
                            }
                            text: "Client code example"
                        }

                        code_snippet = <CodeView> {
                            editor: {
                                pad_left_top: vec2(10.0,10.0)
                                width: Fill,
                                height: Fit,
                                draw_bg: { color: #3c3c3c },
                            }
                        }
                    }
                }

                mofa_section = <View> {
                    spacing: 40
                    <HorizontalFiller> {
                        width: 2,
                        show_bg: true
                        draw_bg: {
                            color: #c3c3c3
                        }
                    }

                    mofa_options = <MofaSettings> {}
                }
            }
        }
    }
}

#[derive(Default, Debug, PartialEq)]
enum ServerPortState {
    OnEdit,
    #[default]
    Editable,
}

#[derive(Widget, Live)]
pub struct SettingsScreen {
    #[deref]
    view: View,

    #[rust]
    server_port_state: ServerPortState,

    #[rust]
    override_port: Option<u16>,
}

impl Widget for SettingsScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();

        match self.server_port_state {
            ServerPortState::OnEdit => {
                self.view.view(id!(port_editable)).set_visible(cx, false);
                self.view.view(id!(port_on_edit)).set_visible(cx, true);
            }
            ServerPortState::Editable => {
                self.view.view(id!(port_editable)).set_visible(cx, true);
                self.view.view(id!(port_on_edit)).set_visible(cx, false);
            }
        }

        let port = self.override_port.or_else(|| {
            if let ModelLoaderStatus::Loaded(info) = store.chats.model_loader.status() {
                Some(info.listen_port)
            } else {
                None
            }
        });

        if let Some(port) = port {
            self.view
                .view(id!(local_server_options.no_model))
                .set_visible(cx, false);
            self.view
                .view(id!(local_server_options.main))
                .set_visible(cx, true);

            self.view
                .label(id!(port_number_label))
                .set_text(cx,&format!("{}", port));

            self.view.code_view(id!(code_snippet)).set_text(cx,&format!(
                "# Load a model and run this example in your terminal
# Choose between streaming and non-streaming mode by setting the \"stream\" field

curl http://localhost:{}/v1/chat/completions \\
-H \"Content-Type: application/json\" \\
-d '{{ 
\"model\": \"moly-chat\",
\"messages\": [ 
{{ \"role\": \"system\", \"content\": \"Use positive language and offer helpful solutions to their problems.\" }},
{{ \"role\": \"user\", \"content\": \"What is the currency used in Spain?\" }}
], 
\"temperature\": 0.7, 
\"stream\": true
}}'
                ",
                port
            ));
        } else {
            self.view
                .view(id!(local_server_options.no_model))
                .set_visible(cx, true);
            self.view
                .view(id!(local_server_options.main))
                .set_visible(cx, false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SettingsScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        for action in actions {
            // Once the modals are reloaded, let's clear the override port
            if let Some(_) = action.downcast_ref::<ModelLoaderStatusChanged>() {
                if store.chats.model_loader.is_loaded() {
                    self.override_port = None;
                }
                if store.chats.model_loader.is_failed() {
                    self.view(id!(load_error_label)).set_visible(cx, true);
                } else {
                    self.view(id!(load_error_label)).set_visible(cx, false);
                }
            }
        }

        let port_number_input = self.view.text_input(id!(port_number_input));

        if self.button(id!(edit_port_number)).clicked(actions) {
            self.server_port_state = ServerPortState::OnEdit;

            let port = self.label(id!(port_number_label)).text();
            port_number_input.set_key_focus(cx);
            port_number_input.set_text(cx,&port);

            self.redraw(cx);
        }

        if let Some(port) = port_number_input.returned(actions) {
            let port = port.parse::<u16>();

            if let Ok(port) = port {
                self.override_port = Some(port);
                store.update_server_port(port);
            }

            self.server_port_state = ServerPortState::Editable;
            self.redraw(cx);
        }

        if let TextInputAction::Escape =
            actions.find_widget_action_cast(port_number_input.widget_uid())
        {
            self.server_port_state = ServerPortState::Editable;
            self.redraw(cx);
        }
    }
}

impl LiveHook for SettingsScreen {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.view
            .view(id!(mofa_section))
            .set_visible(cx, moly_mofa::should_be_visible());
    }
}
