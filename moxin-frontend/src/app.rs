use makepad_widgets::*;
use crate::data::store::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::landing_screen::LandingScreen;

    ICON_EXPLORE = dep("crate://self/resources/icons/explore.svg")
    ICON_FOLDER = dep("crate://self/resources/icons/folder.svg")

    SidebarMenuButton = <RadioButton> {
        width: 70,
        height: 70,
        icon_walk: {width: 48, height: 48}
        flow: Down, spacing: 5.0, align: {x: 0.5, y: 0.5}
        draw_radio: {
            radio_type: Tab,
            color_active: #F2F4F7,
            color_inactive: #fff,
        }
        draw_icon: {
            fn get_color(self) -> vec4 {
                return mix(
                    #000,
                    #666,
                    self.hover
                )
            }
        }
    }

    // This is a placeholder for the actual My Models screen view
    MyModelsView = <View> {
        width: Fill,
        height: Fill,
        margin: 50,
        spacing: 30,

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 20},
                color: #000
            }
            text: "My Models"
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
                    flow: Down, spacing: 10.0,
                    padding: { top: 40, left: 30 }
                    tab1 = <SidebarMenuButton> {
                        animator: {selected = {default: on}}
                        draw_icon: {
                            svg_file: (ICON_EXPLORE),
                        }
                    }
                    tab2 = <SidebarMenuButton> {
                        draw_icon: {
                            svg_file: (ICON_FOLDER),
                        }
                    }
                }

                application_pages = <View> {
                    margin: 0.0,
                    padding: 0.0,

                    flow: Overlay,

                    width: Fill,
                    height: Fill,

                    tab1_frame = <LandingScreen> {visible: true}
                    tab2_frame = <MyModelsView> {visible: false}
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    store: Store,
}

impl LiveHook for App {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.store = Store::new();
    }
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);

        crate::shared::styles::live_design(cx);
        crate::shared::widgets::live_design(cx);
        crate::shared::icon::live_design(cx);

        crate::landing::shared::live_design(cx);
        crate::landing::model_files_list::live_design(cx);
        crate::landing::model_card::live_design(cx);
        crate::landing::model_list::live_design(cx);
        crate::landing::landing_screen::live_design(cx);
        crate::landing::search_bar::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        let scope = &mut Scope::with_data(&mut self.store);
        self.ui.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions){
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

        for action in actions.iter() {
            match action.as_widget_action().cast() {
                StoreAction::Search(keywords) => {
                    self.store.load_search_results(keywords);
                }
                StoreAction::ResetSearch => {
                    self.store.load_featured_models();
                }
                _ => {}
            }
        }
    }
}