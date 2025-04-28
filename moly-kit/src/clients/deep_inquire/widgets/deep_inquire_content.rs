use super::stages::StagesWidgetExt;
use crate::deep_inquire::Data;
use crate::protocol::*;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    // use crate::widgets::stages::*;
    // use crate::widgets::message_markdown::*;

    pub DeepInquireContent = {{DeepInquireContent}} {
        width: 100,
        height: 100,
        show_bg: true,
        draw_bg: { color: #f00},
        // flow: Down,
        // height: Fit,
        // <Label> {
        //     text: "Steps"
        //     draw_text: {
        //         color: #x0,
        //         text_style: <THEME_FONT_BOLD>{font_size: 12},
        //     }
        // }

        // stages = <Stages> {}

        // completed_block = <View> {
        //     width: Fill, height: Fit,
        //     padding: {right: 18, top: 18, bottom: 14},
        //     completed_markdown = <MessageMarkdown> {}
        // }
    }
}

#[derive(Widget, Live, LiveHook)]
pub struct DeepInquireContent {
    #[deref]
    view: View,
}

impl Widget for DeepInquireContent {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        println!("DeepInquireContent draw_walk");
        self.view.draw_walk(cx, scope, walk)
    }
}

impl DeepInquireContent {
    pub(crate) fn set_content(&mut self, cx: &mut Cx, content: &MessageContent) {
        let data = content
            .data
            .as_deref()
            .expect("message without custom data should not reach here");

        let data = serde_json::from_str::<Data>(data)
            .expect("custom data without valid format should not reach here");

        let stages = data.stages.as_slice();

        let mut stages_ui = self.view.stages(id!(stages));
        stages_ui.update_stages(cx, &stages);

        // Check if there is a completion block in any of the stages
        let stage_with_completion = stages.iter().find(|stage| stage.completed.is_some());
        if let Some(competion_block) = stage_with_completion {
            self.markdown(id!(completed_block.completed_markdown))
                .set_text(
                    cx,
                    &competion_block
                        .completed
                        .as_ref()
                        .unwrap()
                        .text
                        .replace("\n\n", "\n\n\u{00A0}\n\n"),
                );
        }
    }
}
