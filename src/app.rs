use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::landing::landing_screen::LandingScreen;

    ICON_EXPLORE = dep("crate://self/resources/icons/explore.svg")
    ICON_FOLDER = dep("crate://self/resources/icons/folder.svg")

    SidebarMenuButton = <RadioButton> {
        width: Fit,
        height: Fit,
        align: {x: 0.0, y: 0.0}
        draw_radio: {
            radio_type: Tab,
            color_active: #fff,
            color_inactive: #fff,
        }
    }

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1280, 1000)},
            pass: {clear_color: #fff}

            body = {
                width: Fill,
                height: Fill,

                sidebar_menu = <View> {
                    width: 100,
                    flow: Down, spacing: 30.0,
                    padding: { top: 60, left: 40 }
                    tab1 = <SidebarMenuButton> {
                        animator: {selected = {default: on}}
                        draw_icon: {
                            svg_file: (ICON_EXPLORE),
                            fn get_color(self) -> vec4 {
                                return mix(
                                    #000,
                                    #666,
                                    self.hover
                                )
                            }
                        }
                        width: Fill,
                        icon_walk: {width: 48, height: 48}
                        flow: Down, spacing: 5.0, align: {x: 0.5, y: 0.5}
                    }
                    tab2 = <SidebarMenuButton> {
                        draw_icon: {
                            svg_file: (ICON_FOLDER),
                            fn get_color(self) -> vec4 {
                                return mix(
                                    #000,
                                    #666,
                                    self.hover
                                )
                            }
                        }
                        width: Fill
                        icon_walk: {width: 48, height: 48}
                        flow: Down, spacing: 5.0, align: {x: 0.5, y: 0.5}
                    }
                }

                application_pages = <View> {
                    margin: 0.0,
                    padding: 0.0,

                    width: Fill,
                    height: Fill,

                    tab1_frame = <LandingScreen> {visible: true}
                    tab2_frame = <View> {visible: false}
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);

        crate::shared::styles::live_design(cx);
        crate::shared::widgets::live_design(cx);
        crate::shared::icon::live_design(cx);

        crate::landing::model_card::live_design(cx);
        crate::landing::model_list::live_design(cx);
        crate::landing::model_all_files::live_design(cx);
        crate::landing::landing_screen::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx:&mut Cx, actions: &Actions){
        self.ui.radio_button_set(ids!(
            sidebar_menu.tab1,
            sidebar_menu.tab2,
        ))
        .selected_to_visible(
            cx,
            &self.ui,
            &actions,
            ids!(
                application_pages.tab1_frame,
                application_pages.tab2_frame,
            ),
        );
    }
}