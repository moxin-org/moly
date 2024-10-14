use makepad_widgets::*;

use crate::data::store::{MoFaTestServerAction, Store};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")

    MofaSettings = {{MofaSettings}} {
        width: Fill, height: Fit
        flow: Down
        spacing: 20

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 16}
                color: #000
            }
            text: "MoFa options"
        }

        <HorizontalFiller> { height: 10 }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12}
                color: #000
            }
            text: "Server address"
        }

        address_on_edit = <View> {
            visible: false,
            width: Fill, height: Fit

            mofa_address_input = <MolyTextInput> {
                width: Fill
                height: Fit
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 12}
                    color: #000
                }
                text: "http://mofa-130.openllm.io"
            }
        }

        address_editable = <View> {
            width: Fit, height: Fit
            spacing: 10
            align: {x: 0.0, y: 0.5}

            mofa_address_label = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 12}
                    color: #000
                }
            }

            edit_mofa_address = <MolyButton> {
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

        mofa_status_label_status = <View> {
            visible: false,
            width: Fill, height: Fit
            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 10}
                    color: #000
                }
                text: "Checking MoFa server status..."
            }
        }

        mofa_status_label_success = <View> {
            visible: false,
            width: Fill, height: Fit
            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 10}
                    color: #099250
                }
                text: "MoFa server is running"
            }
        }

        mofa_status_label_failure = <View> {
            visible: false,
            width: Fill, height: Fit
            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 10}
                    color: #B42318
                }
                text: "MoFa server is not properly running. Please check the address." 
            } 
        }
    }
}

#[derive(Default, Debug, PartialEq)]
enum MofaAddressEditionState {
    OnEdit,
    #[default]
    Editable,
}

#[derive(Widget, LiveHook, Live)]
pub struct MofaSettings {
    #[deref]
    view: View,

    #[rust]
    address_edition_state: MofaAddressEditionState,
}

impl Widget for MofaSettings {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);


    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        match self.address_edition_state {
            MofaAddressEditionState::OnEdit => {
                self.view.view(id!(address_editable)).set_visible(false);
                self.view.view(id!(address_on_edit)).set_visible(true);
            }
            MofaAddressEditionState::Editable => {
                self.view.view(id!(address_editable)).set_visible(true);
                self.view.view(id!(address_on_edit)).set_visible(false);
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MofaSettings {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        for action in actions {
            match action.downcast_ref() {
                Some(MoFaTestServerAction::Success(url)) => {
                    self.view(id!(mofa_status_label_success)).set_visible(true);
                    self.view(id!(mofa_status_label_failure)).set_visible(false);
                    self.view(id!(mofa_status_label_status)).set_visible(false);
                    self.view.label(id!(mofa_address_label)).set_text(url);
                    self.redraw(cx);
                }
                Some(MoFaTestServerAction::Failure(url)) => {
                    self.view(id!(mofa_status_label_success)).set_visible(false);
                    self.view(id!(mofa_status_label_failure)).set_visible(true);
                    self.view(id!(mofa_status_label_status)).set_visible(false);
                    if let Some(url) = url {
                        self.view.label(id!(mofa_address_label)).set_text(url);
                    }
                    self.redraw(cx);
                }
                _ => {}
            }
        }

        let mofa_address_input = self.view.text_input(id!(mofa_address_input));

        if self.button(id!(edit_mofa_address)).clicked(actions) {
            self.address_edition_state = MofaAddressEditionState::OnEdit;

            let address = self.label(id!(mofa_address_label)).text();
            mofa_address_input.set_key_focus(cx);
            mofa_address_input.set_text(&address);

            self.redraw(cx);
        }

        if let Some(address) = mofa_address_input.returned(actions) {
            self.view(id!(mofa_status_label_success)).set_visible(false);
            self.view(id!(mofa_status_label_failure)).set_visible(false);
            self.view(id!(mofa_status_label_status)).set_visible(true);
            self.view.label(id!(mofa_address_label)).set_text(&address);
            store.set_mofa_server_address(address);

            self.address_edition_state = MofaAddressEditionState::Editable;
            self.redraw(cx);
        }

        if let TextInputAction::Escape =
            actions.find_widget_action_cast(mofa_address_input.widget_uid())
        {
            self.address_edition_state = MofaAddressEditionState::Editable;
            self.redraw(cx);
        }
    }
}
