use makepad_widgets::*;
use crate::data::store::Store;
use moxin_protocol::data::{Model, File};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::icon::Icon;
    import crate::landing::shared::*;

    ICON_INFO = dep("crate://self/resources/icons/info.svg")
    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    ICON_DOWNLOAD_DONE = dep("crate://self/resources/icons/download_done.svg")

    ModelFilesRow = <View> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #fff
        }

        cell1 = <View> { width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell2 = <View> { width: 180, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell3 = <View> { width: 180, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell4 = <View> { width: 220, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
    }

    ModelFilesListLabel = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        draw_bg: {
            instance radius: 2.0,
            color: #E6F4D7,
        }

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #3F621A
            }
        }
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

    ModelFilesRowWithData = <ModelFilesRow> {
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
            quantization_tag = <RoundedView> {
                width: Fit,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 10, right: 10}

                draw_bg: {
                    instance radius: 2.0,
                    color: #B9E6FE,
                }
        
                quantization = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 9},
                        color: #1849A9
                    }
                }

                <Icon> {
                    draw_icon: {
                        svg_file: (ICON_INFO), color: #00f,
                    }
                    icon_walk: {width: Fit, height: Fit}
                }
            }
        }

        cell4 = {
            align: {x: 0.5, y: 0.5},
        }
    }

    ModelFilesTags = {{ModelFilesTags}} {
        width: Fit,
        height: Fit,
        flow: Right,
        spacing: 5,

        template: <ModelFilesListLabel> {}
    }

    ModelFilesItems = {{ModelFilesItems}} {
        width: Fill,
        height: Fit,
        flow: Down,

        template_downloaded: <ModelFilesRowWithData> {
            cell1 = {
                filename = { text: "stablelm-zephyr-3b.Q6_K.gguf" }
                tags = <ModelFilesTags> {}
            }
            cell2 = { full_size = { text: "2.30 GB" }}
            cell3 = {
                quantization_tag = { quantization = { text: "Q6_K" }}
            }
            cell4 = {
                <DownloadedButton> {}
            }
        }

        template_download: <ModelFilesRowWithData> {
            cell1 = {
                filename = { text: "stablelm-zephyr-3b.Q6_K.gguf" }
                tags = <ModelFilesTags> {}
            }
            cell2 = { full_size = { text: "2.30 GB" }}
            cell3 = {
                quantization_tag = { quantization = { text: "Q6_K" }}
            }
            cell4 = {
                <DownloadButton> {}
            }
        }
    }

    ModelFilesList = <View> {
        width: Fill,
        height: Fit,
        flow: Down,

        heading_row = <ModelFilesRow> {
            cell1 = {
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #000
                    }
                    text: "Highlighted Files"
                }
            }

            cell4 = {
                align: {x: 0.5, y: 0.5},
                all_files_link = <ModelLink> {
                    width: Fit,
                    text: "See All Files"
                }
            }
        }

        <ModelFilesRow> {
            show_bg: true,
            draw_bg: {
                color: #F2F4F7
            }

            cell1 = {
                height: 40
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #667085
                    }
                    text: "Model File"
                }
            }

            cell2 = {
                height: 40
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #667085
                    }
                    text: "Full Size"
                }
            }

            cell3 = {
                height: 40
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #667085
                    }
                    text: "Quantization"
                }
            }
            cell4 = {
                height: 40
            }
        }

        file_list = <ModelFilesItems> {}
    }
}

#[derive(Live, LiveHook, LiveRegisterWidget, WidgetRef)]
pub struct ModelFilesItems {
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template_downloaded: Option<LivePtr>,
    #[live]
    template_download: Option<LivePtr>,

    #[live(true)]
    show_tags: bool,

    #[live(false)]
    show_featured: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelFilesItems {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_id, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);

        let model = scope.data.get::<Model>();
        let files = if self.show_featured {
            Store::model_featured_files(model)
        } else {
            model.files.clone()
        };

        self.draw_files(cx, walk, &files);

        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}


impl WidgetNode for ModelFilesItems {
    fn walk(&mut self, _cx:&mut Cx) -> Walk{
        self.walk
    }

    fn redraw(&mut self, cx: &mut Cx){
        self.area.redraw(cx)
    }

    fn find_widgets(&mut self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        for item in self.items.values_mut() {
            item.find_widgets(path, cached, results);
        }
    }
}

impl ModelFilesItems {
    fn draw_files(&mut self, cx: &mut Cx2d, walk: Walk, files: &Vec<File>) {
        for i in 0..files.len() {
            let template = if files[i].downloaded {
                self.template_downloaded
            } else {
                self.template_download
            };
            let item_id = LiveId(i as u64).into();
            let item_widget = self.items.get_or_insert(cx, item_id, | cx | {
                WidgetRef::new_from_ptr(cx, template)
            });

            let filename = &files[i].path;
            let size = &files[i].size;
            let quantization = &files[i].quantization;
            item_widget.apply_over(cx, live!{
                cell1 = {
                    filename = { text: (filename) }
                }
                cell2 = { full_size = { text: (size) }}
                cell3 = {
                    quantization_tag = { quantization = { text: (quantization) }}
                 }
            });

            if self.show_tags {
                item_widget.model_files_tags(id!(tags)).set_tags(cx, files[i].tags.clone());
            }

            let _ = item_widget.draw_walk(cx, &mut Scope::empty(), walk);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesTags {
    #[redraw] #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template: Option<LivePtr>,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelFilesTags {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_id, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        for (_id, item) in self.items.iter_mut() {
            let _ = item.draw_walk(cx, scope, walk);
        }
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl ModelFilesTagsRef {
    pub fn set_tags(&self, cx: &mut Cx, tags: Vec<String>) {
        let Some(mut tags_widget) = self.borrow_mut() else { return };
        tags_widget.items.clear();
        for (i, tag) in tags.iter().enumerate() {
            let item_id = LiveId(i as u64).into();
            let item_widget = WidgetRef::new_from_ptr(cx, tags_widget.template);
            item_widget.apply_over(cx, live!{label = { text: (tag) }});
            tags_widget.items.insert(item_id, item_widget);
        }
    }
}
