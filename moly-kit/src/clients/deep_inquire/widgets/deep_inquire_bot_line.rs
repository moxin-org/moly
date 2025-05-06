use super::stages::StagesWidgetExt;
use crate::deep_inquire::{Data, StageType};
use crate::protocol::*;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::clients::deep_inquire::widgets::stages::*;
    use crate::widgets::message_markdown::*;
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
                    completed_markdown = <MessageMarkdown> {}
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
        let completion_stage = stages
            .iter()
            .find(|stage| stage.stage_type == StageType::Completion);
        if let Some(stage) = completion_stage {
            // Iterate over the text of all substages and present them as one
            let final_text = stage
                .substages
                .iter()
                .map(|s| s.text.clone())
                .collect::<String>();
            self.markdown(id!(completed_block.completed_markdown))
                .set_text(cx, &final_text);
        }
    }
}
