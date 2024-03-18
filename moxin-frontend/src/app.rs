use makepad_widgets::*;
use crate::data::store::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::landing_screen::LandingScreen;
    import crate::chat::chat_screen::ChatScreen;

    ICON_DISCOVER = dep("crate://self/resources/icons/discover.svg")
    ICON_CHAT = dep("crate://self/resources/icons/chat.svg")
    ICON_MY_MODELS = dep("crate://self/resources/icons/my_models.svg")

    SidebarMenuButton = <RadioButton> {
        width: 60,
        height: 60,
        icon_walk: {width: 32, height: 32}
        flow: Down, spacing: 5.0, align: {x: 0.5, y: 0.5}
        draw_radio: {
            radio_type: Tab,
            color_active: #EDEEF0,
            color_inactive: #EDEEF0,
        }
        draw_icon: {
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        #344054,
                        #636e82,
                        self.hover
                    ),
                    #B258DD,
                    self.selected
                )
            }
        }
        draw_text: {
            color_selected: #B258DD,
            color_unselected: #344054,
            color_unselected_hover: #636e82,
            text_style: <REGULAR_FONT> {font_size: 8}
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
                    flow: Down, spacing: 20.0,
                    padding: { top: 80 }

                    align: {x: 0.5, y: 0.0},

                    show_bg: true,
                    draw_bg: {
                        color: #EDEEF0,
                    }

                    tab1 = <SidebarMenuButton> {
                        animator: {selected = {default: on}}
                        label: "Discover",
                        draw_icon: {
                            svg_file: (ICON_DISCOVER),
                        }
                    }
                    tab2 = <SidebarMenuButton> {
                        label: "Chat",
                        draw_icon: {
                            svg_file: (ICON_CHAT),
                        }
                    }
                    tab3 = <SidebarMenuButton> {
                        label: "My Models",
                        draw_icon: {
                            svg_file: (ICON_MY_MODELS),
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
                    tab2_frame = <ChatScreen> {visible: false}
                    tab3_frame = <MyModelsView> {visible: false}
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
        crate::landing::sorting::live_design(cx);

        crate::chat::chat_screen::live_design(cx);
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
            sidebar_menu.tab3,
        ))
        .selected_to_visible(
            cx,
            &self.ui,
            &actions,
            ids!(
                application_pages.tab1_frame,
                application_pages.tab2_frame,
                application_pages.tab3_frame,
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
                StoreAction::Sort(criteria) => {
                    self.store.sort_models(criteria);
                }
                _ => {}
            }
        }
    }
}