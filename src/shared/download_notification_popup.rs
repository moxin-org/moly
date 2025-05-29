use makepad_widgets::*;
use moly_protocol::data::{File, FileID};

use crate::{app::NavigationAction, shared::actions::DownloadAction};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::resource_imports::*;
    use crate::shared::widgets::MolyButton;
    use crate::landing::shared::*;

    SUCCESS_ICON = dep("crate://self/resources/images/success_icon.png")
    FAILURE_ICON = dep("crate://self/resources/images/failure_icon.png")

    PRIMARY_LINK_FONT_COLOR = #x0E7090
    SECONDARY_LINK_FONT_COLOR = #667085

    PopupActionLink = <LinkLabel> {
        width: Fit,
        margin: 2,
        draw_text: {
            text_style: <BOLD_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        PRIMARY_LINK_FONT_COLOR,
                        PRIMARY_LINK_FONT_COLOR,
                        self.hover
                    ),
                    PRIMARY_LINK_FONT_COLOR,
                    self.down
                )
            }
        }
    }

    PopupSecondaryActionLink = <LinkLabel> {
        width: Fit,
        margin: 2,
        draw_text: {
            text_style: <BOLD_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        SECONDARY_LINK_FONT_COLOR,
                        SECONDARY_LINK_FONT_COLOR,
                        self.hover
                    ),
                    SECONDARY_LINK_FONT_COLOR,
                    self.down
                )
            }
        }
    }

    PopupDialog = <RoundedView> {
        width: 350
        height: Fit
        margin: {top: 20, right: 20}
        padding: {top: 20, right: 20 bottom: 20 left: 20}
        spacing: 15

        show_bg: true
        draw_bg: {
            color: #fff
            instance border_radius: 4.0
            fn pixel(self) -> vec4 {
                let border_color = #d4;
                let border_size = 1;
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let body = #fff

                sdf.box(
                    1.,
                    1.,
                    self.rect_size.x - 2.0,
                    self.rect_size.y - 2.0,
                    self.border_radius
                )
                sdf.fill_keep(body)

                sdf.stroke(
                    border_color,
                    border_size
                )
                return sdf.result
            }
        }
    }

    PopupCloseButton = <MolyButton> {
        width: Fit,
        height: Fit,

        margin: {top: -8}

        draw_icon: {
            svg_file: (ICON_CLOSE),
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
        icon_walk: {width: 10, height: 10}
    }

    NotificationIcons = <View> {
        width: Fit,
        height: Fit,
        margin: {top: -10, left: -10}
        success_icon = <View> {
            width: Fit,
            height: Fit,
            <Image> {
                source: (SUCCESS_ICON),
                width: 35,
                height: 35,
            }
        }
        failure_icon = <View> {
            visible: false,
            width: Fit,
            height: Fit,
            <Image> {
                source: (FAILURE_ICON),
                width: 35,
                height: 35,
            }
        }
    }

    NotificationContent = <View> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 10

        title = <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 9},
                word: Wrap,
                color: #000
            }
            text: "Model Downloaded Successfully"
        }

        summary = <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                word: Wrap,
                color: #000
            }
            text: ""
        }

        success_actions = <View> {
            width: Fit,
            height: Fit,
            view_in_my_models_link = <PopupActionLink> {
                text: "View in My Models"
            }
        }

        failure_actions = <View> {
            width: Fit,
            height: Fit,
            spacing: 10,

            retry_link = <PopupActionLink> {
                text: "Retry"
            }

            cancel_link = <PopupSecondaryActionLink> {
                text: "Cancel"
            }
        }
    }

    pub DownloadNotificationPopup = {{DownloadNotificationPopup}} {
        width: Fit
        height: Fit

        <PopupDialog> {
            <NotificationIcons> {}
            <NotificationContent> {}
            close_button = <PopupCloseButton> {}
        }
    }

}

#[derive(Clone, Debug, DefaultNone)]
pub enum DownloadNotificationPopupAction {
    None,
    // User has dimissed the popup by clicking the close button, so the popup should be closed by the owner widget.
    CloseButtonClicked,
    // User has clicked any of the links in the popup, so the popup should be closed by the owner widget.
    ActionLinkClicked,
}

#[derive(Default)]
pub enum DownloadResult {
    #[default]
    Success,
    Failure,
}

#[derive(Live, LiveHook, Widget)]
pub struct DownloadNotificationPopup {
    #[deref]
    view: View,
    #[layout]
    layout: Layout,

    #[rust]
    download_result: DownloadResult,
    #[rust]
    file_id: Option<FileID>,
    #[rust]
    filename: String,
    #[rust]
    count: usize,
}

impl Widget for DownloadNotificationPopup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let _ = self
            .view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }));

        DrawStep::done()
    }
}

impl WidgetMatchEvent for DownloadNotificationPopup {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.button(id!(close_button)).clicked(actions) {
            cx.action(DownloadNotificationPopupAction::CloseButtonClicked);
        }

        if self
            .link_label(id!(view_in_my_models_link))
            .clicked(actions)
        {
            cx.action(NavigationAction::NavigateToMyModels);
            cx.action(DownloadNotificationPopupAction::ActionLinkClicked);
        }

        if self.link_label(id!(retry_link)).clicked(actions) {
            let Some(file_id) = &self.file_id else { return };
            cx.action(DownloadAction::Play(file_id.clone()));
            cx.action(DownloadNotificationPopupAction::ActionLinkClicked);
        }

        if self.link_label(id!(cancel_link)).clicked(actions) {
            let Some(file_id) = &self.file_id else { return };
            cx.action(DownloadAction::Cancel(file_id.clone()));
            cx.action(DownloadNotificationPopupAction::ActionLinkClicked);
        }
    }
}

impl DownloadNotificationPopup {
    pub fn update_content(&mut self, cx: &mut Cx) {
        match self.download_result {
            DownloadResult::Success => self.show_success_content(cx),
            DownloadResult::Failure => self.show_failure_content(cx),
        }
    }

    fn show_success_content(&mut self, cx: &mut Cx) {
        self.view(id!(success_icon)).set_visible(cx, true);
        self.view(id!(failure_icon)).set_visible(cx, false);

        self.view(id!(success_actions)).set_visible(cx, true);
        self.view(id!(failure_actions)).set_visible(cx, false);

        self.label(id!(title))
            .set_text(cx, "Model Downloaded Successfully");

        self.label(id!(summary))
            .set_text(cx, &(format!("{} successfuly downloaded.", &self.filename)));
    }

    fn show_failure_content(&mut self, cx: &mut Cx) {
        self.view(id!(success_icon)).set_visible(cx, false);
        self.view(id!(failure_icon)).set_visible(cx, true);

        self.view(id!(success_actions)).set_visible(cx, false);
        self.view(id!(failure_actions)).set_visible(cx, true);

        self.label(id!(title))
            .set_text(cx, "Errors while downloading models");

        self.label(id!(summary)).set_text(
            cx,
            &(format!(
                "{} encountered some errors when downloading.",
                &self.filename
            )),
        );
    }

    pub fn show_retry_content(&mut self, cx: &mut Cx) {
        let content = self.label(id!(summary));
        self.view(id!(success_icon)).set_visible(cx, false);
        self.view(id!(failure_icon)).set_visible(cx, true);

        self.view(id!(success_actions)).set_visible(cx, false);
        self.view(id!(failure_actions)).set_visible(cx, false);

        self.label(id!(title)).set_text(cx, "Retry");

        match self.count {
            0 => {
                content.set_text(cx, "Download interrupted. Will resume in 15 seconds.");
                self.count += 1;
            }
            1 => {
                content.set_text(cx, "Download interrupted. Will resume in 30 seconds.");
                self.count += 1;
            }
            2 => {
                content.set_text(cx, "Download interrupted. Will resume in 60 seconds.");
                self.count += 1;
            }
            _ => {
                self.count = 0;
            }
        }
    }
}

impl DownloadNotificationPopupRef {
    pub fn set_data(&mut self, cx: &mut Cx, file: &File, download_result: DownloadResult) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.file_id = Some(file.id.clone());
            inner.filename = file.name.clone();
            inner.download_result = download_result;

            inner.update_content(cx);
        }
    }

    pub fn set_retry_data(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_retry_content(cx);
        }
    }
}
