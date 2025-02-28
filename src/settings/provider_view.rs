use makepad_widgets::*;

use crate::data::{chats::Provider, remote_servers::RemoteModelId, store::Store};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::widgets::*;
    use crate::shared::styles::*;

    FormGroup = <View> {
        flow: Down
        height: Fit
        spacing: 10
    }

    ModelEntry = {{ModelEntry}} {
        align: {x: 0.5, y: 0.5}
        width: Fill, height: 60
        flow: Down,
        separator = <View> {
            margin: {left: 10, right: 10, top: 0, bottom: 10}
            height: 1,
            show_bg: true,
            draw_bg: {
                color: #D9D9D9
            }
        }

        content =<View> {
            flow: Right,
            width: Fill, height: Fill
            padding: {top: 10, bottom: 10, left: 20, right: 20}
            align: {x: 0.5, y: 0.5}
            model_name = <Label> {
                text: "Model Name"
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 11},
                    color: #000
                }
            }

            vertical_filler = <View> {
                width: Fill, height: 1
            }

            enabled_toggle = <View> {
                flow: Right
                height: Fit, width: Fill
                align: {x: 1.0, y: 0.5}
                spacing: 20
                enabled_switch = <MolySwitch> {
                    // Match the default value to avoid the animation on start.
                    animator: {
                        selected = {
                            default: on
                        }
                    }
                }
            }
        }
    }

    pub ProviderView = {{ProviderView}} {
        width: Fill, height: Fill
        // align: {x: 0.0, y: 0.0}
        padding: {left: 30, right: 30, top: 30, bottom: 30}
        show_bg: true
        draw_bg: { color: (SIDEBAR_BG_COLOR) }

        content = <View> {
            visible: false
            flow: Down, spacing: 20

            <FormGroup> {
                flow: Right,
                name =<Label> {
                    text: "OpenAI"
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 15}
                        color: #000
                    }
                }

                <View> {
                    width: Fill, height: 1
                }

                enabled_switch = <MolySwitch> {
                    // Match the default value to avoid the animation on start.
                    animator: {
                        selected = {
                            default: on
                        }
                    }
                }
            }


            separator = <View> {
                height: 1,
                show_bg: true,
                draw_bg: {
                    color: #D9D9D9
                }
            }
    
            // HOST
            <FormGroup> {
                <Label> {
                    text: "API Host"
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 12}
                        color: #000
                    }
                }
                
                api_host = <MolyTextInput> {
                    width: Fill
                    text: "https://some-api.com"
                    is_read_only: true
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 12}
                        color: #000
                    }
                }
            }
    
            // API KEY
            <FormGroup> {
                <Label> {
                    text: "API Key"
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 12}
                        color: #000
                    }
                }
        
                api_key = <MolyTextInput> {
                    width: Fill
                    draw_text: {
                        text_style: <REGULAR_FONT>{
                            font_size: 12
                            is_secret: true
                        }
                        color: #000 
                    }
                }
            }

            // MODELS
            <Label> {
                text: "Models"
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 12}
                    color: #000
                }
            }
    
            <RoundedView> {
                show_bg: true
                draw_bg: { 
                    color: #f
                    // radius: 3
                }
                padding: 10
                models_list = <PortalList> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    spacing: 10,
    
                    model_entry = <ModelEntry> {}
                }
            }
        }
    }
}

// TODO: Rename into ProviderView
#[derive(Widget, LiveHook, Live)]
struct ProviderView {
    #[deref]
    view: View,

    #[rust]
    provider: Provider,
}

impl Widget for ProviderView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();

        let models = store.chats.get_provider_models(&self.provider.url);

        self.view.text_input(id!(api_host)).set_text(cx, &self.provider.url);
        self.view.text_input(id!(api_key)).set_text(cx, &self.provider.api_key.clone().unwrap_or("no key".to_string()));

        let entries_count = models.len();
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, entries_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < entries_count {
                        let template = live_id!(model_entry);
                        let item = list.item(cx, item_id, template);

                        // hide the separator for the first item
                        if item_id == 0 {
                            item.view(id!(separator)).set_visible(cx, false);
                        }

                        // Trim the 'models/' prefix from Gemini models
                        let name = models[item_id].name.trim_start_matches("models/");
                        item.label(id!(model_name)).set_text(cx, &name);
                        item.check_box(id!(enabled_switch)).set_selected(cx, models[item_id].enabled);

                        item.as_model_entry().set_model_name(&models[item_id].name);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for ProviderView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // TODO(Julian): handle when a provider is enabled/disabled
        // Re-register this provider if enabled
        // if new_enabled && new_api_key.is_some() {
        //     if let Some(mut provider) = store.chats.providers.get_mut(&url) {
        //         provider.api_key = new_api_key; // update the in-memory struct
        //         // Now call register_provider to build the client and fetch
        //         let clone = provider.clone();
        //         store.chats.register_provider(clone);
        //     }
        // }

        for action in actions {
            if let Some(action) = action.downcast_ref::<ModelEntryAction>() {
                match action {
                    ModelEntryAction::ModelEnabledChanged(model_name, enabled) => {
                        // Update the model status in the preferences
                        let store = scope.data.get_mut::<Store>().unwrap();
                        store
                            .preferences
                            .update_model_status(&self.provider.url, model_name, *enabled);

                        // Update the model status in the store
                        if let Some(model) = store.chats.remote_models.get_mut(
                            &RemoteModelId::from_model_and_server(
                                model_name,
                                &self.provider.url),
                        ) {
                            model.enabled = *enabled;
                        }
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
        }
    }    
}


impl ProviderViewRef {
    pub fn set_provider(&mut self, cx: &mut Cx, provider: &Provider) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.provider = provider.clone();
            inner.view(id!(content)).set_visible(cx, true);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
struct ModelEntry {
    #[deref]
    view: View,

    #[rust]
    model_name: String,
}

impl Widget for ModelEntry {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

// MODEL ENTRY

impl WidgetMatchEvent for ModelEntry {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let enabled_switch = self.check_box(id!(enabled_switch));
        if let Some(change) = enabled_switch.changed(actions) {
            cx.action(ModelEntryAction::ModelEnabledChanged(
                self.model_name.clone(),
                change,
            ));
            self.redraw(cx);
        }
    }
}

impl ModelEntryRef {
    pub fn set_model_name(&mut self, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.model_name = name.to_string();
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
enum ModelEntryAction {
    None,
    ModelEnabledChanged(String, bool),
}
