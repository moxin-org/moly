use crate::data::{remote_servers::RemoteModelId, store::Store};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::MolyButton;
    use crate::shared::widgets::MolySwitch;
    use crate::shared::resource_imports::*;


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
    pub ConfigureConnectionModal = {{ConfigureConnectionModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 600, height: 600
            padding: {top: 44, right: 30 bottom: 30 left: 50}
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                radius: 3
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Right

                padding: {top: 8, bottom: 20}

                title = <View> {
                    width: Fit,
                    height: Fit,

                    model_name = <Label> {
                        text: "Configure Connection"
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
                width: Fill,
                height: Fill,
                flow: Down,
                spacing: 40,

                <Label> {
                    text: "You can disable models to prevent them from showing up in the chat view as options."
                    width: Fill
                    draw_text: {
                        text_style: <REGULAR_FONT>{
                            font_size: 10,
                            height_factor: 1.3
                        },
                        color: #000
                        wrap: Word
                    }
                }

                list = <PortalList> {
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

#[derive(Clone, Debug, DefaultNone)]
pub enum ConfigureConnectionModalAction {
    None,
    ModalDismissed,
}

#[derive(Live, LiveHook, Widget)]
pub struct ConfigureConnectionModal {
    #[deref]
    view: View,

    #[rust]
    address: String,
}

impl Widget for ConfigureConnectionModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();

        let models = store.chats.get_remote_models_list_for_server(&self.address);

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

impl WidgetMatchEvent for ConfigureConnectionModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.button(id!(close_button)).clicked(actions) {
            cx.action(ConfigureConnectionModalAction::ModalDismissed);
        }

        for action in actions {
            if let Some(action) = action.downcast_ref::<ModelEntryAction>() {
                match action {
                    ModelEntryAction::ModelEnabledChanged(model_name, enabled) => {
                        // Update the model status in the preferences
                        let store = scope.data.get_mut::<Store>().unwrap();
                        store
                            .preferences
                            .update_model_status(&self.address, model_name, *enabled);

                        // Update the model status in the store
                        if let Some(model) = store.chats.remote_models.get_mut(
                            &RemoteModelId::from_model_and_server(model_name, &self.address),
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

impl ConfigureConnectionModalRef {
    pub fn set_server_address(&mut self, address: String) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.address = address;
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
