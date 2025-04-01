use makepad_widgets::*;
use crate::protocol::Message;

use super::stages_pill_list::{StagePillAction, StagesPillListWidgetExt};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::shaders::*;

    use crate::widgets::chat_lines::*;
    use crate::widgets::stages_pill_list::*;
    use crate::widgets::message_markdown::*;
    
    BOLD_FONT = {
        font: {path: dep("crate://makepad-widgets/resources/IBMPlexSans-SemiBold.ttf")}
    }

    // A specialized bot line that supports DeepInquire's multi-stage messages
    pub DeepInquireBotLine = {{DeepInquireBotLine}}<BotLine> {
        message_section = {
            bubble = {
                // Add stages pill list at the top of the bubble
                stages_pill_list = <StagesPillList> {
                    margin: {bottom: 10}
                }
                
                // Container for stage content
                stage_container = <RoundedView> {
                    width: Fill, height: Fit,f
                    visible: false,
                    flow: Down,
                    spacing: 10,
                    
                    thinking_block = <View> {
                        visible: false,
                        flow: Down, spacing: 10,
                        <Label> {
                            text: "Thinking"
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 15},
                                color: #15859A
                            }
                        }
                        width: Fill, height: Fit,
                        padding: {left: 16, right: 18, top: 18, bottom: 14},
                        thinking_markdown = <MessageMarkdown> {}
                    }
                    
                    writing_block = <View> {
                        visible: false,
                        flow: Down, spacing: 10,
                        <Label> {
                            text: "Writing"
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 15},
                                color: #15859A
                            }
                        }
                        width: Fill, height: Fit,
                        padding: {left: 16, right: 18, top: 18, bottom: 14},
                        writing_markdown = <MessageMarkdown> {}
                    }
                    
                    completed_block = <View> {
                        visible: false,
                        flow: Down, spacing: 10,
                        <Label> {
                            text: "Completed"
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 15},
                                color: #15859A
                            }
                        }
                        width: Fill, height: Fit,
                        padding: {left: 16, right: 18, top: 18, bottom: 14},
                        completed_markdown = <MessageMarkdown> {}
                    }
                }
            }
        }
    }
}

#[derive(Widget, Live, LiveHook)]
pub struct DeepInquireBotLine {
    #[deref]
    view: View,

    #[rust]
    message: Option<Message>,

    #[rust]
    selected_stage: Option<usize>,
    
    #[rust]
    user_manually_selected: bool,
}

impl Widget for DeepInquireBotLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for DeepInquireBotLine {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.cast() {
                StagePillAction::StagePillClicked(stage_id) => {
                    // Mark that a user has manually selected a stage
                    self.user_manually_selected = true;
                    self.update_selected_stage(cx, stage_id);
                }
                _ => {}
            }
        }
    }
}

impl DeepInquireBotLine {
    fn set_message(&mut self, cx: &mut Cx, message: &Message, is_streaming: bool) {
        // Skip if message doesn't have stages
        if !message.has_stages() {
            return;
        }

        // Store previous message for comparison
        let previous_message = self.message.clone();
        let is_first_message = previous_message.is_none();
        
        // Always update the message
        self.message = Some(message.clone());
        
        // Get stages from the message
        let stages = message.get_stages();
        
        // Get stage IDs and update pill list
        let stage_ids: Vec<usize> = stages.iter().map(|stage| stage.id).collect();
        let mut stages_pill_list = self.view.stages_pill_list(id!(message_section.bubble.stages_pill_list));
        stages_pill_list.update_stages(cx, &stage_ids);
        
        // Determine which stage to show
        let stage_to_display = if self.user_manually_selected {
            // User has manually selected a stage, try to keep that selection
            if let Some(selected_id) = self.selected_stage {
                if let Some(stage) = stages.iter().find(|s| s.id == selected_id) {
                    Some(stage)
                } else {
                    // If the manually selected stage no longer exists, fall back to the last stage
                    // but keep the "manual selection" flag
                    stages.last()
                }
            } else {
                // Shouldn't happen, but just in case
                stages.last()
            }
        } else {
            // Auto-select the most recent stage (the last one)
            let last_stage = stages.last();
            if let Some(stage) = last_stage {
                self.selected_stage = Some(stage.id);
            }
            last_stage
        };
        
        // Update stage display
        if let Some(stage) = stage_to_display {
            // Update pill list selection
            stages_pill_list.set_selected_stage(cx, stage.id);
            
            // If this is the first message or the selected stage changed, do a full update
            if is_first_message || self.selected_stage != Some(stage.id) {
                self.selected_stage = Some(stage.id);
                self.update_selected_stage(cx, stage.id);
            } else {
                // Just update content for this stage without changing selection
                self.update_stage_content(cx, stage.id);
            }
            
            // If streaming, check for updates in other stages
            if is_streaming && previous_message.is_some() {
                let prev_message = previous_message.unwrap();
                let prev_stages = prev_message.get_stages();
                
                // Check each stage for new content
                for current_stage in &stages {
                    // Skip the currently displayed stage
                    if current_stage.id == stage.id {
                        continue;
                    }
                    
                    // Find matching stage in previous message
                    let stage_changed = if let Some(prev_stage) = prev_stages.iter().find(|s| s.id == current_stage.id) {
                        // Compare each block to see if anything changed
                        let thinking_changed = match (&current_stage.thinking, &prev_stage.thinking) {
                            (Some(curr), Some(prev)) => curr.content != prev.content,
                            (Some(_), None) => true,
                            _ => false
                        };
                        
                        let writing_changed = match (&current_stage.writing, &prev_stage.writing) {
                            (Some(curr), Some(prev)) => curr.content != prev.content,
                            (Some(_), None) => true,
                            _ => false
                        };
                        
                        let completed_changed = match (&current_stage.completed, &prev_stage.completed) {
                            (Some(curr), Some(prev)) => curr.content != prev.content,
                            (Some(_), None) => true,
                            _ => false
                        };
                        
                        thinking_changed || writing_changed || completed_changed
                    } else {
                        // New stage that didn't exist before
                        true
                    };
                    
                    // If the stage changed, set the new content flag
                    if stage_changed {
                        let mut stages_pill_list = self.view.stages_pill_list(id!(message_section.bubble.stages_pill_list));
                        stages_pill_list.set_stage_has_new_content(cx, current_stage.id);
                    }
                }
            }
        }
    }
    
    // Updates just the content of the stage without changing visibility or other settings
    fn update_stage_content(&mut self, cx: &mut Cx, stage_id: usize) {
        if let Some(message) = &self.message {
            if let Some(stage) = message.get_stages().iter().find(|s| s.id == stage_id) {
                // Update thinking block if present
                if let Some(thinking) = &stage.thinking {
                    self.markdown(id!(message_section.bubble.stage_container.thinking_block.thinking_markdown))
                        .set_text(cx, &thinking.content);
                }
                
                // Update writing block if present
                if let Some(writing) = &stage.writing {
                    self.markdown(id!(message_section.bubble.stage_container.writing_block.writing_markdown))
                        .set_text(cx, &writing.content);
                }
                
                // Update completed block if present
                if let Some(completed) = &stage.completed {
                    self.markdown(id!(message_section.bubble.stage_container.completed_block.completed_markdown))
                        .set_text(cx, &completed.content);
                }
            }
        }
    }
    
    fn update_selected_stage(&mut self, cx: &mut Cx, stage_id: usize) {
        // Store the selected stage ID
        self.selected_stage = Some(stage_id);
        
        // Show stage container
        self.view(id!(message_section.bubble.stage_container)).set_visible(cx, true);
        
        // Hide standard message content
        self.view(id!(message_section.bubble.text)).set_visible(cx, false);
        
        // Hide all stage blocks initially
        self.view(id!(message_section.bubble.stage_container.thinking_block)).set_visible(cx, false);
        self.view(id!(message_section.bubble.stage_container.writing_block)).set_visible(cx, false);
        self.view(id!(message_section.bubble.stage_container.completed_block)).set_visible(cx, false);
        
        // Get the message and stage
        if let Some(message) = &self.message {
            if let Some(stage) = message.get_stages().iter().find(|s| s.id == stage_id) {
                // Show thinking block if present
                if let Some(thinking) = &stage.thinking {
                    self.view(id!(message_section.bubble.stage_container.thinking_block)).set_visible(cx, true);
                    self.markdown(id!(message_section.bubble.stage_container.thinking_block.thinking_markdown))
                        .set_text(cx, &thinking.content);
                }
                
                // Update writing block if present
                if let Some(writing) = &stage.writing {
                    self.view(id!(message_section.bubble.stage_container.writing_block)).set_visible(cx, true);
                    self.markdown(id!(message_section.bubble.stage_container.writing_block.writing_markdown))
                        .set_text(cx, &writing.content);
                }
                
                // Update completed block if present
                if let Some(completed) = &stage.completed {
                    self.view(id!(message_section.bubble.stage_container.completed_block)).set_visible(cx, true);
                    self.markdown(id!(message_section.bubble.stage_container.completed_block.completed_markdown))
                        .set_text(cx, &completed.content);
                }
                
                // Update the pill list to show this stage as selected
                let mut stages_pill_list = self.view.stages_pill_list(id!(message_section.bubble.stages_pill_list));
                stages_pill_list.set_selected_stage(cx, stage.id);
                
                // Clear the "new content" indicator for this stage
                stages_pill_list.clear_stage_new_content(cx, stage.id);
            }
        }
        
        // Redraw the widget
        self.redraw(cx);
    }
} 

impl DeepInquireBotLineRef {
    pub fn set_message(&mut self, cx: &mut Cx, message: &Message, is_streaming: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_message(cx, message, is_streaming);
        }
    }
}
