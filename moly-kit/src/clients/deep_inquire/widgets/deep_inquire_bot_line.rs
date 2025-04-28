use super::stages::StagesWidgetExt;
use crate::deep_inquire::Data;
use crate::protocol::*;
use crate::standard_message_content::StandardMessageContentWidgetExt;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::clients::deep_inquire::widgets::stages::*;
    use crate::widgets::standard_message_content::*;
    use crate::widgets::chat_lines::*;

    pub DeepInquireBotLine = {{DeepInquireBotLine}} <BotLine> {
        message_section = {
            content_section = {
                flow: Down,
                <Label> {
                    text: "Steps"
                    draw_text: {
                        color: #x0,
                        text_style: <THEME_FONT_BOLD>{font_size: 12},
                    }
                }

                stages = <Stages> {}

                completed_block = <View> {
                    width: Fill, height: Fit,
                    padding: {right: 18, top: 18, bottom: 14},
                    completed_content = <StandardMessageContent> {}
                }
            }
        }
        actions_section = {
            actions = {
                // Disable the edit action for now, DeepInquire does not really use previous messages as context
                edit = {
                    visible: false
                }
            }
        }

    }
}

#[derive(Widget, Live, LiveHook)]
pub struct DeepInquireBotLine {
    #[deref]
    view: View,
}

impl Widget for DeepInquireBotLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl DeepInquireBotLine {
    pub(crate) fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        let data = content
            .data
            .as_deref()
            .expect("message without custom data should not reach here");

        let data = serde_json::from_str::<Data>(data)
            .expect("custom data without valid format should not reach here");

        let stages = data.stages.as_slice();

        let mut stages_ui = self.view.stages(id!(stages));
        stages_ui.update_stages(cx, stages);

        // Check if there is a completion block in any of the stages
        let stage_with_completion = stages.iter().find(|stage| stage.completed.is_some());
        if let Some(competion_block) = stage_with_completion {
            self.standard_message_content(id!(completed_block.completed_content))
                .set_content(cx, competion_block.completed.as_ref().unwrap());
        }
    }
}
