use makepad_widgets::*;
use moxin_protocol::data::{File, Model};

use super::model_files_tags::ModelFilesTagsWidgetExt;
use crate::shared::{actions::DownloadedFileAction, widgets::c_button::CButtonWidgetExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::c_button::*;
    import crate::landing::model_files_tags::ModelFilesTags;

    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    START_CHAT = dep("crate://self/resources/icons/start_chat.svg")
    RESUME_CHAT = dep("crate://self/resources/icons/play_arrow.svg")

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

    ModelCardButton = <CButton> {
        width: 140,
        height: 32,
        spacing: 6,

        icon = {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
            icon_walk: {width: 14, height: 14}
        }

        label = {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
        }
    }

    DownloadButton = <ModelCardButton> {
        draw_bg: { color: #099250, border_color: #099250 }
        label = { text: "Download" }
        icon = { draw_icon: {
            svg_file: (ICON_DOWNLOAD),
        }}
    }

    StartChatButton = <ModelCardButton> {
        draw_bg: { color: #fff, border_color: #d0d5dd }
        label = {
            text: "Chat with Model"
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #087443;
                }
            }
        }
        icon = {
            draw_icon: {
                svg_file: (START_CHAT),
                fn get_color(self) -> vec4 {
                    return #087443;
                }
            }
        }
    }

    ResumeChatButton = <ModelCardButton> {
        draw_bg: { color: #087443, border_color: #087443 }
        label = {
            text: "Resume Chat"
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
        }
        icon = {
            draw_icon: {
                svg_file: (RESUME_CHAT),
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
        }
    }

    // TODO This is a very temporary solution, we will have a better way to handle this.
    DownloadPendingButton = <ModelCardButton> {
        draw_bg: { color: #fff, border_color: #x155EEF, border_width: 0.5}
        label = {
            text: "Downloading..."
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #x155EEF;
                }
            }
        }
        icon = {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    // invisible for now
                    return #0000;
                }
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
            align: {x: 0.5, y: 0.5},
            download_button = <DownloadButton> { visible: false }
            start_chat_button = <StartChatButton> { visible: false }
            resume_chat_button = <ResumeChatButton> { visible: false }
            download_pending_button = <DownloadPendingButton> { visible: false }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelFileItemAction {
    Download(File, Model),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesItem {
    #[deref]
    view: View,

    #[rust]
    model: Option<Model>,

    #[rust]
    file: Option<File>,
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

        if self.cbutton(id!(download_button)).clicked(&actions) {
            let Some(model) = &self.model else { return };
            let Some(file) = &self.file else { return };

            cx.widget_action(
                widget_uid,
                &scope.path,
                ModelFileItemAction::Download(file.clone(), model.clone()),
            );
        }

        if self.cbutton(id!(start_chat_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadedFileAction::StartChat(self.file.as_ref().unwrap().id.clone()),
            );
        }

        if self.cbutton(id!(resume_chat_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadedFileAction::ResumeChat(self.file.as_ref().unwrap().id.clone()),
            );
        }
    }
}

impl ModelFilesItemRef {
    pub fn set_model_and_file(&mut self, cx: &mut Cx, model: Model, file: File) {
        let Some(mut item_widget) = self.borrow_mut() else {
            return;
        };

        item_widget.model = Some(model);
        item_widget.file = Some(file.clone());

        item_widget
            .model_files_tags(id!(tags))
            .set_tags(cx, &file.tags);
    }
}
