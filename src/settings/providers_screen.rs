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

    HorizontalSeparator = <RoundedView> {
        width: 2, height: Fill
        show_bg: true
        draw_bg: {
            color: #d3d3d3
        }
    }

    pub ProvidersScreen = {{ProvidersScreen}} {
        width: Fill, height: Fill
        spacing: 20
        flow: Down

        header = <View> {
            height: Fit
            spacing: 20
            flow: Down

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
        }

        adaptive_view =<AdaptiveView> {
            Desktop = {
                spacing: 10
                padding: {top: 10}
                providers = <Providers> {}
                provider_view = <ProviderView> {}
            }

            Mobile = {
                navigation = <StackNavigation> {
                    width: Fill, height: Fill
                    root_view = {
                        width: Fill, height: Fill
                        providers = <Providers> {
                            width: Fill, height: Fill
                        }
                    }

                    provider_navigation_view = <StackNavigationView> {
                        width: Fill, height: Fill
                        header = {
                            height: 300
                            content = {
                                button_container = {
                                    padding: {left: 14}
                                }
                                title_container = {
                                    show_bg: true
                                    draw_bg: { color: #x0 }
                                    title = {
                                        text: "Provider Settings"
                                        draw_text: {
                                            color: #x0
                                        }
                                    }
                                }
                            }
                        }
                        body = {
                            provider_view = <ProviderView> {
                                width: Fill, height: Fill
                            }
                        }
                    }
                }
            }
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
        let stack_navigation = self.stack_navigation(id!(navigation));
        stack_navigation.handle_stack_view_actions(cx, actions);

        for action in actions {
            if let ConnectionSettingsAction::ProviderSelected(address) = action.cast() {
                stack_navigation.show_stack_view_by_id(live_id!(provider_navigation_view), cx);

                // fetch provider from store
                let provider = scope
                    .data
                    .get_mut::<Store>()
                    .unwrap()
                    .chats
                    .providers
                    .get(&address);
                if let Some(provider) = provider {
                    self.view
                        .provider_view(id!(provider_view))
                        .set_provider(cx, provider);
                } else {
                    eprintln!("Provider not found: {}", address);
                }
            }
        }
    }
}
