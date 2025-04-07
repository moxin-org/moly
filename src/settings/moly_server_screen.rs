use makepad_widgets::*;

use crate::{app::NavigationAction, data::store::Store};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::modal::*;

    use crate::landing::landing_screen::LandingScreen;
    use crate::my_models::my_models_screen::MyModelsScreen;

    ICON_DISCOVER = dep("crate://self/resources/icons/discover.svg")
    ICON_MY_MODELS = dep("crate://self/resources/icons/my_models.svg")
    ICON_CLOUD = dep("crate://self/resources/icons/cloud.svg")
    ICON_RETRY = dep("crate://self/resources/icons/retry.svg")

    SUBSIDEBAR_BG_COLOR = (MAIN_BG_COLOR)
    SUBSIDEBAR_BG_COLOR_HOVER = #ebedee
    SUBSIDEBAR_BG_COLOR_SELECTED = #ebedee
    
    SUBSIDEBAR_FONT_COLOR = #2C3E50
    SUBSIDEBAR_FONT_COLOR_HOVER = #2C3E50
    SUBSIDEBAR_FONT_COLOR_SELECTED = #344054

    SubSidebarMenuButton = <SidebarMenuButton> {
        width: Fill, height: Fit,
        padding: {top: 8, bottom: 8, left: 15},
        flow: Right
        align: {x: 0.0, y: 0.5}

        icon_walk: {margin: 0, width: 22, height: 22}

        draw_bg: {
            radio_type: Tab,

            instance border_size: 0.0
            instance border_color_1: #0000
            instance inset: vec4(0.0, 0.0, 0.0, 0.0)
            instance border_radius: 2.5

            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        (SUBSIDEBAR_BG_COLOR),
                        (SUBSIDEBAR_BG_COLOR_HOVER),
                        self.hover
                    ),
                    (SUBSIDEBAR_BG_COLOR_SELECTED),
                    self.active
                )
            }
        }

        draw_text: {
            color: (SUBSIDEBAR_FONT_COLOR)
            color_hover: (SUBSIDEBAR_FONT_COLOR_HOVER)
            color_active: (SUBSIDEBAR_FONT_COLOR_SELECTED)

            text_style: <REGULAR_FONT>{font_size: 10}
        }

        draw_icon: {
            instance color: (SUBSIDEBAR_FONT_COLOR)
            instance color_hover: (SUBSIDEBAR_FONT_COLOR_HOVER)
            instance color_active: (SUBSIDEBAR_FONT_COLOR_SELECTED)
        }
    }

    MolyServerNotAccesible = <View> {
        visible: false
        padding: {left: 30, top: 40}
        spacing: 50
        flow: Down

        header = <View> {
            height: Fit
            flow: Down, spacing: 40
            <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 25}
                    color: #000
                }
                text: "MolyServer (disconnected)"
            }
    
            <Label> {
                width: Fill, height: Fit
                draw_text:{
                    text_style: {font_size: 12}
                    color: #000
                }
                text: "MolyServer is a local HTTP server that powers the Moly app by providing capabilities for searching, downloading, and running local LLMs).\nYou can install MolyServer by following the instructions in https://github.com/moxin-org/moly-server."
            }
        }

        <View> {
            height: Fit, width: Fill
            spacing: 20
            flow: Down
            <Label> {
                width: Fit, height: Fit
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 12}
                    color: #000
                }
                text: "Could not reach MolyServer.\nPlease check if its running, and verify your provider settings."
            }

            <View> {
                width: Fill, height: Fit
                spacing: 8
                go_to_providers = <MolyButton> {
                    draw_bg: { color: #099250, border_size: 0}
                    draw_icon: {
                        svg_file: (ICON_CLOUD),
                    }
                    draw_text: {
                        text_style: <BOLD_FONT>{ font_size: 10}
                    }
                    text: "Go to Providers"
                }
                refresh = <MolyButton> {
                    draw_bg: { color: #099250, border_size: 0}
                    draw_icon: {
                        svg_file: (ICON_RETRY),
                    }
                    draw_text: {
                        text_style: <BOLD_FONT>{ font_size: 10}
                    }
                    text: "refresh"
                }
            }
        }
    }

    pub MolyServerScreen = {{MolyServerScreen}} {
        show_bg: true,
        main_content = <View> {
            visible: false
            menu = <RoundedView> {
                width: 130, height: Fill,
                flow: Down,
                padding: { top: 50, bottom: 20, left: 5, right: 8 },
    
                show_bg: true,
                draw_bg: {
                    color: (SUBSIDEBAR_BG_COLOR),
                    instance border_radius: 0.0,
                }
    
                discover_tab = <SubSidebarMenuButton> {
                    animator: {active = {default: on}}
                    text: "Discover",
                    draw_icon: {
                        svg_file: (ICON_DISCOVER),
                    }
                }
                my_models_tab = <SubSidebarMenuButton> {
                    text: "My Models",
                    draw_icon: {
                        svg_file: (ICON_MY_MODELS),
                    }
                }
            }
    
            right_border = <View> {
                width: 1.6, height: Fill
                margin: {top: 15, bottom: 15}
                show_bg: true,
                draw_bg: {
                    color: #eaeaea
                }
            }
    
            pages = <View> {
                discover_frame = <View> { visible: true, <LandingScreen> {} }
                my_models_frame = <View> { visible: false, <MyModelsScreen> {} }
            }
        }

        server_not_accessible = <MolyServerNotAccesible> {}
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct MolyServerScreen {
    #[deref]
    view: View,
}

impl Widget for MolyServerScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        if store.is_moly_server_connected {
            self.view(id!(server_not_accessible)).set_visible(cx, false);
            self.view(id!(main_content)).set_visible(cx, true);
        } else {
            self.view(id!(server_not_accessible)).set_visible(cx, true);
            self.view(id!(main_content)).set_visible(cx, false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MolyServerScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // RadioButton's selected_to_visible does not seem to work at this level
        // So we're manually checking the selected index and setting the visibility of the pages manually
        let selected_index = self.radio_button_set(ids!(menu.discover_tab, menu.my_models_tab)).selected(cx, actions);

        let discover_frame = self.view(id!(pages.discover_frame));
        let my_models_frame = self.view(id!(pages.my_models_frame));

        match selected_index {
            Some(0) => {
                discover_frame.set_visible(cx, true);
                my_models_frame.set_visible(cx, false);
                self.redraw(cx);
            }
            Some(1) => {
                discover_frame.set_visible(cx, false);
                my_models_frame.set_visible(cx, true);
                self.redraw(cx);
            }
            _ => (),
        }


        if self.button(id!(go_to_providers)).clicked(actions) {
            cx.action(NavigationAction::NavigateToProviders);
        }
        if self.button(id!(refresh)).clicked(actions) {
            let store = scope.data.get_mut::<Store>().unwrap();
            store.sync_with_moly_server();
        }
    }
}
