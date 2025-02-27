use makepad_widgets::*;

use crate::data::store::Store;

use super::provider_view::ProviderViewWidgetExt;
use super::providers::ConnectionSettingsAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::modal::*;
    use crate::settings::delete_server_modal::DeleteServerModal;
    use crate::settings::configure_connection_modal::ConfigureConnectionModal;
    use crate::settings::provider_view::ProviderView;
    use crate::settings::providers::Providers;

    HorizontalSeparator =  <RoundedView> {
        width: 2, height: Fill
        show_bg: true
        draw_bg: {
            color: #d3d3d3
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

    AddProvider = <RoundedView> {
        flow: Right
        width: Fill, height: 55
        spacing: 10
        align: {x: 0.0, y: 0.5}
        padding: {left: 20, right: 20, top: 10, bottom: 10}
        margin: {right: 20}
        show_bg: true
        draw_bg: {
            color: #f
            radius: 3
            border_color: #e
            border_width: 1
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

            labels: ["OpenAI", "MoFa"]
            values: [OpenAIAPI, MoFa]
        }

        add_server_button = <MolyButton> {
            width: Fit
            height: Fill
            padding: {left: 20, right: 20, top: 0, bottom: 0}
            text: "Add Provider"
            draw_bg: { color: #099250, border_color: #099250 }
        }
    }

    pub ProvidersScreen = {{ProvidersScreen}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 20
        padding: {left: 30, top: 40}

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 25}
                color: #000
            }
            text: "Provider Settings"
        }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12}
                color: #000
            }
            text: "Manage providers and models"
        }

        <AddProvider> {}

        <View> {
            providers = <Providers> {}
            provider_view = <ProviderView> {}
        }
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct ProvidersScreen {
    #[deref]
    view: View,
}

impl Widget for ProvidersScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ProvidersScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for action in actions {
            if let ConnectionSettingsAction::ProviderSelected(address) = action.cast() {
                // fetch provider from store
                let provider = scope.data.get_mut::<Store>().unwrap().chats.get_provider_by_url(&address);
                if let Some(provider) = provider {
                    self.view.provider_view(id!(provider_view)).set_provider(cx, provider);
                } else {
                    eprintln!("Provider not found: {}", address);
                }
            }
        }
    }
}
