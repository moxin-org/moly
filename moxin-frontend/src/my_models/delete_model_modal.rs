use makepad_widgets::*;
use moxin_protocol::data::ModelID;

use crate::{data::store::Store, shared::modal::ModalAction};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::resource_imports::*;

    DeleteModelModal = {{DeleteModelModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 600
            height: Fit
            padding: {top: 50, right: 30 bottom: 30 left: 50}
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                radius: 3
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Right

                title = <View> {
                    width: Fit,
                    height: Fit,
                    padding: {bottom: 20}

                    model_name = <Label> {
                        text: "Delete Model"
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 16},
                            color: #000
                        }
                    }
                }

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

            body = <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 40,

                delete_prompt = <Label> {
                    width: Fill
                    draw_text: {
                        text_style: <REGULAR_FONT>{
                            font_size: 13,
                            height_factor: 1.3
                        },
                        color: #000
                        wrap: Word
                    }
                }

                actions = <View> {
                    width: Fill, height: Fit
                    flow: Right,
                    align: {x: 1.0, y: 0.5}
                    spacing: 20

                    cancel_button = <RoundedView> {
                        width: Fit,
                        height: Fit,
                        padding: {top: 10, bottom: 10, left: 14, right: 14}
                        cursor: Hand

                        draw_bg: {
                            instance radius: 2.0,
                            border_color: #D0D5DD,
                            border_width: 1.2,
                            color: #fff,
                        }

                        <Label> {
                            text: "Cancel"
                            draw_text:{
                                text_style: <REGULAR_FONT>{font_size: 13},
                                color: #x0
                            }
                        }
                    }
                    delete_button = <RoundedView> {
                        width: Fit,
                        height: Fit,
                        padding: {top: 10, bottom: 10, left: 14, right: 14}
                        cursor: Hand

                        draw_bg: {
                            instance radius: 2.0,
                            color: #D92D20,
                        }

                        <Label> {
                            text: "Delete"
                            draw_text:{
                                text_style: <REGULAR_FONT>{font_size: 13},
                                color: #fff
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DeleteModelModal {
    #[deref]
    view: View,
    #[rust]
    file_id: String,
}

impl Widget for DeleteModelModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let downloaded_files = &scope.data.get::<Store>().unwrap().downloaded_files;

        let downloaded_file = downloaded_files
            .iter()
            .find(|f| f.file.id.eq(&self.file_id))
            .expect("Downloaded file not found");

        let prompt_text = format!(
            "Are you sure you want to delete {}?\nThis action cannot be undone.",
            downloaded_file.file.name
        );
        self.label(id!(wrapper.body.delete_prompt))
            .set_text(&prompt_text);

        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetMatchEvent for DeleteModelModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        if let Some(fe) = self.view(id!(close_button)).finger_up(actions) {
            if fe.was_tap() {
                cx.widget_action(widget_uid, &scope.path, ModalAction::CloseModal);
            }
        }

        if let Some(fe) = self
            .view(id!(wrapper.body.actions.cancel_button))
            .finger_up(actions)
        {
            if fe.was_tap() {
                cx.widget_action(widget_uid, &scope.path, ModalAction::CloseModal);
            }
        }
    }
}

impl DeleteModelModalRef {
    pub fn set_file_id(&mut self, file_id: ModelID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.file_id = file_id;
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DeleteModelAction {
    FileSelected(String),
    None,
}
