use makepad_widgets::*;
use moxin_protocol::data::{File, Model};

use super::model_files_tags::ModelFilesTagsWidgetExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    import crate::landing::model_files_tags::ModelFilesTags;

    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    ICON_DOWNLOAD_DONE = dep("crate://self/resources/icons/download_done.svg")

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

    ModelCardButton = <RoundedView> {
        width: 140,
        height: 32,
        align: {x: 0.5, y: 0.5}
        spacing: 6,

        draw_bg: { color: #099250 }

        button_icon = <Icon> {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
            icon_walk: {width: Fit, height: Fit}
        }

        button_label = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
        }
    }

    DownloadButton = <ModelCardButton> {
        cursor: Hand,
        button_label = { text: "Download" }
        button_icon = { draw_icon: {
            svg_file: (ICON_DOWNLOAD),
        }}
    }

    DownloadedButton = <ModelCardButton> {
        draw_bg: { color: #fff, border_color: #099250, border_width: 0.5}
        button_label = {
            text: "Downloaded"
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #099250;
                }
            }
        }
        button_icon = {
            draw_icon: {
                svg_file: (ICON_DOWNLOAD_DONE),
                fn get_color(self) -> vec4 {
                    return #099250;
                }
            }
        }
    }

    // TODO This is a very temporary solution, we will have a better way to handle this.
    DownloadPendingButton = <ModelCardButton> {
        draw_bg: { color: #fff, border_color: #x155EEF, border_width: 0.5}
        button_label = {
            text: "Downloading..."
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #x155EEF;
                }
            }
        }
        button_icon = {
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
            downloaded_button = <DownloadedButton> { visible: false }
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
        if let Some(fd) = self.view(id!(download_button)).finger_down(&actions) {
            if fd.tap_count == 1 {
                let widget_uid = self.widget_uid();
                let Some(model) = &self.model else { return };
                let Some(file) = &self.file else { return };

                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    ModelFileItemAction::Download(file.clone(), model.clone()),
                );
            }
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
