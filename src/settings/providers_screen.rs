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

        // TODO(Julian): add this back in as a modal
        // <AddProvider> {}

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
                let provider = scope.data.get_mut::<Store>().unwrap().chats.providers.get(&address);
                if let Some(provider) = provider {
                    self.view.provider_view(id!(provider_view)).set_provider(cx, provider);
                } else {
                    eprintln!("Provider not found: {}", address);
                }
            }
        }
    }
}
