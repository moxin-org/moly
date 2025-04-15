use makepad_widgets::*;
use crate::protocol::Message;

use super::{message_loading::MessageLoadingWidgetExt, stages::StagesWidgetExt};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::widgets::chat_lines::*;
    use crate::widgets::message_loading::*;
    use crate::widgets::stages::*;
    use crate::widgets::message_markdown::*;
    

    // A specialized bot line that supports DeepInquire's multi-stage messages
    pub DeepInquireBotLine = {{DeepInquireBotLine}}<BotLine> {
        message_section = {
            bubble = {

                <Label> {
                    text: "Steps"
                    draw_text: {
                        color: #x0,
                        text_style: <THEME_FONT_BOLD>{font_size: 12},
                    }
                }
                // Add stages pill list at the top of the bubble
                stages = <Stages> {}

                                    
                completed_block = <View> {
                    visible: false,
                    width: Fill, height: Fit,
                    padding: {right: 18, top: 18, bottom: 14},
                    completed_markdown = <MessageMarkdown> {}
                }

                loading_block = <View> {
                    visible: false
                    height: Fit, width: 600
                    padding: {left: 30, top: 10}
                    message_loading = <MessageLoading> {}
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
    fn set_message(&mut self, cx: &mut Cx, message: &Message, is_streaming: bool) {
        // Skip if message doesn't have stages
        if !message.has_stages() {
            return;
        }

        // Get stages from the message
        let stages = message.get_stages();
        
        let mut stages_ui = self.view.stages(id!(message_section.bubble.stages));
        stages_ui.update_stages(cx, &stages);

        // Check if there is a completion block in any of the stages
        let stage_with_completion = stages.iter().find(|stage| stage.completed.is_some());
        if let Some(competion_block) = stage_with_completion {
            self.view(id!(completed_block)).set_visible(cx, true);
            self.markdown(id!(completed_block.completed_markdown))
                .set_text(cx, &competion_block.completed.as_ref().unwrap().content.replace("\n\n", "\n\n\u{00A0}\n\n"));

            self.view(id!(loading_block)).set_visible(cx, false);
            self.message_loading(id!(message_loading)).stop_animation();
        } else if is_streaming {
            self.view(id!(completed_block)).set_visible(cx, false);
            self.view(id!(loading_block)).set_visible(cx, true);
            self.message_loading(id!(message_loading)).animate(cx);
        }
    }
} 

impl DeepInquireBotLineRef {
    pub fn set_message(&mut self, cx: &mut Cx, message: &Message, is_streaming: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_message(cx, message, is_streaming);
        }
    }
}
