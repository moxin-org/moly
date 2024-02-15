use makepad_widgets::*;
use crate::data::store::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::icon::Icon;
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
            height: Fit,
            <ModelLink> {
                text: "Hugging Face"
            }
        }
    }

    ModelAllFilesInfo = <View> {
        width: Fill,
        height: Fit,
        margin: {left: 10},
        flow: Down,
        spacing: 10,
        model_id_label = <Label> {
            text: "TheBloke/stablelm-zephyr-3b-GGUF",
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 9},
                color: #000
            }
        }
        files_count_label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #000
            }
        }
    }

    ModelAllFiles = {{ModelAllFiles}} {
        width: Fill,
        height: Fill,

        <RectView> {
            width: Fill,
            height: Fill,

            show_bg: true,
            draw_bg: {
                border_color: #D9D9D9,
                border_width: 1.0,
                color: #fff,
            },

            flow: Down,
            padding: 30,
            spacing: 26,

            <ModelHeading> { margin: {left: 10}}
            <ModelAllFilesInfo> {}
            <ModelFilesList> {
                heading_row = { visible: false }
                file_list = { show_tags: false }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelAllFiles {
    #[deref]
    view: View,

    #[rust]
    model_id: String,
}

impl Widget for ModelAllFiles {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(model) = Store::new().models.iter().find(|m| m.name == self.model_id) {
            let _ = self.view.draw_walk(cx, &mut Scope::with_data(&mut model.clone()), walk);
        };

        DrawStep::done()
    }
}

impl ModelAllFilesRef {
    pub fn set_model(&self, model: Model) {
        let Some(mut all_files_widget) = self.borrow_mut() else { return };

        let name = &model.name;
        all_files_widget.label(id!(model_name)).set_text(name);

        let size = &model.size;
        all_files_widget.label(id!(model_size_tag.attr_value)).set_text(size);

        let requires = &model.requires;
        all_files_widget.label(id!(model_requires_tag.attr_value)).set_text(requires);

        let architecture = &model.architecture;
        all_files_widget.label(id!(model_architecture_tag.attr_value)).set_text(architecture);

        let file_count_str = format!("{} Available Files", model.files.len());
        all_files_widget.label(id!(files_count_label)).set_text(&file_count_str);

        // TODO Check later what is the model id
        all_files_widget.model_id = model.name;
    }
}