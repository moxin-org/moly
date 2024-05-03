use crate::data::store::{ModelWithPendingDownloads, Store};
use crate::shared::external_link::ExternalLinkWidgetExt;
use crate::shared::modal::ModalAction;
use crate::shared::utils::{hugging_face_model_url, human_readable_model_name};
use makepad_widgets::*;
use moxin_protocol::data::ModelID;
use unicode_segmentation::UnicodeSegmentation;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::resource_imports::*;
    import crate::shared::widgets::*;
    import crate::landing::shared::*;
    import crate::landing::model_files_list::ModelFilesList;
    import crate::shared::external_link::*;

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


            <VerticalFiller> {}

            model_like_count = <ModelAttributeTag> {
                width: Fit,
                height: Fit,

                padding: {top: 6, bottom: 6, left: 6, right: 10}

                draw_bg: {
                    color: #0000,
                    border_color: #98A2B3,
                    border_width: 1.0,
                },
                attr_name = <Icon> {
                    draw_icon: {
                        svg_file: (ICON_FAVORITE),
                        fn get_color(self) -> vec4 {
                            return #000;
                        }
                    }
                    icon_walk: {width: 14, height: 14}
                }

                attr_value = {
                    margin: {left: 5},
                    draw_text: {
                        color: #000
                        text_style: <REGULAR_FONT>{font_size: 9},
                    }
                }
            }

            model_download_count = <ModelAttributeTag> {
                width: Fit,
                height: Fit,

                padding: {top: 6, bottom: 6, left: 6, right: 10}

                draw_bg: {
                    color: #0000,
                    border_color: #98A2B3,
                    border_width: 1.0,
                },
                attr_name = <Icon> {
                    draw_icon: {
                        svg_file: (ICON_DOWNLOADS),
                        fn get_color(self) -> vec4 {
                        return #000;
                        }
                    }
                    icon_walk: {width: 12, height: 12}
                }

                attr_value = {
                    margin: {left: 5},
                    draw_text: {
                        color: #000
                        text_style: <REGULAR_FONT>{font_size: 9},
                    }
                }
            }


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

        view_all_button = <ModelLink> {
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
        icon_walk: {width: 14, height: 14}
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
            text: "Resources"
        }

        <View> {
            align: {x: 0.5, y: 1.0},
            width: Fit,
            height: Fit,
            author_link = <ExternalLink> {}
            <ExternalLinkIcon> {}
        }

        <View> {
            align: {x: 0.5, y: 1.0},
            width: Fit,
            height: Fit,
            model_hugging_face_link = <ExternalLink> { link = { text: "Hugging Face" } }
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


    ModelCardViewAllModal = {{ModelCardViewAllModal}} {
        width: Fit
        height: Fit

        <RoundedView> {
            flow: Down
            width: 600
            height: Fit
            padding: {top: 30, right: 30 bottom: 50 left: 50}
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                radius: 3
            }

            <View> {
                width: Fill,
                height: Fit,
                filler_x = <View> {width: Fill, height: Fit}

                close_button = <RoundedView> {
                    width: Fit,
                    height: Fit,
                    align: {x: 0.5, y: 0.5}
                    cursor: Hand

                    button_icon = <Icon> {
                        draw_icon: {
                            svg_file: (ICON_CLOSE),
                            fn get_color(self) -> vec4 {
                                return #000;
                            }
                        }
                        icon_walk: {width: 12, height: 12}
                    }
                }
            }

            <View> {
                width: Fill,
                height: Fit,
                padding: {bottom: 20}

                model_name = <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 16},
                        color: #000
                    }
                }
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 10,

                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #000
                    }
                    text: "Model Description"
                }
                model_summary = <Label> {
                    width: Fill,
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 9},
                        word: Wrap,
                        color: #000
                    }
                }
            }
        }
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
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let model = &scope.data.get::<ModelWithPendingDownloads>().unwrap().model;

        self.model_id = model.id.clone();

        let name = &human_readable_model_name(&model.name);
        self.label(id!(model_name)).set_text(name);

        let download_count = &model.download_count;
        self.label(id!(model_download_count.attr_value))
            .set_text(&format!("{}", download_count));

        let like_count = &model.like_count;
        self.label(id!(model_like_count.attr_value))
            .set_text(&format!("{}", like_count));

        let size = &model.size;
        self.label(id!(model_size_tag.attr_value)).set_text(size);

        let requires = &model.requires;
        self.label(id!(model_requires_tag.attr_value))
            .set_text(requires);

        let architecture = &model.architecture;
        self.label(id!(model_architecture_tag.attr_value))
            .set_text(architecture);

        let summary = &model.summary;
        const MAX_SUMMARY_LENGTH: usize = 500;
        let trimmed_summary = if summary.len() > MAX_SUMMARY_LENGTH {
            let trimmed = summary
                .graphemes(true)
                .take(MAX_SUMMARY_LENGTH)
                .collect::<String>();
            format!("{}...", trimmed)
        } else {
            summary.to_string()
        };
        self.label(id!(model_summary)).set_text(&trimmed_summary);

        let author_name = &model.author.name;
        let author_url = &model.author.url;
        let mut author_external_link = self.external_link(id!(author_link));
        author_external_link
            .link_label(id!(link))
            .set_text(author_name);
        author_external_link.set_url(author_url);

        let model_hugging_face_url = hugging_face_model_url(&model.id);
        let mut model_hugging_face_external_link = self.external_link(id!(model_hugging_face_link));
        model_hugging_face_external_link.set_url(&model_hugging_face_url);

        let author_description = &model.author.description;
        self.label(id!(author_description))
            .set_text(author_description);

        let released_at_str = Store::formatted_model_release_date(&model);
        self.label(id!(model_released_at_tag.attr_value))
            .set_text(&released_at_str);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        if self.link_label(id!(view_all_button.link)).clicked(actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                ModalAction::ShowModalView(live_id!(model_card_view_all_modal_view)),
            );

            cx.widget_action(
                widget_uid,
                &scope.path,
                ViewAllModalAction::ModelSelected(self.model_id.clone()),
            );
        }
    }
}

// TODO: Make it so this action and setting the model id from the outside isnt needed
//       find a way to pass the data inside the Modal action itself.
#[derive(Clone, DefaultNone, Eq, Hash, PartialEq, Debug)]
pub enum ViewAllModalAction {
    None,
    ModelSelected(ModelID),
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelCardViewAllModal {
    #[deref]
    view: View,
    #[rust]
    model_id: ModelID,
}

impl Widget for ModelCardViewAllModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let model = store.models.iter().find(|model| model.id == self.model_id);

        if let Some(model) = model {
            let name = &model.name;
            self.label(id!(model_name)).set_text(name);

            let summary = &model.summary;
            self.label(id!(model_summary)).set_text(summary);
        }

        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetMatchEvent for ModelCardViewAllModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        if let Some(fe) = self.view(id!(close_button)).finger_up(actions) {
            if fe.was_tap() {
                cx.widget_action(widget_uid, &scope.path, ModalAction::CloseModal);
            }
        }
    }
}

impl ModelCardViewAllModalRef {
    pub fn set_model_id(&mut self, model_id: ModelID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.model_id = model_id;
        }
    }
}
