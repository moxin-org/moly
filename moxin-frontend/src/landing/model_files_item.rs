use makepad_widgets::*;
use moxin_protocol::data::{File, FileID};

use super::model_files_tags::ModelFilesTagsWidgetExt;
use crate::shared::{
    actions::{ChatAction, DownloadAction},
    widgets::c_button::CButtonWidgetExt,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::MoxinButton;
    import crate::landing::model_files_tags::ModelFilesTags;

    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    START_CHAT = dep("crate://self/resources/icons/start_chat.svg")
    RESUME_CHAT = dep("crate://self/resources/icons/play_arrow.svg")

    ICON_PAUSE = dep("crate://self/resources/icons/pause_download.svg")
    ICON_CANCEL = dep("crate://self/resources/icons/cancel_download.svg")
    ICON_PLAY = dep("crate://self/resources/icons/play_download.svg")
    ICON_RETRY = dep("crate://self/resources/icons/retry_download.svg")

    ModelFilesRow = <RoundedYView> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #00f
            radius: vec2(1.0, 1.0)
        }

        cell1 = <View> { width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell2 = <View> { width: 140, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell3 = <View> { width: 340, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell4 = <View> { width: 250, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
    }

    ModelCardButton = <MoxinButton> {
        width: 140,
        height: 32,
    }

    DownloadButton = <ModelCardButton> {
        draw_bg: { color: #099250, border_color: #099250 }
        text: "Download"
        draw_icon: {
            svg_file: (ICON_DOWNLOAD),
        }
    }

    StartChatButton = <ModelCardButton> {
        draw_bg: { color: #fff, color_hover: #09925033, border_color: #d0d5dd }
        text: "Chat with Model"
        draw_text: {
            color: #087443;
        }
        draw_icon: {
            svg_file: (START_CHAT),
            color: #087443
        }
    }

    ResumeChatButton = <ModelCardButton> {
        draw_bg: { color: #099250, border_color: #09925033 }
        text: "Resume Chat"
        draw_text: {
            color: #fff;
        }
        draw_icon: {
            svg_file: (RESUME_CHAT),
        }
    }

    DownloadPendingButton = <Button> {
        padding: 4,
        draw_icon: {
            fn get_color(self) -> vec4 {
                return #667085;
            }
        }
        icon_walk: {width: 14, height: 14}
    }

    DownloadPendingControls = <View> {
        align: {y: 0.5},
        spacing: 8,
        progress_bar = <View> {
            width: 74,
            height: 12,
            flow: Overlay,

            <RoundedView> {
                height: Fill,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 2.5,
                }
            }

            progress_fill = <RoundedView> {
                width: 0,
                height: Fill,
                draw_bg: {
                    color: #099250,
                    radius: 2.5,
                }
            }
        }
        progress_text = <Label> {
            text: "0%",
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 9},
                color: #087443
            }
        }
        resume_download_button = <DownloadPendingButton> {
            draw_icon: {
                svg_file: (ICON_PLAY),
            }
        }
        pause_download_button = <DownloadPendingButton> {
            draw_icon: {
                svg_file: (ICON_PAUSE),
            }
        }
        cancel_download_button = <DownloadPendingButton> {
            draw_icon: {
                svg_file: (ICON_CANCEL),
            }
        }
    }

    ModelFilesItem = {{ModelFilesItem}}<ModelFilesRow> {
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        cell1 = {
            spacing: 10,
            filename = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 9},
                    color: #000
                }
            }
        }

        cell2 = {
            full_size = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 9},
                    color: #000
                }
            }
        }

        cell3 = {
            spacing: 6,
            quantization_tag = <RoundedView> {
                width: Fit,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 10, right: 10}

                draw_bg: {
                    instance radius: 2.0,
                    border_color: #B4B4B4,
                    border_width: 0.5,
                    color: #FFF,
                }

                quantization = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 9},
                        color: #000
                    }
                }
            }
            tags = <ModelFilesTags> {}
        }

        cell4 = {
            download_button = <DownloadButton> { visible: false }
            start_chat_button = <StartChatButton> { visible: false }
            resume_chat_button = <ResumeChatButton> { visible: false }
            download_pending_controls = <DownloadPendingControls> { visible: false }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelFileItemAction {
    Download(FileID),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesItem {
    #[deref]
    view: View,

    #[rust]
    file_id: Option<FileID>,
}

impl Widget for ModelFilesItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelFilesItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();
        let Some(file_id) = self.file_id.clone() else {
            return;
        };

        if self.button(id!(download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                ModelFileItemAction::Download(file_id.clone()),
            );
        }

        if self.button(id!(start_chat_button)).clicked(&actions) {
            cx.widget_action(widget_uid, &scope.path, ChatAction::Start(file_id.clone()));
        }

        if self.button(id!(resume_chat_button)).clicked(&actions) {
            cx.widget_action(widget_uid, &scope.path, ChatAction::Resume(file_id));
        }

        if self.button(id!(resume_download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Play(file_id.clone()),
            );
        }

        if self.button(id!(pause_download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Pause(file_id.clone()),
            );
        }

        if self.button(id!(cancel_download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Cancel(file_id.clone()),
            );
        }
    }
}

impl ModelFilesItemRef {
    pub fn set_file(&mut self, cx: &mut Cx, file: File) {
        let Some(mut item_widget) = self.borrow_mut() else {
            return;
        };

        item_widget.file_id = Some(file.id.clone());

        item_widget
            .model_files_tags(id!(tags))
            .set_tags(cx, &file.tags);
    }
}
