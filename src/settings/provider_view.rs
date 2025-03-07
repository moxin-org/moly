use makepad_widgets::*;

use crate::data::{chats::Provider, remote_servers::RemoteModelId, store::Store};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::widgets::*;
    use crate::shared::styles::*;

    REFRESH_ICON = dep("crate://self/resources/images/refresh_icon.png")

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

        content = <View> {
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
            flow: Down, spacing: 20

            <FormGroup> {
                flow: Right,
                name = <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 15}
                        color: #000
                    }
                }

                <View> {
                    width: Fill, height: 1
                }

                <View> {
                    align: {x: 0.5, y: 0.5}
                    width: Fit, height: Fit
                    flow: Right, spacing: 10
                    refresh_button = <View> {
                        padding: {top: 4} // TODO: this is a hack to align the image view with the switch
                        cursor: Hand
                        width: 30, height: 30
                        
                        icon = <Image> {
                            width: 22, height: 22
                            source: (REFRESH_ICON)
                        }
                    }
                    provider_enabled_switch = <MolySwitch> {
                        // Match the default value to avoid the animation on start.
                        animator: {
                            selected = {
                                default: on
                            }
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
        
                <View> {
                    spacing: 10
                    width: Fill, height: 30
                    api_key = <MolyTextInput> {
                        empty_message: ""
                        width: Fill, height: 30
                        draw_text: {
                            text_style: <REGULAR_FONT>{
                                font_size: 12
                                is_secret: true
                            }
                            color: #000 
                        }
                    }
                    save_api_key = <MolyButton> {
                        width: Fit
                        height: 30
                        padding: {left: 20, right: 20, top: 0, bottom: 0}
                        text: "Save"
                        draw_bg: { color: #099250, border_color: #099250 }
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

            remove_provider_view = <View> {
                width: Fill, height: Fit
                align: {x: 1.0, y: 0.5}
                remove_provider_button = <MolyButton> {
                    padding: {left: 20, right: 20, top: 10, bottom: 10}
                    width: Fit, height: Fit
                    text: "Remove Provider"
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 10}
                    }
                    draw_bg: { color: #f00, border_color: #f00 }
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
        let store = scope.data.get_mut::<Store>().unwrap();
        // Handle provider enabled/disabled
        let provider_enabled_switch = self.check_box(id!(provider_enabled_switch));
        if let Some(enabled) = provider_enabled_switch.changed(actions) {
            if enabled {
                store.chats.test_provider_and_fetch_models(&self.provider.url);
            }

            self.provider.enabled = enabled;
            // TODO(Julian): unify the update_provider_enabled method in the store
            // Update the provider in preferences
            store.preferences.update_provider_enabled(&self.provider.url, enabled);
            // Update the provider in the store
            store.chats.update_provider_enabled(&self.provider.url, enabled);
            // TODO: this is a hack to force a redraw of the chat panel, this will be replaced by integration with MolyKit
            cx.redraw_all();
        }

        for action in actions {
            if let Some(action) = action.downcast_ref::<ModelEntryAction>() {
                match action {
                    ModelEntryAction::ModelEnabledChanged(model_name, enabled) => {
                        // Update the model status in the preferences
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

        // Handle API Key save
        if self.button(id!(save_api_key)).clicked(actions) {
            let should_fetch_models = self.provider.api_key.is_none();
            self.provider.api_key = Some(self.view.text_input(id!(api_key)).text());

            // Update the provider in the store and preferences
            store.insert_or_update_provider(&self.provider);

            if should_fetch_models {
                store.chats.test_provider_and_fetch_models(&self.provider.url);
            }
        }

        // Handle refresh button
        if let Some(_fe) = self.view(id!(refresh_button)).finger_up(actions) {
            store.chats.test_provider_and_fetch_models(&self.provider.url);
        }

        // Handle remove provider button
        if self.button(id!(remove_provider_button)).clicked(actions) {
            println!("Removing provider: {}", self.provider.url);
            store.remove_provider(&self.provider.url);
            cx.action(ProviderViewAction::ProviderRemoved);
            self.redraw(cx);
        }
    }    
}

impl ProviderViewRef {
    pub fn set_provider(&mut self, cx: &mut Cx, provider: &Provider) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.provider = provider.clone();
            
            // Update the text inputs
            let api_key_input = inner.text_input(id!(api_key));
            if let Some(api_key) = &provider.api_key {
                api_key_input.set_text(cx, api_key);
            } else {
                api_key_input.set_text(cx, "");
            }

            inner.text_input(id!(api_host)).set_text(cx, &provider.url);
            inner.label(id!(name)).set_text(cx, &provider.name);
            inner.check_box(id!(provider_enabled_switch)).set_selected(cx, provider.enabled);

            if provider.was_customly_added {
                inner.view(id!(remove_provider_view)).set_visible(cx, true);
            } else {
                inner.view(id!(remove_provider_view)).set_visible(cx, false);
            }

            inner.view.redraw(cx);
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum ProviderViewAction {
    None,
    ProviderRemoved
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

impl WidgetMatchEvent for ModelEntry {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Handle the enabled switch
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
