use crate::chat::model_selector_list::ModelSelectorListAction;
use crate::data::downloads::download::DownloadFileAction;
use crate::data::downloads::DownloadPendingNotification;
use crate::data::moly_client::MolyClientAction;
use crate::data::store::*;
use crate::landing::model_files_item::ModelFileItemAction;
use crate::shared::actions::{ChatAction, DownloadAction};
use crate::shared::download_notification_popup::{
    DownloadNotificationPopupAction, DownloadNotificationPopupRef, DownloadNotificationPopupWidgetRefExt, DownloadResult
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

                root = <View> {
                    width: Fill,
                    height: Fill,
                    show_bg: true,
                    draw_bg: {
                        color: (MAIN_BG_COLOR_DARK),
                    }

                    sidebar_menu = <RoundedView> {
                        width: 90, height: Fill,
                        flow: Down, spacing: 15.0,
                        padding: { top: 50, bottom: 20, left: 0, right: 0 },

                        align: {x: 0.5, y: 0.0},

                        show_bg: true,
                        draw_bg: {
                            color: (SIDEBAR_BG_COLOR),
                            instance border_radius: 0.0,
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

                    application_pages = <RoundedShadowView> {
                        show_bg: true,
                        draw_bg: {
                            color: (MAIN_BG_COLOR),
                            border_radius: 4.5,
                            uniform shadow_color: #0003
                            shadow_radius: 15.0,
                            shadow_offset: vec2(0.0,-1.5)
                        }

                        margin: {top: 12, right: 12, bottom: 12}
                        padding: 3

                        flow: Overlay,

                        width: Fill,
                        height: Fill,

                        chat_frame = <ChatScreen> {visible: true}
                        moly_server_frame = <MolyServerScreen> {visible: false}
                        providers_frame = <ProvidersScreen> {visible: false}
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
    ui: WidgetRef,

    #[rust]
    store: Store,

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
        
        // It triggers when the timer expires.
        if self.timer.is_event(event).is_some() {
            if let Some(file_id) = &self.file_id {
                let (model, file) = self.store.get_model_and_file_download(&file_id);
                self.store.downloads.download_file(model, file);
                self.ui.redraw(cx);
            }
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
                sidebar_menu.chat_tab,
                sidebar_menu.moly_server_tab,
                sidebar_menu.providers_tab,
            ))
            .selected_to_visible(
                cx,
                &self.ui,
                actions,
                ids!(
                    application_pages.chat_frame,
                    application_pages.moly_server_frame,
                    application_pages.providers_frame,
                ),
            );

        for action in actions.iter() {
            if let MarkdownAction::LinkNavigated(url) = action.as_widget_action().cast() {
                let _ = robius_open::Uri::new(&url).open();
            }

            self.store.handle_action(action);

            if let Some(_) = action.downcast_ref::<DownloadFileAction>() {
                self.notify_downloaded_files(cx);
            }

            match action.cast() {
                StoreAction::Search(keywords) => {
                    self.store.search.load_search_results(keywords);
                }
                StoreAction::ResetSearch => {
                    self.store.search.load_featured_models();
                }
                StoreAction::Sort(criteria) => {
                    self.store.search.sort_models(criteria);
                }
                _ => {}
            }

            match action.cast() {
                ModelFileItemAction::Download(file_id) => {
                    let (model, file) = self.store.get_model_and_file_download(&file_id);
                    self.store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            match action.cast() {
                DownloadAction::Play(file_id) => {
                    let (model, file) = self.store.get_model_and_file_download(&file_id);
                    self.store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                DownloadAction::Pause(file_id) => {
                    self.store.downloads.pause_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                DownloadAction::Cancel(file_id) => {
                    self.store.downloads.cancel_download_file(&file_id);
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
            }

            if let NavigationAction::NavigateToProviders = action.cast() {
                let providers_radio_button = self.ui.radio_button(id!(providers_tab));
                providers_radio_button.select(cx, &mut Scope::empty());
            }

            self.store.handle_provider_connection_action(action.cast());
            // redraw the UI to reflect the connection status
            self.ui.redraw(cx);

            if matches!(
                action.cast(),
                DownloadNotificationPopupAction::ActionLinkClicked
                    | DownloadNotificationPopupAction::CloseButtonClicked
            ) {
                self.ui
                    .popup_notification(id!(download_popup))
                    .close(cx);
            }

            if let MolyClientAction::ServerUnreachable = action.cast() {
                self.ui.popup_notification(id!(moly_server_popup)).open(cx);
            }

            if let MolyServerPopupAction::CloseButtonClicked = action.cast() {
                self.ui.popup_notification(id!(moly_server_popup)).close(cx);
            }
        }
    }
}

impl App {
    fn notify_downloaded_files(&mut self, cx: &mut Cx) {
        if let Some(notification) = self.store.downloads.next_download_notification() {
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

    fn start_retry_timeout(&mut self, cx: &mut Cx, mut popup: DownloadNotificationPopupRef, file: File) {
        match self.download_retry_attempts {
            0 => {
                self.timer = cx.start_timeout(15.0);
                self.download_retry_attempts += 1;
                popup.set_retry_data(cx);
            },
            1 => {
                self.timer = cx.start_timeout(30.0);
                self.download_retry_attempts += 1;
                popup.set_retry_data(cx);
            },
            2 => {
                self.timer = cx.start_timeout(60.0);
                self.download_retry_attempts += 1;
                popup.set_retry_data(cx);
            },
            _ => {
                popup.set_data(cx, &file, DownloadResult::Failure);
                self.download_retry_attempts = 0;
            }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum NavigationAction {
    // TODO: Implement a proper navigation system that supports NavigateTo(some_id), NavigateBack, etc.
    NavigateToProviders,
    NavigateToMyModels,
    None,
}
