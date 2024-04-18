use crate::data::store::*;
use crate::landing::model_card::{ModelCardViewAllModalWidgetRefExt, ViewAllModalAction};
use crate::landing::model_files_list::ModelFileItemsAction;
use crate::my_models::downloaded_files_table::DownloadedFileAction;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::modal::*;
    import crate::landing::landing_screen::LandingScreen;
    import crate::landing::model_card::ModelCardViewAllModal;
    import crate::chat::chat_screen::ChatScreen;
    import crate::my_models::my_models_screen::MyModelsScreen;


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

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1440, 1024)},
            pass: {clear_color: #fff}

            body = {
                flow: Overlay
                width: Fill,
                height: Fill,

                root = <View> {
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

                        discover_tab = <SidebarMenuButton> {
                            animator: {selected = {default: on}}
                            label: "Discover",
                            draw_icon: {
                            svg_file: (ICON_DISCOVER),
                            }
                        }
                        chat_tab = <SidebarMenuButton> {
                            label: "Chat",
                            draw_icon: {
                            svg_file: (ICON_CHAT),
                            }
                        }
                        my_models_tab = <SidebarMenuButton> {
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

                        discover_frame = <LandingScreen> {visible: true}
                        chat_frame = <ChatScreen> {visible: false}
                        my_models_frame = <MyModelsScreen> {visible: false}
                    }
                }

                modal_root = <Modal> {
                    model_card_view_all_modal_view = <ModalView> {
                        content = {
                            model_card_view_all_modal = <ModelCardViewAllModal> {}
                        }
                    }
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

        // Shared
        crate::shared::styles::live_design(cx);
        crate::shared::widgets::live_design(cx);
        crate::shared::icon::live_design(cx);
        crate::shared::modal::live_design(cx);
        crate::shared::external_link::live_design(cx);

        // Landing
        crate::landing::shared::live_design(cx);
        crate::landing::model_files_list::live_design(cx);
        crate::landing::model_card::live_design(cx);
        crate::landing::model_list::live_design(cx);
        crate::landing::landing_screen::live_design(cx);
        crate::landing::search_bar::live_design(cx);
        crate::landing::sorting::live_design(cx);

        // Chat
        crate::chat::chat_screen::live_design(cx);
        crate::chat::model_selector::live_design(cx);
        crate::chat::chat_panel::live_design(cx);
        crate::chat::chat_line::live_design(cx);

        // My Models
        crate::my_models::my_models_screen::live_design(cx);
        crate::my_models::downloaded_files_table::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Process all possible store incoming events
        if let Event::Signal = event {
            self.store.process_event_signal();
            self.ui.redraw(cx);
        }

        let scope = &mut Scope::with_data(&mut self.store);
        self.ui.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.ui
            .radio_button_set(ids!(
                sidebar_menu.discover_tab,
                sidebar_menu.chat_tab,
                sidebar_menu.my_models_tab,
            ))
            .selected_to_visible(
                cx,
                &self.ui,
                &actions,
                ids!(
                    application_pages.discover_frame,
                    application_pages.chat_frame,
                    application_pages.my_models_frame,
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

            if let ModelFileItemsAction::Download(file) = action.as_widget_action().cast() {
                self.store.download_file(&file);
            }

            // Set modal viewall model id
            if let ViewAllModalAction::ModelSelected(model_id) = action.as_widget_action().cast() {
                let mut modal = self
                    .ui
                    .model_card_view_all_modal(id!(model_card_view_all_modal));
                modal.set_model_id(model_id);
            }

            if let DownloadedFileAction::StartChat(_) = action.as_widget_action().cast() {
                let chat_radio_button = self.ui.radio_button(id!(chat_tab));
                chat_radio_button.select(cx, &mut Scope::empty());
            }
        }
    }
}
