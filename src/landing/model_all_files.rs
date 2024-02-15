use makepad_widgets::*;
use crate::data::store::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::icon::Icon;
    import crate::landing::shared::*;

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

    ModelAllFiles = {{ModelAllFiles}} {
        width: Fill,
        height: Fill,

        <View> {
            width: Fill,
            height: Fill,

            flow: Down,
            padding: 30,
            spacing: 20,

            <ModelHeading> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelAllFiles {
    #[deref]
    view: View,
}

impl Widget for ModelAllFiles {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
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

        let released_at_str = &model.formatted_release_date();
        all_files_widget.label(id!(model_released_at_tag.attr_value)).set_text(&released_at_str);
    }
}