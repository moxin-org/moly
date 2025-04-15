use std::collections::HashMap;

use makepad_widgets::*;

use crate::MessageStage;

use super::citation_list::CitationListWidgetRefExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use makepad_code_editor::code_view::CodeView;
    use crate::widgets::citation_list::*;
    use crate::widgets::message_markdown::*;


    // A workaround for RoundedShadowView having the border_size defined as a uniform,
    // which breaks whenever updated through apply_over. This custom version replaces the properties with `instance` fields instead.
    // This will be fixed in Makepad.
    CustomRoundedShadowView = <View>{
        clip_x:false,
        clip_y:false,
                            
        show_bg: true,
        draw_bg: {
            color: #8
            instance border_radius: 2.5
            instance border_size: 0.0
            instance border_color: #0000
            instance shadow_color: #0007
            instance shadow_radius: 20.0,
            instance shadow_offset: vec2(0.0,0.0)
                                            
            varying rect_size2: vec2,
            varying rect_size3: vec2,
            varying rect_pos2: vec2,     
            varying rect_shift: vec2,    
            varying sdf_rect_pos: vec2,
            varying sdf_rect_size: vec2,
                                              
            fn get_color(self) -> vec4 {
                return self.color
            }
                                            
            fn vertex(self) -> vec4 {
                let min_offset = min(self.shadow_offset,vec2(0));
                self.rect_size2 = self.rect_size + 2.0*vec2(self.shadow_radius);
                self.rect_size3 = self.rect_size2 + abs(self.shadow_offset);
                self.rect_pos2 = self.rect_pos - vec2(self.shadow_radius) + min_offset;
                self.sdf_rect_size = self.rect_size2 - vec2(self.shadow_radius * 2.0 + self.border_size * 2.0)
                self.sdf_rect_pos = -min_offset + vec2(self.border_size + self.shadow_radius);
                self.rect_shift = -min_offset;
                                                            
                return self.clip_and_transform_vertex(self.rect_pos2, self.rect_size3)
            }
                                                        
            fn get_border_color(self) -> vec4 {
                return self.border_color
            }
                                                
            fn pixel(self) -> vec4 {
                                                                
                let sdf = Sdf2d::viewport(self.pos * self.rect_size3)
                sdf.box(
                    self.sdf_rect_pos.x,
                    self.sdf_rect_pos.y,
                    self.sdf_rect_size.x,
                    self.sdf_rect_size.y, 
                    // max(1.0, self.border_radius)
                    self.border_radius
                )
                if sdf.shape > -1.0{ // try to skip the expensive gauss shadow
                    let m = self.shadow_radius;
                    let o = self.shadow_offset + self.rect_shift;
                    let v = GaussShadow::rounded_box_shadow(vec2(m) + o, self.rect_size2+o, self.pos * (self.rect_size3+vec2(m)), self.shadow_radius*0.5, self.border_radius*2.0);
                    sdf.clear(self.shadow_color*v)
                }
                                                                    
                sdf.fill_keep(self.get_color())
                if self.border_size > 0.0 {
                    sdf.stroke(self.get_border_color(), self.border_size)
                }
                return sdf.result
            }
        }
    }

    StageBlockBase = <View> {
        padding: {left: 30}
        margin: {left: 30}            
        width: Fill, height: 20
        show_bg: true
        draw_bg: {
            color: #f9f9f9        
            instance left_border_color: #eaeaea
            instance left_border_width: 3.0

            fn pixel(self) -> vec4 {        
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                // draw bg
                sdf.rect(0., 0., self.rect_size.x, self.rect_size.y);
                sdf.fill(self.color);

                // draw the left vertical line
                sdf.rect(0., 0., self.left_border_width, self.rect_size.y);
                sdf.fill(self.left_border_color);

                return sdf.result;
            }
        }
    }

    StageContentBlock = <StageBlockBase> {
        visible: false
        width: Fill, height: Fit
        padding: {left: 30}
        margin: {left: 30}

        flow: Down
        spacing: 8
        content_heading_label = <Label> {
            width: Fill
            draw_text: {
                wrap: Word
                text_style: <THEME_FONT_BOLD>{font_size: 11},
                color: #003E62
            }
        }
        content_block_markdown = <MessageMarkdown> {}

        citations_view = <View> {
            visible: false
            height: Fit
            flow: Down, spacing: 10
            <Label> {
                draw_text: {
                    color: #000
                    text_style: {font_size: 10},
                }                
                text: "Sources"
            }
            citations_list = <CitationList> {}
        }
    }

    StageView = {{StageView}}<View> {
        width: Fill, height: Fit,
        wrapper = <View> {
            width: Fill, height: Fit,
            cursor: Hand
            flow: Down
            align: {x: 0, y: 0.5}
            header = <View> {
                width: Fill, height: Fit,
                spacing: 10
                align: {x: 0, y: 0.5}
                padding: 10

                stage_toggle = <CustomRoundedShadowView> {
                    width: 40, height: 40
                    padding: 4
                    draw_bg: {
                        color: #f9f9f9,
                        border_radius: 10.0,
                        uniform shadow_color: #0001
                        shadow_radius: 8.0,
                        shadow_offset: vec2(0.0,-2.0)
                        border_size: 0.0,
                        border_color: #1A2533
                    }
                    align: {x: 0.5, y: 0.5}

                    stage_bubble_text = <Label> {
                        text: "1"
                        draw_text: {
                            text_style: <THEME_FONT_BOLD>{font_size: 10},
                            color: #000
                        }
                    }
                }
                stage_title = <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_BOLD>{font_size: 10},
                        color: #000
                    }
                }
            }
            stage_content_preview = <StageBlockBase> {
                padding: {left: 30}
                margin: {left: 30}            
                width: Fill, height: 20

                stage_preview_label = <Label> {
                    width: Fill
                    draw_text: {
                        wrap: Word
                        text_style: {font_size: 10},
                        color: #x0
                    }
                }
            }

            stage_content = <View> {
                visible: false
                width: Fill, height: Fit
                spacing: 20
                flow: Down
                thinking_content_block = <StageContentBlock> { content_heading_label = { text: "Thinking" } }
                writing_content_block = <StageContentBlock> { content_heading_label = { text: "Content" } }
            }
        }
    }

    pub Stages = {{Stages}} {
        flow: Down
        visible: false,
        width: Fill, height: Fit,
        padding: {right: 200}

        stage_view_template: <StageView> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Stages {
    #[deref]
    view: View,

    /// The template for the citation views.
    #[live]
    stage_view_template: Option<LivePtr>,

    #[rust]
    stage_ids: Vec<usize>,

    #[rust]
    stage_views: HashMap<usize, StageView>,

    #[rust]
    active_stage: usize,
}

impl Widget for Stages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        for pill in self.stage_views.values_mut() {
            pill.handle_event(cx, event, scope);
        }

        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        for stage_id in self.stage_ids.iter() {
            if let Some(stage_pill) = self.stage_views.get_mut(&stage_id) {
                let _ = stage_pill.draw(cx, scope);
            }
        }
        
        cx.end_turtle();
        DrawStep::done()
    }
}

impl WidgetMatchEvent for Stages {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.cast() {
                StageViewAction::StageViewClicked(stage_id) => {
                    self.active_stage = stage_id;
                    for (id, pill) in self.stage_views.iter_mut() {
                        if *id == stage_id {
                        } else {
                            pill.is_active = false;
                        }
                        pill.redraw(cx);
                    }

                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }
}

impl Stages {
    fn update_stages(&mut self, cx: &mut Cx, stages: &Vec<MessageStage>) {
        self.visible = true;
        self.stage_ids = stages.iter().map(|stage| stage.id).collect();
        self.visible = true;

        for stage in stages.iter() {
            // If the stage widget exists, update it
            if let Some(stage_view) = self.stage_views.get_mut(&stage.id) {
                stage_view.set_stage(cx, &stage);
            } else {
            // Othwerise, create a new stage widget
                let mut stage_view = StageView::new_from_ptr(cx, self.stage_view_template);
                stage_view.set_stage(cx, &stage);                    

                self.stage_views.insert(stage.id, stage_view);
            }
        }

        self.redraw(cx);
    }
}

impl StagesRef {
    pub fn update_stages(&mut self, cx: &mut Cx, stages: &Vec<MessageStage>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_stages(cx, stages);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct StageView {
    #[deref]
    view: View,

    #[rust]
    id: usize,

    #[rust]
    is_active: bool,

    #[rust]
    has_new_content: bool,
}

impl Widget for StageView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.is_active {
            self.view(id!(stage_toggle)).apply_over(cx, 
                live! {
                    draw_bg: { border_size: 1 }
                }
            );
        } else {
            self.view(id!(stage_toggle)).apply_over(cx, 
                live! {
                    draw_bg: { border_size: 0 }
                }
            );
            self.view(id!(content)).set_visible(cx, false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for StageView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(_fe) = self.view(id!(wrapper)).finger_down(actions) {
            self.is_active = !self.is_active;
            self.has_new_content = false;
            self.view(id!(stage_content)).set_visible(cx, self.is_active);
            self.view(id!(stage_content_preview)).set_visible(cx, !self.is_active);
            cx.action(StageViewAction::StageViewClicked(self.id));
        }
    }
}

impl StageView {
    fn set_stage(&mut self, cx: &mut Cx, stage: &MessageStage) {
        self.id = stage.id;
        self.label(id!(stage_title)).set_text(cx, &format!("Stage {}", stage.id + 1));
        self.label(id!(stage_bubble_text)).set_text(cx, &format!("{}", stage.id + 1));

        // Roughly the first 10 words of the thinking block of the stage
        // TODO: this will be replaced in the future by an AI-provided summary
        let mut stage_preview_text = None;

        if let Some(thinking) = &stage.thinking {
            let thinking_content_block = self.view(id!(thinking_content_block));                    
            thinking_content_block.set_visible(cx, true);
            thinking_content_block.markdown(id!(content_block_markdown)).set_text(cx, &thinking.content.replace("\n\n", "\n\n\u{00A0}\n\n"));
            stage_preview_text = Some(thinking.content.split_whitespace()
            .take(10)
            .collect::<Vec<_>>()
            .join(" "));
        }

        if let Some(writing) = &stage.writing {
            let writing_content_block = self.view(id!(writing_content_block));
            writing_content_block.set_visible(cx, true);
            writing_content_block.markdown(id!(content_block_markdown)).set_text(cx, &writing.content.replace("\n\n", "\n\n\u{00A0}\n\n"));

            // Set citations from the message
            if !writing.citations.is_empty() {
                writing_content_block.view(id!(citations_view)).set_visible(cx, true);
                let citations = writing_content_block.citation_list(id!(citations_list));
                let mut citations = citations.borrow_mut().unwrap();
                citations.urls = writing.citations.clone();
            }
        }

        if let Some(stage_preview_text) = stage_preview_text {
            self.label(id!(stage_preview_label)).set_text(cx, &format!("{}...", stage_preview_text));
        } else {
            self.label(id!(stage_preview_label)).set_text(cx, "Loading...");
        }

        self.redraw(cx);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum StageViewAction {
    None,
    StageViewClicked(usize),
}
