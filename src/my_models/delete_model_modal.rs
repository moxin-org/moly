use makepad_widgets::*;

use crate::{chat::model_selector_list::ModelSelectorListAction, data::store::Store};

use super::downloaded_files_row::DownloadedFilesRowProps;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::MolyButton;
    use crate::shared::resource_imports::*;

    pub DeleteModelModal = {{DeleteModelModal}} {
        width: Fit
        height: Fit

        wrapper = <RoundedView> {
            flow: Down
            width: 600
            height: Fit
            padding: {top: 44, right: 30 bottom: 30 left: 50}
            spacing: 10

            show_bg: true
            draw_bg: {
                color: #fff
                border_radius: 3
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Right

                padding: {top: 8, bottom: 20}

                title = <View> {
                    width: Fit,
                    height: Fit,

                    model_name = <Label> {
                        text: "Delete Model"
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 13},
                            color: #000
                        }
                    }
                }

                filler_x = <View> {width: Fill, height: Fit}

                close_button = <MolyButton> {
                    width: Fit,
                    height: Fit,

                    margin: {top: -8}

                    draw_icon: {
                        svg_file: (ICON_CLOSE),
                        fn get_color(self) -> vec4 {
                            return #000;
                        }
                    }
                    icon_walk: {width: 12, height: 12}
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
                            font_size: 10,
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

                    cancel_button = <MolyButton> {
                        width: Fit,
                        height: Fit,
                        padding: {top: 10, bottom: 10, left: 14, right: 14}

                        draw_bg: {
                            instance border_radius: 2.0,
                            border_color_1: #D0D5DD,
                            border_size: 1.2,
                            color: #fff,
                        }

                        text: "Cancel"
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 10},
                            color: #x0
                        }
                    }

                    delete_button = <MolyButton> {
                        width: Fit,
                        height: Fit,
                        padding: {top: 10, bottom: 10, left: 14, right: 14}

                        draw_bg: {
                            instance border_radius: 2.0,
                            color: #D92D20,
                        }

                        text: "Delete"
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 10},
                            color: #fff
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum DeleteModelModalAction {
    None,
    ModalDismissed,
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
        let props = scope.props.get::<DownloadedFilesRowProps>().unwrap();
        let downloaded_file = &props.downloaded_file;

        self.file_id = downloaded_file.file.id.clone();

        let prompt_text = format!(
            "Are you sure you want to delete {}?\nThis action cannot be undone.",
            downloaded_file.file.name
        );
        self.label(ids!(wrapper.body.delete_prompt))
            .set_text(cx, &prompt_text);

        self.view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }))
    }
}

impl WidgetMatchEvent for DeleteModelModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.button(ids!(close_button)).clicked(actions) {
            cx.action(DeleteModelModalAction::ModalDismissed);
        }

        if self
            .button(ids!(wrapper.body.actions.delete_button))
            .clicked(actions)
        {
            let store = scope.data.get_mut::<Store>().unwrap();
            store.delete_file(self.file_id.clone());

            cx.action(DeleteModelModalAction::ModalDismissed);
            cx.action(ModelSelectorListAction::AddedOrDeletedModel);
        }

        if self
            .button(ids!(wrapper.body.actions.cancel_button))
            .clicked(actions)
        {
            cx.action(DeleteModelModalAction::ModalDismissed);
        }
    }
}
