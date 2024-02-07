use makepad_widgets::*;
use crate::data::store::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::icon::Icon;

    const MODEL_LINK_FONT_COLOR = #x155EEF
    ICON_INFO = dep("crate://self/resources/icons/info.svg")
    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    ICON_DOWNLOAD_DONE = dep("crate://self/resources/icons/download_done.svg")

    ModelLink = <LinkLabel> {
        width: Fill,
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        MODEL_LINK_FONT_COLOR,
                        MODEL_LINK_FONT_COLOR,
                        self.hover
                    ),
                    MODEL_LINK_FONT_COLOR,
                    self.pressed
                )
            }
        }
    }

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            instance radius: 2.0,
        }

        attr_name = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #fff
            }
        }

        attr_value = <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 9},
                color: #fff
            }
        }

    }

    ModelAttributes = <View> {
        width: Fit,
        height: Fit,
        spacing: 10,

        <ModelAttributeTag> {
            draw_bg: { color: #3538CD },
            attr_name = { text: "Model Size" }
            attr_value = { text: "7B params" }
        }

        <ModelAttributeTag> {
            draw_bg: { color: #CA8504 },
            attr_name = { text: "Requires" }
            attr_value = { text: "8GB+ RAM" }
        }

        <ModelAttributeTag> {
            draw_bg: { color: #FCCEEE },
            attr_name = {
                draw_text: { color: #C11574 },
                text: "Architecture"
            }
            attr_value = {
                draw_text: { color: #C11574 },
                text: "Mistral"
            }
        }
    }

    ModelHeading = <View> {
        height: Fit,
        <View> {
            width: Fit,
            height: Fit,
            flow: Down,
            spacing: 10,
            name = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16},
                    color: #000
                }
                text: "OpenHermes 2.5 Mistral 7B"
            }
            <ModelAttributes> {}
        }
        <VerticalFiller> {}
        <View> {
            width: Fit,
            height: Fit,
            <ModelAttributeTag> {
                width: Fit,
                height: Fit,
                padding: 4,
    
                draw_bg: {
                    color: #0000,
                    border_color: #98A2B3,
                    border_width: 1.0,
                },
                attr_name = {
                    draw_text: { color: #000 }
                    text: "Released"
                }
                attr_value = {
                    draw_text: { color: #000 }
                    text: "Oct 29, 2023 (90 days ago)"
                }
            }
        }
    }

    ModelSummary = <View> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 20,
        padding: { right: 100 }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 11},
                color: #000
            }
            text: "Model Summary"
        }
        summary = <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                word: Wrap,
                color: #000
            }
            text: "OpenHermes 2.5 Mistral 7B is an advanced iteration of the OpenHermes 2 language model, enhanced by training on a significant proportion of code datasets. This additional training improved performance across several benchmarks, notably TruthfulQA, AGIEval, and the GPT4All suite, while slightly decreasing the BigBench score. Notably, the model's ability to handle code-related tasks, measured by the humaneval score..."
        }

        <ModelLink> {
            text: "View All"
        }
    }

    ModelDetails = <View> {
        width: 400,
        height: Fit,
        flow: Down,
        spacing: 20,

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 11},
                color: #000
            }
            text: "Author"
        }

        <ModelLink> {
            text: "Teknium"
        }

        <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                word: Wrap,
                color: #000
            }
            text: "Creator of numerous chart topping fine-tunes and a Co-founder of NousResearch"
        }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 11},
                color: #000
            }
            text: "Model Resources"
        }

        <ModelLink> {
            text: "Hugging Face"
        }
    }

    ModelInformation = <View> {
        width: Fill,
        height: Fit,
        spacing: 10,
        <ModelSummary> {}
        <ModelDetails> {}
    }

    ModelHighlightedFilesRow = <View> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #F9FAFB
        }

        cell1 = <View> { width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell2 = <View> { width: 180, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell3 = <View> { width: 180, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell4 = <View> { width: 220, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
    }

    ModelHighlightedFilesLabel = <RoundedView> {
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

    ModelHighlightedFilesRowWithData = <ModelHighlightedFilesRow> {
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

    ModelHighlightedFilesList = {{ModelHighlightedFilesList}} {
        width: Fill,
        height: Fit,
        flow: Down,

        template_downloaded: <ModelHighlightedFilesRowWithData> {
            cell1 = {
                filename = { text: "stablelm-zephyr-3b.Q6_K.gguf" }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Less Compressed" }
                }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Might be slower" }
                }
            }
            cell2 = { full_size = { text: "2.30 GB" }}
            cell3 = {
                quantization_tag = { quantization = { text: "Q6_K" }}
            }
            cell4 = {
                <DownloadedButton> {}
            }
        }

        template_download: <ModelHighlightedFilesRowWithData> {
            cell1 = {
                filename = { text: "stablelm-zephyr-3b.Q6_K.gguf" }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Less Compressed" }
                }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Might be slower" }
                }
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

    ModelHighlightedFiles = <View> {
        width: Fill,
        height: Fit,
        flow: Down,

        <ModelHighlightedFilesRow> {
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
                <ModelLink> {
                    width: Fit,
                    text: "See All Files"
                }
            }
        }

        <ModelHighlightedFilesRow> {
            show_bg: false,
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

        <ModelHighlightedFilesList> {}
    }

    ModelCard = <RoundedView> {
        width: Fill,
        height: Fit,

        draw_bg: {
            instance radius: 3.0,
            color: #F2F4F7
        }

        flow: Down,
        padding: 30,
        spacing: 20,

        <ModelHeading> {}
        <Line> {}
        <ModelInformation> {}
        <ModelHighlightedFiles> {}
    }
}


#[derive(Live, Widget)]
pub struct ModelHighlightedFilesList {
    #[redraw] #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template_downloaded: Option<LivePtr>,
    #[live]
    template_download: Option<LivePtr>,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl LiveHook for ModelHighlightedFilesList {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        // TODO Temporary data load
        let store = Store::new();
        let files = &store.models[0].files;

        let mut item_widget;
        for i in 0..files.len() {
            let item_id = LiveId(i as u64).into();
            if files[i].downloaded {
                item_widget = WidgetRef::new_from_ptr(cx, self.template_downloaded);
            } else {
                item_widget = WidgetRef::new_from_ptr(cx, self.template_download);
            }

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

            self.items.insert(item_id, item_widget);
        }
    }
}

impl Widget for ModelHighlightedFilesList {
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