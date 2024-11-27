// TODO: Unify with agent button by removing external actions.

use makepad_widgets::*;

use super::prompt_input::PromptInputAction;
use moly_protocol::data::{File, FileID};

live_design!(
    use link::theme::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::shared::ChatAgentAvatar;

    pub ModelButton = {{ModelButton}} <RoundedView> {
        flow: Right,
        width: Fill,
        visible: false,
        height: 40,
        align: { x: 0.0, y: 0.5 },
        padding: { left: 9, top: 4, bottom: 4, right: 9 },
        spacing: 10,

        cursor: Hand
        show_bg: true,
        draw_bg: {
            radius: 0,
            color: #0000
        }

        // agent_avatar = <ChatAgentAvatar> {}
        text_layout = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            spacing: 10

            caption = <Label> {
                width: Fit,
                height: Fit,
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000;
                }
            }
            /*description = <View> {
                visible: false,
                width: Fill,
                height: Fit,
                label = <Label> {
                    width: Fill,
                    height: Fit,
                    draw_text: {
                        wrap: Ellipsis,
                        text_style: <REGULAR_FONT>{font_size: 9, height_factor: 1.1},
                        color: #667085,
                    }
                }
            }*/
        }
        animator: {
            hover = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        draw_bg: {color: #F2F4F700}
                    }
                }
                on = {
                    from: {all: Snap}
                    apply: {
                        draw_bg: {color: #EAECEF88}
                    }
                }
            }
            down = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.5}}
                    ease: OutExp
                    apply: {
                        draw_bg: {down: 0.0}
                    }
                }
                on = {
                    ease: OutExp
                    from: {
                        all: Forward {duration: 0.2}
                    }
                    apply: {
                        draw_bg: {down: 1.0}
                    }
                }
            }
        }
    }
);

#[derive(Live, Widget, LiveHook)]
pub struct ModelButton {
    #[deref]
    view: View,

    #[rust]
    file_id: Option<FileID>,
}

impl Widget for ModelButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelButton {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let Some(file_id) = &self.file_id else { return };

        if let Some(item) = actions.find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerDown(fd) = item.cast() {
                if fd.tap_count == 1 {
                    cx.action(PromptInputAction::ModelFileSelected(file_id.clone()));
                }
            }
        }
    }
}

impl ModelButton {
    pub fn set_file(&mut self, file: &File) {
        self.visible = true;
        self.label(id!(caption)).set_text(&file.name);
        self.file_id = Some(file.id.clone());
    }
}

impl ModelButtonRef {
    pub fn set_file(&mut self, file: &File) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_file(file);
        }
    }
}
