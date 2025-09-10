use makepad_widgets::*;
use moly_kit::BotId;

use crate::data::{
    providers::{Provider, ProviderConnectionStatus, ProviderType},
    store::Store,
};

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
            padding: {top: 8, bottom: 10, left: 15, right: 15}
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

    pub ProviderView = {{ProviderView}}<RoundedShadowView> {
        width: Fill, height: Fill
        // align: {x: 0.0, y: 0.0}
        padding: {left: 30, right: 30, top: 30, bottom: 30}
        show_bg: true
        draw_bg: {
            color: (MAIN_BG_COLOR_DARK)
            border_radius: 4.5,
            uniform shadow_color: #0002
            shadow_radius: 8.0,
            shadow_offset: vec2(0.0,-1.5)
        }

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
                        visible: false
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

                <View> {
                    spacing: 10
                    width: Fill, height: 35
                    api_host = <MolyTextInput> {
                        width: Fill, height: 30
                        text: "https://some-api.com/v1"
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 12}
                            color: #000
                        }
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
                    width: Fill, height: 35
                    api_key = <MolyTextInput> {
                        empty_text: ""
                        width: Fill, height: 30
                        is_password: true
                        draw_text: {
                            text_style: <REGULAR_FONT>{
                                font_size: 12
                            }
                            color: #000
                        }
                    }
                    // save_api_key = <MolyButton> {
                    //     width: Fit
                    //     height: 30
                    //     padding: {left: 20, right: 20, top: 0, bottom: 0}
                    //     text: "Save"
                    //     draw_bg: { color: (CTA_BUTTON_COLOR), border_size: 0 }
                    // }
                }
                <View> {
                    width: Fill, height: Fit
                    align: {x: 0.0, y: 0.5}
                    connection_status = <Label> {
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 10},
                            color: #000
                        }
                    }
                }
            }

            // SYSTEM PROMPT
            system_prompt_group = <FormGroup> {
                height: Fit
                visible: false
                <Label> {
                    text: "System Prompt"
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 12}
                        color: #000
                    }
                }

                <View> {
                    height: 85
                    scroll_bars: <ScrollBars> {
                        show_scroll_x: false, show_scroll_y: true
                        scroll_bar_y: {
                            draw_bg: {
                                color: #D9
                                color_hover: #888
                                color_drag: #777
                            }
                        }
                    }
                    system_prompt = <MolyTextInput> {
                        width: Fill, height: Fit
                        padding: 8
                        empty_text: "Optional: enter a custom system prompt.
When using a custom prompt, we recommend including the language you'd like to be greeted on, knowledge cutoff, and tool usage eagerness.
Moly automatically appends useful context to your prompt, like the time of day."
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 11}
                        }
                    }
                }
            }

            save_provider = <MolyButton> {
                width: Fit
                height: 30
                padding: {left: 20, right: 20, top: 0, bottom: 0}
                text: "Save"
                draw_bg: { color: (CTA_BUTTON_COLOR), border_size: 0 }
            }


            // TOOLS ENABLED
            tools_form_group = <FormGroup> {
                visible: false
                height: Fit

                <View> {
                    width: Fill, height: 1
                    margin: {bottom: 10}
                    show_bg: true,
                    draw_bg: {
                        color: #D9D9D9
                    }
                }

                <Label> {
                    text: "MCP Configuration"
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 12}
                        color: #000
                    }
                }

                <View> {
                    flow: Right, spacing: 12
                    width: Fit, height: Fit
                    align: {x: 0.5, y: 0.5}
                    <Label> {
                        text: "Enable Tools"
                        draw_text: {
                            text_style: {font_size: 12}
                            color: #000
                        }
                    }

                    provider_tools_switch = <MolySwitch> {
                        // Match the default value to avoid the animation on start.
                        animator: {
                            selected = {
                                default: on
                            }
                        }
                    }
                }

                <View> {
                    width: Fill, height: 1
                    margin: {top: 10}
                    show_bg: true,
                    draw_bg: {
                        color: #D9D9D9
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

            models_view = <RoundedView> {
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
                    draw_bg: { color: #B4605A, border_size: 0 }
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

    #[rust]
    initialized: bool,
}

impl Widget for ProviderView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let models = store.chats.get_provider_models(&self.provider.id);

        let provider = store.chats.providers.get(&self.provider.id).cloned();

        if let Some(provider) = provider {
            if !self.initialized {
                // Full sync on first initialization
                self.provider = provider;
                self.initialized = true;
            } else {
                // Only sync the connection status on subsequent draws
                self.provider.connection_status = provider.connection_status;
            }
        }

        self.update_connection_status(cx);

        if self.provider.enabled {
            self.view(id!(refresh_button)).set_visible(cx, true);
        } else {
            self.view(id!(refresh_button)).set_visible(cx, false);
        }

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

                        let name = models[item_id].human_readable_name();
                        item.label(id!(model_name)).set_text(cx, &name);
                        item.check_box(id!(enabled_switch))
                            .set_active(cx, models[item_id].enabled && self.provider.enabled);

                        item.as_model_entry().set_model_name(&models[item_id].name);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl ProviderView {
    fn update_connection_status(&mut self, cx: &mut Cx) {
        let connection_status_label = self.label(id!(connection_status));
        connection_status_label.set_text(cx, &self.provider.connection_status.to_human_readable());
        let text_color = match &self.provider.connection_status {
            ProviderConnectionStatus::Connected => {
                // green
                vec4(0.0, 0.576, 0.314, 1.0)
            }
            ProviderConnectionStatus::Disconnected => {
                // black
                vec4(0.0, 0.0, 0.0, 1.0)
            }
            ProviderConnectionStatus::Connecting => {
                // gray
                vec4(0.5, 0.5, 0.5, 1.0)
            }
            ProviderConnectionStatus::Error(_error) => {
                // red
                vec4(1.0, 0.0, 0.0, 1.0)
            }
        };
        connection_status_label.apply_over(
            cx,
            live! {
                draw_text: {
                    color: (text_color)
                }
            },
        );
    }
}

impl WidgetMatchEvent for ProviderView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        // Handle provider enabled/disabled
        let provider_enabled_switch = self.check_box(id!(provider_enabled_switch));
        if let Some(enabled) = provider_enabled_switch.changed(actions) {
            self.provider.enabled = enabled;
            // Update the provider in store and preferences
            store.insert_or_update_provider(&self.provider);
            self.redraw(cx);
        }

        // Handle tools enabled/disabled
        let provider_tools_switch = self.check_box(id!(provider_tools_switch));
        if let Some(tools_enabled) = provider_tools_switch.changed(actions) {
            self.provider.tools_enabled = tools_enabled;
            // Update the provider in store and preferences
            store.insert_or_update_provider(&self.provider);
            self.redraw(cx);
        }

        for action in actions {
            if let Some(action) = action.downcast_ref::<ModelEntryAction>() {
                match action {
                    ModelEntryAction::ModelEnabledChanged(model_name, enabled) => {
                        // Update the model status in the preferences
                        store.preferences.update_model_status(
                            &self.provider.id,
                            model_name,
                            *enabled,
                        );

                        // Update the model status in the store
                        if let Some(model) = store
                            .chats
                            .available_bots
                            .get_mut(&BotId::new(model_name, &self.provider.url))
                        {
                            model.enabled = *enabled;
                        }
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
        }

        // Handle save
        if self.button(id!(save_provider)).clicked(actions) {
            self.provider.url = self
                .view
                .text_input(id!(api_host))
                .text()
                .trim()
                .to_string();
            let api_key = self.view.text_input(id!(api_key)).text().trim().to_string();
            if api_key.is_empty() {
                self.provider.api_key = None;
            } else {
                self.provider.api_key = Some(api_key);
            }

            // Save system prompt for Realtime providers
            if self.provider.provider_type == ProviderType::OpenAIRealtime {
                let system_prompt = self
                    .view
                    .text_input(id!(system_prompt))
                    .text()
                    .trim()
                    .to_string();
                if system_prompt.is_empty() {
                    self.provider.system_prompt = None;
                } else {
                    self.provider.system_prompt = Some(system_prompt);
                }
            }

            // Since we auto-fetch the models upon update, also enable it
            self.provider.enabled = true;
            // Clear any previous error state and set to connecting
            self.provider.connection_status = ProviderConnectionStatus::Connecting;
            self.check_box(id!(provider_enabled_switch))
                .set_active(cx, true);
            // Keep the tools_enabled state as set by the user (don't change it on save)

            // Update the provider in the store first to ensure the connecting status is saved
            store.insert_or_update_provider(&self.provider);

            // Update UI immediately to show "Connecting..." status
            self.update_connection_status(cx);
            self.redraw(cx);
        }

        // Handle refresh button
        if let Some(_fe) = self.view(id!(refresh_button)).finger_up(actions) {
            // Clear any previous error state and set to connecting
            self.provider.connection_status = ProviderConnectionStatus::Connecting;

            // Update the provider in the store first to ensure the connecting status is saved
            store.insert_or_update_provider(&self.provider);

            // Update UI immediately to show "Connecting..." status
            self.update_connection_status(cx);
            self.redraw(cx);
        }

        // Handle remove provider button
        if self.button(id!(remove_provider_button)).clicked(actions) {
            store.remove_provider(&self.provider.id);
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
                api_key_input.set_text(cx, &api_key);
            } else {
                api_key_input.set_text(cx, "");
            }

            inner.text_input(id!(api_host)).set_text(cx, &provider.url);
            inner.label(id!(name)).set_text(cx, &provider.name);
            inner
                .check_box(id!(provider_enabled_switch))
                .set_active(cx, provider.enabled);
            inner
                .check_box(id!(provider_tools_switch))
                .set_active(cx, provider.tools_enabled);

            // Show/hide system prompt field for Realtime providers
            if provider.provider_type == ProviderType::OpenAIRealtime {
                inner.view(id!(system_prompt_group)).set_visible(cx, true);
                if let Some(system_prompt) = &provider.system_prompt {
                    inner
                        .text_input(id!(system_prompt))
                        .set_text(cx, &system_prompt);
                } else {
                    inner.text_input(id!(system_prompt)).set_text(cx, "");
                }
            } else {
                inner.view(id!(system_prompt_group)).set_visible(cx, false);
            }

            if provider.provider_type == ProviderType::OpenAIRealtime
                || provider.provider_type == ProviderType::OpenAI
            {
                inner.view(id!(tools_form_group)).set_visible(cx, true);
            } else {
                inner.view(id!(tools_form_group)).set_visible(cx, false);
            }

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
    ProviderRemoved,
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
        // TODO: Do this using AdaptiveView instead, currently here because there PortalList
        // does not support height: Fit on children, and there's also no proper text wrapping.
        if cx.display_context.is_desktop() {
            self.apply_over(
                cx,
                live! {
                    height: 60
                    content = { model_name = { width: Fit } }
                    vertical_filler = { visible: true }
                },
            );
        } else {
            self.apply_over(
                cx,
                live! {
                    height: 80
                    content = { model_name = { width: 200 } }
                    vertical_filler = { visible: false }
                },
            );
        }

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
