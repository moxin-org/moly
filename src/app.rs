use crate::chat::model_selector_list::ModelSelectorListAction;
use crate::data::capture::register_capture_manager;
use crate::data::downloads::DownloadPendingNotification;
use crate::data::downloads::download::DownloadFileAction;
use crate::data::moly_client::MolyClientAction;
use crate::data::store::*;
use crate::landing::model_files_item::ModelFileItemAction;
use crate::shared::actions::{ChatAction, DownloadAction};
use crate::shared::download_notification_popup::{
    DownloadNotificationPopupAction, DownloadNotificationPopupRef,
    DownloadNotificationPopupWidgetRefExt, DownloadResult,
};
use crate::shared::moly_server_popup::MolyServerPopupAction;
use crate::shared::popup_notification::PopupNotificationWidgetRefExt;
use moly_protocol::data::{File, FileID};

use makepad_widgets::*;
use markdown::MarkdownAction;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::popup_notification::*;
    use crate::shared::widgets::SidebarMenuButton;
    use crate::shared::download_notification_popup::DownloadNotificationPopup;
    use crate::shared::moly_server_popup::MolyServerPopup;
    use crate::shared::desktop_buttons::MolyDesktopButton;

    use crate::landing::model_card::ModelCardViewAllModal;
    use crate::chat::chat_screen::ChatScreen;
    use crate::settings::moly_server_screen::MolyServerScreen;
    use crate::settings::providers_screen::ProvidersScreen;

    ICON_CHAT = dep("crate://self/resources/icons/chat.svg")
    ICON_LOCAL = dep("crate://self/resources/icons/local.svg")
    ICON_CLOUD = dep("crate://self/resources/icons/cloud.svg")
    ICON_MOLYSERVER = dep("crate://self/resources/images/providers/molyserver.png")

    ApplicationPages = <RoundedShadowView> {
        width: Fill, height: Fill
        margin: {top: 12, right: 12, bottom: 12}
        padding: 3
        flow: Overlay

        show_bg: true
        draw_bg: {
            color: (MAIN_BG_COLOR),
            border_radius: 4.5,
            uniform shadow_color: #0003
            shadow_radius: 15.0,
            shadow_offset: vec2(0.0,-1.5)
        }

        chat_frame = <ChatScreen> {visible: true}
        moly_server_frame = <MolyServerScreen> {visible: false}
        providers_frame = <ProvidersScreen> {visible: false}
    }

    SidebarMenu = <RoundedView> {
        width: 90, height: Fill,
        flow: Down, spacing: 15.0,
        padding: { top: 40, bottom: 20, left: 0, right: 0 },

        align: {x: 0.5, y: 0.5},

        show_bg: true,
        draw_bg: {
            color: (SIDEBAR_BG_COLOR),
            instance border_radius: 0.0,
        }

        logo = <View> {
            width: Fit, height: Fit
            margin: {bottom: 5}
            <Image> {
                width: 50, height: 50,
                source: (ICON_MOLYSERVER),
            }
        }

        seprator = <View> {
            width: Fill, height: 1.6,
            margin: {left: 15, right: 15, bottom: 10}
            show_bg: true
            draw_bg: {
                color: #dadada,
            }
        }

        chat_tab = <SidebarMenuButton> {
            animator: {active = {default: on}}
            text: "Chat",
            draw_icon: {
                svg_file: (ICON_CHAT),
            }
        }
        moly_server_tab = <SidebarMenuButton> {
            text: "MolyServer",
            draw_icon: {
                svg_file: (ICON_LOCAL),
            }
        }
        <HorizontalFiller> {}
        providers_tab = <SidebarMenuButton> {
            text: "Providers",
            draw_icon: {
                svg_file: (ICON_CLOUD),
            }
        }
    }

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1440, 1024), title: "Moly"},
            pass: {clear_color: #fff}

            caption_bar = {
                caption_label = <View> {} // empty view to remove the default caption label
                windows_buttons = <View> {
                    visible: false,
                    width: Fit, height: Fit,
                    min = <MolyDesktopButton> {draw_bg: {button_type: WindowsMin}}
                    max = <MolyDesktopButton> {draw_bg: {button_type: WindowsMax}}
                    close = <MolyDesktopButton> {draw_bg: {button_type: WindowsClose}}
                }
            }

            body = {
                flow: Overlay
                width: Fill,
                height: Fill,
                padding: 0

                // View that shows by default, eventually gets replaced by the root view.
                loading_view = <View> {
                    align: {x: 0.5, y: 0.5}
                    flow: Down, spacing: 20
                    <Image> {
                        width: 100, height: 100,
                        source: (ICON_MOLYSERVER),
                    }
                    <Label> {
                        text: "Loading..."
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> { font_size: 12 }
                            color: #444
                        }
                    }
                }

                root = {{MolyRoot}} {
                    width: Fill,
                    height: Fill,
                    show_bg: true,
                    draw_bg: {
                        color: (MAIN_BG_COLOR_DARK),
                    }

                    root_adaptive_view = <AdaptiveView> {
                        Mobile = {
                            application_pages = <ApplicationPages> {
                                margin: 0
                            }
                        }

                        Desktop = {
                            sidebar_menu = <SidebarMenu> {}
                            application_pages = <ApplicationPages> {}
                        }
                    }
                }

                download_popup = <PopupNotification> {
                    content: {
                        popup_download_notification = <DownloadNotificationPopup> {}
                    }
                }

                moly_server_popup = <PopupNotification> {
                    content: {
                        popup_moly_server = <MolyServerPopup> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    pub ui: WidgetRef,

    #[rust]
    pub store: Option<Store>,

    #[rust]
    timer: Timer,

    #[rust]
    download_retry_attempts: usize,

    #[rust]
    file_id: Option<FileID>,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        moly_kit::live_design(cx);

        crate::shared::live_design(cx);
        crate::landing::live_design(cx);
        crate::chat::live_design(cx);
        crate::my_models::live_design(cx);
        crate::settings::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui_runner()
            .handle(cx, event, &mut Scope::empty(), self);

        if let Event::Startup = event {
            // Prevent rendering the ui before the store is initialized.
            self.ui.view(id!(body)).set_visible(cx, false);
            register_capture_manager();

            #[cfg(any(target_os = "android", target_os = "ios"))]
            // Initialize filesystem with the data directory if available, required for mobile platforms.
            if let Some(data_dir) = cx.get_data_dir() {
                // Ensure the data directory exists
                let path = std::path::PathBuf::from(data_dir.clone());
                let _ = std::fs::create_dir_all(path.clone());
                if path.exists() {
                    crate::shared::utils::filesystem::init_cx_data_dir(path);
                } else {
                    panic!("Failed to create data directory: {}", data_dir);
                }
            }

            Store::load_into_app();
        }

        // If the store is not loaded, do not continue with store-dependent logic
        // however, we still want the window to handle Makepad events. (e.g. window initialization events, platform context changes, etc.)
        let Some(store) = self.store.as_mut() else {
            self.ui.handle_event(cx, event, &mut Scope::empty());
            return;
        };

        self.ui.view(id!(loading_view)).set_visible(cx, false);

        // It triggers when the timer expires.
        if self.timer.is_event(event).is_some() {
            if let Some(file_id) = &self.file_id {
                let (model, file) = store.get_model_and_file_download(&file_id);
                store.downloads.download_file(model, file);
                self.ui.redraw(cx);
            }
        }

        let scope = &mut Scope::with_data(store);
        self.ui.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut navigate_to_chat = false;
        let mut navigate_to_moly_server = false;
        let mut navigate_to_providers = false;

        // TODO: Replace this with a proper navigation widget.
        if let Some(selected_tab) = self
            .ui
            .radio_button_set(ids!(
                sidebar_menu.chat_tab,
                sidebar_menu.moly_server_tab,
                sidebar_menu.providers_tab,
            ))
            .selected(cx, actions)
        {
            match selected_tab {
                0 => navigate_to_chat = true,
                1 => navigate_to_moly_server = true,
                2 => navigate_to_providers = true,
                _ => {}
            }
        }

        for action in actions.iter() {
            if let MarkdownAction::LinkNavigated(url) = action.as_widget_action().cast() {
                let _ = robius_open::Uri::new(&url).open();
            }

            self.store.as_mut().unwrap().handle_action(action);

            if let Some(_) = action.downcast_ref::<DownloadFileAction>() {
                self.notify_downloaded_files(cx);
            }

            let store = self.store.as_mut().unwrap();

            match action.cast() {
                StoreAction::Search(keywords) => {
                    store.search.load_search_results(keywords);
                }
                StoreAction::ResetSearch => {
                    store.search.load_featured_models();
                }
                StoreAction::Sort(criteria) => {
                    store.search.sort_models(criteria);
                }
                _ => {}
            }

            match action.cast() {
                ModelFileItemAction::Download(file_id) => {
                    let (model, file) = store.get_model_and_file_download(&file_id);
                    store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            match action.cast() {
                DownloadAction::Play(file_id) => {
                    let (model, file) = store.get_model_and_file_download(&file_id);
                    store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                DownloadAction::Pause(file_id) => {
                    store.downloads.pause_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                DownloadAction::Cancel(file_id) => {
                    store.downloads.cancel_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            if let ChatAction::Start(_) = action.cast() {
                let chat_radio_button = self.ui.radio_button(id!(chat_tab));
                chat_radio_button.select(cx, &mut Scope::empty());
            }

            if let NavigationAction::NavigateToMyModels = action.cast() {
                let my_models_radio_button = self.ui.radio_button(id!(my_models_tab));
                my_models_radio_button.select(cx, &mut Scope::empty());
                // navigate_to_my_models = true;
            }

            if let NavigationAction::NavigateToProviders = action.cast() {
                let providers_radio_button = self.ui.radio_button(id!(providers_tab));
                providers_radio_button.select(cx, &mut Scope::empty());
                navigate_to_providers = true;
            }

            store.handle_provider_connection_action(action.cast());
            // redraw the UI to reflect the connection status
            self.ui.redraw(cx);

            if matches!(
                action.cast(),
                DownloadNotificationPopupAction::ActionLinkClicked
                    | DownloadNotificationPopupAction::CloseButtonClicked
            ) {
                self.ui.popup_notification(id!(download_popup)).close(cx);
            }

            if let MolyClientAction::ServerUnreachable = action.cast() {
                self.ui.popup_notification(id!(moly_server_popup)).open(cx);
            }

            if let MolyServerPopupAction::CloseButtonClicked = action.cast() {
                self.ui.popup_notification(id!(moly_server_popup)).close(cx);
            }
        }

        // Handle navigation after processing all actions
        if navigate_to_providers {
            self.navigate_to(cx, id!(application_pages.providers_frame));
        } else if navigate_to_chat {
            self.navigate_to(cx, id!(application_pages.chat_frame));
        } else if navigate_to_moly_server {
            self.navigate_to(cx, id!(application_pages.moly_server_frame));
        }
    }
}

impl App {
    fn notify_downloaded_files(&mut self, cx: &mut Cx) {
        let store = self.store.as_mut().unwrap();
        if let Some(notification) = store.downloads.next_download_notification() {
            let mut popup = self
                .ui
                .download_notification_popup(id!(popup_download_notification));

            match notification {
                DownloadPendingNotification::DownloadedFile(file) => {
                    popup.set_data(cx, &file, DownloadResult::Success);
                    cx.action(ModelSelectorListAction::AddedOrDeletedModel);
                }
                DownloadPendingNotification::DownloadErrored(file) => {
                    self.file_id = Some((file.id).clone());
                    self.start_retry_timeout(cx, popup, file);
                }
            }

            self.ui.popup_notification(id!(download_popup)).open(cx);
        }
    }

    fn start_retry_timeout(
        &mut self,
        cx: &mut Cx,
        mut popup: DownloadNotificationPopupRef,
        file: File,
    ) {
        match self.download_retry_attempts {
            0 => {
                self.timer = cx.start_timeout(15.0);
                self.download_retry_attempts += 1;
                popup.set_retry_data(cx);
            }
            1 => {
                self.timer = cx.start_timeout(30.0);
                self.download_retry_attempts += 1;
                popup.set_retry_data(cx);
            }
            2 => {
                self.timer = cx.start_timeout(60.0);
                self.download_retry_attempts += 1;
                popup.set_retry_data(cx);
            }
            _ => {
                popup.set_data(cx, &file, DownloadResult::Failure);
                self.download_retry_attempts = 0;
            }
        }
    }

    fn navigate_to(&mut self, cx: &mut Cx, id: &[LiveId]) {
        let providers_id = id!(application_pages.providers_frame);
        let chat_id = id!(application_pages.chat_frame);
        let moly_server_id = id!(application_pages.moly_server_frame);

        if id != providers_id {
            self.ui.widget(providers_id).set_visible(cx, false);
        }

        if id != chat_id {
            self.ui.widget(chat_id).set_visible(cx, false);
        }

        if id != moly_server_id {
            self.ui.widget(moly_server_id).set_visible(cx, false);
        }

        self.ui.widget(id).set_visible(cx, true);
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum NavigationAction {
    // TODO: Implement a proper navigation system that supports NavigateTo(some_id), NavigateBack, etc.
    NavigateToProviders,
    NavigateToMyModels,
    None,
}

// Ugly workaround to be abale to switch between sync and async code in the `Store`.
pub fn app_runner() -> UiRunner<App> {
    // `0` is reserved for whatever implements `AppMain`.
    UiRunner::new(0)
}

/// A wrapper around the main Moly view, used to prevent draw/events
/// from being propagated to the all of Moly if the store is not loaded.
#[derive(Live, Widget, LiveHook)]
pub struct MolyRoot {
    #[deref]
    view: View,
}

impl Widget for MolyRoot {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if scope.data.get::<Store>().is_none() {
            return DrawStep::done();
        }
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if scope.data.get::<Store>().is_none() {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}
