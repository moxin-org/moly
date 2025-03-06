use makepad_widgets::*;

use crate::data::{chats::{Provider, ServerConnectionStatus}, store::{ProviderType, Store}};

// use crate::data::{chats::Provider, store::Store};

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
            border_width: 1.0
            border_color: #ddd
        }
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 12},
            fn get_color(self) -> vec4 {
                if self.is_empty > 0.5 {
                    return #475467;
                }
                return #000;
            }
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
                border_width: 1.0
            }
        }
    }

    pub AddProviderModal = {{AddProviderModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 600
            height: Fit
            padding: {top: 44, right: 30 bottom: 30 left: 50}
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                radius: 3
            }

            header =<View> {
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
                        text: "API Host"
                    }
                    api_host = <ModalTextInput> {
                        empty_message: "e.g. https://api.openai.com/v1"
                    }
                }
                
                <FormGroup> {
                    <ModalLabel> {
                        text: "API Key (optional)"
                    }
                    api_key = <ModalTextInput> {
                        empty_message: "sk-..."
                    }
                }
        
                <FormGroup> {
                    <ModalLabel> {
                        text: "Provider Type"
                    }
                    provider_type = <ProviderDropDown> {
                        width: Fill
                        labels: ["OpenAI", "MoFa"]
                        values: [OpenAI, MoFa]
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
                        draw_bg: { color: #099250, border_color: #099250 }
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
    view: View
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
            let api_host = self.text_input(id!(api_host)).text();

            // Do not create a provider if the api host is already in the list
            if store.chats.providers.contains_key(&api_host) {
                // TODO(Julian): inform the user that the provider already exists
                eprintln!("Provider already exists: {}", api_host);
                return;
            }

            let api_key = self.text_input(id!(api_key)).text();
            let provider_type_idx = self.drop_down(id!(provider_type)).selected_item();

            let provider_type = ProviderType::from_usize(provider_type_idx);
            let _provider = match provider_type {
                ProviderType::OpenAI => {
                    Provider {
                        name: "OpenAI".to_string(),
                        url: api_host.clone(),
                        api_key: Some(api_key.clone()),
                        provider_type: ProviderType::OpenAI,
                        connection_status: ServerConnectionStatus::Disconnected,
                        enabled: true,
                        models: vec![],
                    }
                }
                ProviderType::MoFa => {
                    Provider {
                        name: "MoFa".to_string(),
                        url: api_host.clone(),
                        api_key: Some(api_key.clone()),
                        provider_type: ProviderType::MoFa,
                        connection_status: ServerConnectionStatus::Disconnected,
                        enabled: true,
                        models: vec![],
                    }
                }
            };

            // TODO(Julian): store the provider, provider dropdown not working
            // store.insert_or_update_provider(&provider);

            cx.action(AddProviderModalAction::ModalDismissed);
        }
    }
}
