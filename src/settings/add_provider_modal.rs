use makepad_widgets::*;

use crate::data::{
    providers::{Provider, ProviderConnectionStatus, ProviderType},
    store::Store,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::widgets::MolyButton;
    use crate::shared::resource_imports::*;

    FormGroup = <View> {
        flow: Down
        height: Fit
        spacing: 10
        align: {x: 0.0, y: 0.5}
    }

    ModalTextInput = <MolyTextInput> {
        draw_bg: {
            border_size: 1.0
            border_color_1: #ddd
        }
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 12},
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
            text_style: <REGULAR_FONT>{font_size: 12},
            color: #000
        }
    }

    ProviderDropDown = <DropDownFlat> {
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 12}
            fn get_color(self) -> vec4 {
                return mix(
                    #2,
                    #x0,
                    self.down
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

                draw_text: {
                    fn get_color(self) -> vec4 {
                        return mix(
                            mix(
                                #3,
                                #x0,
                                self.active
                            ),
                            #x0,
                            self.hover
                        )
                    }
                }

                draw_bg: {
                    instance color: #f //(THEME_COLOR_FLOATING_BG)
                    instance color_active: #f2 //(THEME_COLOR_CTRL_HOVER)
                }
            }

            draw_bg: {
                instance color: #f9 //(THEME_COLOR_FLOATING_BG)
                border_size: 1.0
            }
        }
    }

    CustomProviderRadio = <RadioButton> {
        draw_text: {
            color: #000
            color_hover: #000
            color_active: #000
            color_focus: #000
            text_style: <REGULAR_FONT>{font_size: 12},
        }

        draw_bg: {
            color: (TRANSPARENT)
            color_hover: (TRANSPARENT)
            color_down: (TRANSPARENT)
            color_active: (TRANSPARENT)
            color_focus: (TRANSPARENT)
            color_disabled: (TRANSPARENT)

            border_color_1: #ddd
            border_color_1_hover: #ddd
            border_color_1_down: #ddd
            border_color_1_active: #ddd
            border_color_1_focus: #ddd
            border_color_1_disabled: #ddd

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

    pub AddProviderModal = {{AddProviderModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 420
            height: Fit
            padding: {top: 44, right: 30 bottom: 30 left: 50}
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
                        text: "Add a custom provider",
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

                <FormGroup> {
                    <ModalLabel> {
                        text: "Name"
                    }
                    name = <ModalTextInput> {
                        empty_text: "OpenAI"
                    }
                }

                <FormGroup> {
                    <ModalLabel> {
                        text: "API Host"
                    }
                    api_host = <ModalTextInput> {
                        empty_text: "e.g. https://api.openai.com/v1"
                    }
                }

                <FormGroup> {
                    <ModalLabel> {
                        text: "API Key (optional)"
                    }
                    api_key = <ModalTextInput> {
                        empty_text: "sk-..."
                    }
                }

                <FormGroup> {
                    <ModalLabel> {
                        text: "Provider Type"
                    }

                    // TODO: we should replace the radio buttons with a dropdown
                    // currently the dropdown popup is not working inside the modal
                    // provider_type = <ProviderDropDown> {
                    //     width: Fill
                    //     labels: ["OpenAI", "MoFa"]
                    //     values: [OpenAI, MoFa]
                    // }

                    radios = <View> {
                        flow: Down, spacing: 10
                        width: Fit, height: Fit,
                        radio_openai = <CustomProviderRadio> { text: "OpenAI" }
                        radio_mofa = <CustomProviderRadio> { text: "MoFa" }
                        radio_deepinquire = <CustomProviderRadio> { text: "DeepInquire" }
                        radio_moly_server = <CustomProviderRadio> { text: "MolyServer" }
                    }
                }

                error_view = <View> {
                    visible: false
                    width: Fill, height: Fit
                    error_message = <Label> {
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 12},
                            color: #f00
                        }
                    }
                }

                <View> {
                    width: Fill, height: Fit
                    align: {x: 1.0, y: 0.5}
                    add_server_button = <MolyButton> {
                        width: 130
                        height: 40
                        padding: {left: 20, right: 20, top: 0, bottom: 0}
                        text: "Add Provider"
                        draw_bg: { color: (CTA_BUTTON_COLOR), border_color_1: (CTA_BUTTON_COLOR) }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum AddProviderModalAction {
    None,
    ModalDismissed,
}

#[derive(Live, LiveHook, Widget)]
pub struct AddProviderModal {
    #[deref]
    view: View,

    #[rust]
    selected_provider: Option<ProviderType>,
}

impl Widget for AddProviderModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetMatchEvent for AddProviderModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if self.button(id!(close_button)).clicked(actions) {
            cx.action(AddProviderModalAction::ModalDismissed);
        }

        if self.button(id!(add_server_button)).clicked(actions) {
            self.clear_error_message(cx);
            let api_host = self.text_input(id!(api_host)).text();
            let name = self.text_input(id!(name)).text();
            // Do not create a provider if the api host is already in the list
            if store.chats.providers.contains_key(&api_host) {
                self.set_error_message(cx, "Provider already exists with this API host");
                return;
            }

            // Check if the URL is valid
            if !api_host.starts_with("http") {
                self.set_error_message(cx, "Invalid API host");
                return;
            }

            // Check if the provider type is selected
            if self.selected_provider.is_none() {
                self.set_error_message(cx, "Please select a provider type");
                return;
            }

            // Check if the name is empty
            if name.is_empty() {
                self.set_error_message(cx, "Please enter a name for the provider");
                return;
            }

            let api_key = self.text_input(id!(api_key)).text();
            let provider = match self.selected_provider.as_ref().unwrap() {
                ProviderType::OpenAI => Provider {
                    name: name.clone(),
                    url: api_host.clone(),
                    api_key: Some(api_key.clone()),
                    provider_type: ProviderType::OpenAI,
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: true,
                    models: vec![],
                    was_customly_added: true,
                },
                ProviderType::OpenAIImage => Provider {
                    name: name.clone(),
                    url: api_host.clone(),
                    api_key: Some(api_key.clone()),
                    provider_type: ProviderType::OpenAI,
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: true,
                    models: vec![],
                    was_customly_added: true,
                },
                ProviderType::MolyServer => Provider {
                    name: name.clone(),
                    url: api_host.clone(),
                    api_key: Some(api_key.clone()),
                    provider_type: ProviderType::MolyServer,
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: true,
                    models: vec![],
                    was_customly_added: true,
                },
                ProviderType::MoFa => Provider {
                    name: name.clone(),
                    url: api_host.clone(),
                    api_key: Some(api_key.clone()),
                    provider_type: ProviderType::MoFa,
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: true,
                    models: vec![],
                    was_customly_added: true,
                },
                ProviderType::DeepInquire => Provider {
                    name: name.clone(),
                    url: api_host.clone(),
                    api_key: Some(api_key.clone()),
                    provider_type: ProviderType::DeepInquire,
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: true,
                    models: vec![],
                    was_customly_added: true,
                },
            };

            store.insert_or_update_provider(&provider);

            cx.action(AddProviderModalAction::ModalDismissed);
            self.clear_form(cx);
        }

        let selected = self
            .radio_button_set(ids!(
                radios.radio_openai,
                radios.radio_mofa,
                radios.radio_deepinquire,
                radios.radio_moly_server
            ))
            .selected(cx, actions);
        if let Some(selected) = selected {
            self.selected_provider = match selected {
                0 => Some(ProviderType::OpenAI),
                1 => Some(ProviderType::MoFa),
                2 => Some(ProviderType::DeepInquire),
                3 => Some(ProviderType::MolyServer),
                _ => Some(ProviderType::OpenAI),
            };
        }
    }
}

impl AddProviderModal {
    fn set_error_message(&mut self, cx: &mut Cx, message: &str) {
        self.view(id!(error_view)).set_visible(cx, true);
        self.label(id!(error_message)).set_text(cx, message);
    }

    fn clear_error_message(&mut self, cx: &mut Cx) {
        self.label(id!(error_message)).set_text(cx, "");
        self.view(id!(error_view)).set_visible(cx, false);
    }

    fn clear_form(&mut self, cx: &mut Cx) {
        self.text_input(id!(api_host)).set_text(cx, "");
        self.text_input(id!(api_key)).set_text(cx, "");
        self.clear_error_message(cx);
        self.selected_provider = None;
    }
}
