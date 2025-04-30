use std::collections::HashMap;

use makepad_widgets::*;

use crate::citation_list::CitationListWidgetExt;
use crate::deep_inquire::{Stage, StageType, SubStage};

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

    SubStage = <StageBlockBase> {
        width: Fill, height: Fit
        padding: {left: 30}
        margin: {left: 30}

        flow: Down
        spacing: 10
        content_heading_label = <Label> {
            width: Fill
            draw_text: {
                wrap: Word
                text_style: <THEME_FONT_BOLD>{font_size: 11},
                color: #003E62
            }
        }
        content_block_markdown = <MessageMarkdown> {}
    }

    SubStages = {{SubStages}} {
        flow: Down
        width: Fill, height: Fit,
        padding: {right: 200}
        spacing: 20

        substage_template: <SubStage> {}
    }

    StageView = {{StageView}}<View> {
        visible: false
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

            expanded_stage_content = <View> {
                visible: false
                flow: Down,
                spacing: 25
                height: Fit
                citations_view = <StageBlockBase> {
                    visible: false
                    height: Fit
                    flow: Down, spacing: 10
                    <Label> {
                        draw_text: {
                            color: #003E62
                            text_style: <THEME_FONT_BOLD> {font_size: 11},
                        }
                        text: "Sources"
                    }
                    citations_list = <CitationList> {}
                }
                substages = <SubStages> {}
            }
        }
        animator: {
            streaming = {
                default: off,
                off = {
                    from: {all: Snap}
                    apply: {
                        wrapper = {
                            header = {
                                stage_toggle = { draw_bg: { shadow_color: #x0007 } }
                            }
                        }
                    }
                }
                pulse_on = {
                    redraw: true,
                    from: {all: Forward { duration: 0.7 }}
                    apply: {
                        // Slightly more opaque shadow
                        wrapper = {
                            header = {
                                stage_toggle = { draw_bg: { shadow_color: #x000A } }
                            }
                        }
                    }
                }
                pulse_off = {
                    redraw: true,
                    from: {all: Forward { duration: 0.7 }}
                    apply: {
                         // Back to default shadow
                        wrapper = {
                            header = {
                                stage_toggle = { draw_bg: { shadow_color: #x0007 } }
                            }
                        }
                    }
                }
            }
        }
    }

    pub Stages = {{Stages}} {
        flow: Down
        visible: false,
        width: Fill, height: Fit,

        thinking_stage = <StageView> {
            stage_type: Thinking
            wrapper = {
                header = {
                    stage_title = { text: "Thinking" }
                    stage_toggle = {
                        stage_bubble_text = { text: "🧠" }
                    }
                }
            }
        }

        content_stage = <StageView> {
            stage_type: Content
            wrapper = {
                header = {
                    stage_title = { text: "Detailed Anaylsis" }
                    stage_toggle = {
                        stage_bubble_text = { text: "🔬" }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Stages {
    #[deref]
    view: View,

    #[rust]
    stage_ids: Vec<String>,
}

impl Widget for Stages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for Stages {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.cast() {
                StageViewAction::StageViewClicked(clicked_stage) => {
                    match clicked_stage {
                        StageType::Thinking => {
                            self.stage_view(id!(content_stage)).set_active(cx, false);
                        }
                        StageType::Content => {
                            self.stage_view(id!(thinking_stage)).set_active(cx, false);
                        }
                        _ => {}
                    }
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }
}

impl Stages {
    fn update_stages(&mut self, cx: &mut Cx, stages: &[Stage]) {
        self.visible = true;
        self.stage_ids = stages.iter().map(|stage| stage.id.clone()).collect();

        let has_content_stage = stages.iter().any(|s| s.stage_type == StageType::Content);
        let has_completion_stage = stages.iter().any(|s| s.stage_type == StageType::Completion);

        for stage in stages.iter() {
            match stage.stage_type {
                StageType::Thinking => {
                    let mut thinking_stage = self.stage_view(id!(thinking_stage));
                    thinking_stage.set_stage(cx, &stage);
                    // Thinking streams if content stage doesn't exist yet
                    thinking_stage.set_streaming_state(cx, !has_content_stage);
                },
                StageType::Content => {
                    let mut content_stage = self.stage_view(id!(content_stage));
                    content_stage.set_stage(cx, &stage);
                    // Content streams if completion stage doesn't exist yet
                    content_stage.set_streaming_state(cx, !has_completion_stage);
                },
                _ => {}
            }
        }

        self.redraw(cx);
    }
}

impl StagesRef {
    pub fn update_stages(&mut self, cx: &mut Cx, stages: &[Stage]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_stages(cx, stages);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct StageView {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    timer: Timer,

    #[rust]
    id: String,

    #[live]
    stage_type: StageType,

    #[rust]
    is_active: bool,

    #[rust]
    is_streaming: bool,
}

impl Widget for StageView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle timer events for looping animation
        if self.timer.is_event(event).is_some() {
            if self.is_streaming {
                if self.animator_in_state(cx, id!(streaming.pulse_on)) {
                    self.animator_play(cx, id!(streaming.pulse_off));
                } else { // Assumes it's in pulse_off or just started
                    self.animator_play(cx, id!(streaming.pulse_on));
                }
                // Restart the timer for the next half cycle
                self.timer = cx.start_timeout(0.7);
            } else {
                 // If streaming stopped while timer was pending, ensure animation is off
                 self.animator_cut(cx, id!(streaming.off));
                 self.timer = Timer::empty();
            }
        }

        if self.animator_handle_event(cx, event).must_redraw() {
             self.redraw(cx);
        }

        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.is_active {
            self.view(id!(stage_toggle)).apply_over(
                cx,
                live! {
                    draw_bg: { border_size: 1 }
                },
            );
        } else {
            self.view(id!(stage_toggle)).apply_over(
                cx,
                live! {
                    draw_bg: { border_size: 0 }
                },
            );
        }

        self.view(id!(expanded_stage_content)).set_visible(cx, self.is_active);
        self.view(id!(stage_content_preview)).set_visible(cx, !self.is_active);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for StageView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(_fe) = self.view(id!(wrapper)).finger_down(actions) {
            self.is_active = !self.is_active;
            
            cx.action(StageViewAction::StageViewClicked(self.stage_type.clone()));
            self.redraw(cx);
        }
    }
}

impl StageView {
    fn set_stage(&mut self, cx: &mut Cx, stage: &Stage) {
        self.id = stage.id.clone();
        self.stage_type = stage.stage_type.clone();
        self.visible = true;

        self.sub_stages(id!(substages)).set_substages(cx, &stage.substages);

        if !stage.citations.is_empty() {
            self.view(id!(citations_view)).set_visible(cx, true);
            let citations = self.citation_list(id!(citations_list));
            let mut citations = citations.borrow_mut().unwrap(); 
            citations.urls = stage.citations.iter().map(|a| a.url.clone()).collect();
        } else {
            self.view(id!(citations_view)).set_visible(cx, false);
        }

        // TODO: this should be replaced in the future by an AI-provided summary
        // Roughly grab the first 10 words of the first substage text to display as a preview
        let stage_preview_text: Option<String> = stage.substages.get(0).and_then(|substage| {
            // Since we're using plain text for summary, remove common markdown characters
            let cleaned_text = substage.text
                .replace("*", "")
                .replace("_", "")
                .replace("#", "")
                .replace("`", "")
                .replace("[", "")
                .replace("]", "")
                .replace("(", "")
                .replace(")", "")
                .replace(">", "");
            
            let words: Vec<&str> = cleaned_text.split_whitespace().collect();
            if words.len() > 10 {
                Some(words[0..10].join(" "))
            } else {
                Some(cleaned_text)
            }
        });

        if let Some(stage_preview_text) = stage_preview_text {
            self.label(id!(stage_preview_label))
                .set_text(cx, &format!("{}...", stage_preview_text));
        } else {
            self.label(id!(stage_preview_label))
                .set_text(cx, "Loading...");
        }

        self.redraw(cx);
    }

    fn set_streaming_state(&mut self, cx: &mut Cx, is_streaming: bool) {
        if is_streaming == self.is_streaming {
            return; // No change
        }
        self.is_streaming = is_streaming;

        if self.is_streaming {
            // Start animation only if timer isn't already running
            if self.timer.is_empty() {
                self.animator_play(cx, id!(streaming.pulse_on));
                self.timer = cx.start_timeout(0.01); // Start timer almost immediately
            }
        } else {
            // Stop animation
            self.animator_cut(cx, id!(streaming.off)); // Go directly to off state
            if !self.timer.is_empty() {
                cx.stop_timer(self.timer);
                self.timer = Timer::empty();
            }
        }
        self.redraw(cx);
    }
}

impl StageViewRef {
    pub fn set_stage(&mut self, cx: &mut Cx, stage: &Stage) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_stage(cx, stage);
        }
    }

    pub fn set_active(&mut self, cx: &mut Cx, is_active: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.is_active = is_active;
            inner.redraw(cx);
        }
    }

    pub fn set_streaming_state(&mut self, cx: &mut Cx, is_streaming: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_streaming_state(cx, is_streaming);
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum StageViewAction {
    None,
    StageViewClicked(StageType),
}

#[derive(Widget, Live, LiveHook)]
pub struct SubStages {
    #[deref]
    view: View,

    #[live]
    substage_template: Option<LivePtr>,

    #[rust]
    substage_ids: Vec<String>,

    #[rust]
    substage_views: HashMap<String, View>,
}

impl Widget for SubStages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible { return DrawStep::done() }
        cx.begin_turtle(walk, self.layout);

        for stage_id in self.substage_ids.iter() {
            if let Some(substage_view) = self.substage_views.get_mut(stage_id) {
                let _ = substage_view.draw(cx, scope);
            }
        }

        cx.end_turtle();
        DrawStep::done()
    }
}

impl SubStages {
    pub fn update_substages(&mut self, cx: &mut Cx, substages: &[SubStage]) {
        self.substage_ids = substages.iter().map(|substage| substage.id.clone()).collect();
        self.visible = true;
        for substage in substages.iter() {
            // If the substage widget exists, update it
            let substage_view = if let Some(substage_view) = self.substage_views.get_mut(&substage.id) {
                substage_view
            } else {
                // Otherwise, create a new substage widget
                let substage_view = View::new_from_ptr(cx, self.substage_template);
                // substage_view.set_stage(cx, &substage);
                self.substage_views.insert(substage.id.clone(), substage_view);
                self.substage_views.get_mut(&substage.id).unwrap()
            };

            substage_view
                .label(id!(content_heading_label))
                .set_text(cx, &get_human_readable_stage_name(&substage.name));
            substage_view
                .view(id!(citations_view))
                .set_visible(cx, false);
            substage_view
                .markdown(id!(content_block_markdown))
                .set_text(cx, &substage.text.replace("\n\n", "\n\n\u{00A0}\n\n"));
        }
    }
}

impl SubStagesRef {
    pub fn set_substages(&mut self, cx: &mut Cx, substages: &[SubStage]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_substages(cx, substages);
        }
    }
}

// Replaces underscores with spaces, and capitalizes the first letter of each word
pub fn get_human_readable_stage_name(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
