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

    ModelHeading = <View> {
        height: Fit,
        <View> {
            width: Fit,
            height: Fit,
            flow: Down,
            spacing: 10,
            model_name = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16},
                    color: #000
                }
            }
            <ModelAttributes> {}
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

        author_name = <ModelLink> {}

        author_description = <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                word: Wrap,
                color: #000
            }
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
            <ModelFilesList> { file_list = { show_featured: true } }
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
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let model = scope.data.get::<Model>();

        self.model_id = model.id.clone();

        let name = &model.name;
        self.label(id!(model_name)).set_text(name);

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
        self.link_label(id!(author_name)).set_text(author_name);

        let author_description = &model.author.description;
        self.label(id!(author_description)).set_text(&author_description);

        let released_at_str = Store::formatted_model_release_date(model);
        self.label(id!(model_released_at_tag.attr_value)).set_text(&released_at_str);

        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelCardAction {
    ViewAllFiles(String),
    None,
}

impl WidgetMatchEvent for ModelCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.link_label(id!(all_files_link)).clicked(&actions) {
            let widget_uid = self.widget_uid();
            cx.widget_action(widget_uid, &scope.path, ModelCardAction::ViewAllFiles(self.model_id.clone()));
        }
    }
}