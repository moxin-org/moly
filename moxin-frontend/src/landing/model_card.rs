use makepad_widgets::*;
use moxin_protocol::data::Model;
use crate::data::store::Store;
use unicode_segmentation::UnicodeSegmentation;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::landing::shared::*;
    import crate::landing::model_files_list::ModelFilesList;

    ICON_DOWNLOADS = dep("crate://self/resources/icons/downloads.svg")
    ICON_FAVORITE = dep("crate://self/resources/icons/favorite.svg")
    ICON_EXTERNAL_LINK = dep("crate://self/resources/icons/external_link.svg")

    ModelHeading = <View> {
        flow: Down,
        width: Fill,
        height: Fit,

        spacing: 10,

        <View> {
            width: Fill,
            height: Fit,

            spacing: 10,
            align: {x: 0.0, y: 0.5},

            <View> {
                width: Fit,
                height: Fit,
                model_name = <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 16},
                        color: #000
                    }
                }
            }

            <RoundedView> {
                width: Fit,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 4, right: 10}
                margin: {left: 30}

                spacing: 4,
                align: {x: 0.0, y: 0.5},

                draw_bg: {
                    instance radius: 2.0,
                    color: #FFEDED,
                }

                <Icon> {
                    draw_icon: {
                        svg_file: (ICON_FAVORITE),
                        fn get_color(self) -> vec4 {
                            return #000;
                        }
                    }
                    icon_walk: {width: 14, height: 14}
                }

                model_like_count = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 10},
                        color: #1C1917
                    }
                }
            }

            <RoundedView> {
                width: Fit,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 4, right: 10}

                spacing: 4,
                align: {x: 0.0, y: 0.5},

                draw_bg: {
                    instance radius: 2.0,
                    color: #DCF6E6,
                }

                <Icon> {
                    draw_icon: {
                        svg_file: (ICON_DOWNLOADS),
                        fn get_color(self) -> vec4 {
                            return #000;
                        }
                    }
                    icon_walk: {width: 12, height: 12}
                }

                model_download_count = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 10},
                        color: #1C1917
                    }
                }
            }

            <VerticalFiller> {}

            <View> {
                width: 260,
                height: Fit,
                model_released_at_tag = <ModelAttributeTag> {
                    width: Fit,
                    height: Fit,
    
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
                        margin: {left: 10},
                        draw_text: { color: #000 }
                    }
                }
            }
        }
        <ModelAttributes> {}
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
        model_summary = <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                word: Wrap,
                color: #000
            }
        }

        <ModelLink> {
            link = { text: "View All" }
        }
    }

    ExternalLinkIcon = <Icon> {
        draw_icon: {
            svg_file: (ICON_EXTERNAL_LINK),
            fn get_color(self) -> vec4 {
                return (MODEL_LINK_FONT_COLOR);
            }
        }
        icon_walk: {width: 12, height: 12}
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
            text: "Resouces"
        }

        <View> {
            width: Fit,
            height: Fit,
            author_name = <ModelLink> {}
            <ExternalLinkIcon> {}
        }

        <View> {
            width: Fit,
            height: Fit,
            <ModelLink> { link = { text: "Hugging Face" } }
            <ExternalLinkIcon> {}
        }
    }

    ModelInformation = <View> {
        width: Fill,
        height: Fit,
        spacing: 10,
        <ModelSummary> {}
        <ModelDetails> {}
    }

    ModelCard = {{ModelCard}} {
        width: Fill,
        height: Fit,

        <RoundedView> {
            width: Fill,
            height: Fit,

            draw_bg: {
                instance radius: 3.0,
                color: #F9FAFB,
                border_color: #DFDFDF,
                border_width: 1.0,
            }

            flow: Down,
            padding: 30,
            spacing: 20,

            <ModelHeading> {}
            <Line> {}
            <ModelInformation> {}
            <ModelFilesList> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelCard {
    #[deref]
    view: View,

    #[rust]
    model_id: String,
}

impl Widget for ModelCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let model = scope.data.get::<Model>();

        self.model_id = model.id.clone();

        let name = &model.name;
        self.label(id!(model_name)).set_text(name);

        let download_count = &model.download_count;
        self.label(id!(model_download_count)).set_text(&format!("{}", download_count));

        let like_count = &model.like_count;
        self.label(id!(model_like_count)).set_text(&format!("{}", like_count));

        let size = &model.size;
        self.label(id!(model_size_tag.attr_value)).set_text(size);

        let requires = &model.requires;
        self.label(id!(model_requires_tag.attr_value)).set_text(requires);

        let architecture = &model.architecture;
        self.label(id!(model_architecture_tag.attr_value)).set_text(architecture);

        let summary = &model.summary;
        const MAX_SUMMARY_LENGTH: usize = 500;
        let trimmed_summary = if summary.len() > MAX_SUMMARY_LENGTH {
            let trimmed = summary.graphemes(true).take(MAX_SUMMARY_LENGTH).collect::<String>();
            format!("{}...", trimmed)
        } else {
            summary.to_string()
        };
        self.label(id!(model_summary)).set_text(&trimmed_summary);

        let author_name = &model.author.name;
        self.link_label(id!(author_name.link)).set_text(author_name);

        let author_description = &model.author.description;
        self.label(id!(author_description)).set_text(&author_description);

        let released_at_str = Store::formatted_model_release_date(model);
        self.label(id!(model_released_at_tag.attr_value)).set_text(&released_at_str);

        self.view.draw_walk(cx, scope, walk)
    }
}