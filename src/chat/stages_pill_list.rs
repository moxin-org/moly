use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use makepad_code_editor::code_view::CodeView;
    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::resource_imports::*;

    StagePill = {{StagePill}}<View> {
        width: Fill, height: Fit,
        wrapper_view = <RoundedView> {
            cursor: Hand,
            flow: Right, spacing: 4
            width: Fit, height: Fit
            padding: {left: 10, right: 10, top: 7, bottom: 7}
            draw_bg: { color: #f2f2f2, border_color: #f2f2f2, radius: 0, border_width: 2 }
            stage_text = <Label> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 10},
                    color: #000
                }
            }
            new_content_badge = <RoundedView> {
                visible: false
                width: 10, height: 10,
                show_bg: true
                draw_bg: {
                    // color: #b33939,
                    color: #c23616
                    radius: 3,
                }
            }
        }
    }

    pub StagesPillList = {{StagesPillList}} {
        cursor: Hand
        flow: RightWrap, spacing: 0
        visible: false,
        width: Fit, height: Fit,

        stage_pill_template: <StagePill> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct StagesPillList {
    #[deref]
    view: View,

    /// The template for the citation views.
    #[live]
    stage_pill_template: Option<LivePtr>,

    #[rust]
    stages: Vec<usize>,

    #[rust]
    stage_pills: ComponentMap<usize, StagePill>,

    #[rust]
    active_stage: usize,
}

impl Widget for StagesPillList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        for pill in self.stage_pills.values_mut() {
            pill.handle_event(cx, event, scope);
        }

        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        // Draw pills in the order defined by self.stages instead of map order
        for (index, &stage_id) in self.stages.iter().enumerate() {
            if let Some(stage_pill) = self.stage_pills.get_mut(&index) {
                stage_pill.set_id(stage_id);

                if index == self.active_stage {
                    // #f2f2f2 -> #e0e0e0
                    let darker_color = vec4(0.882, 0.882, 0.882, 1.0);
                    stage_pill.view(id!(wrapper_view)).apply_over(cx, live!{
                        draw_bg: {
                            border_color: (darker_color)
                        }
                    });
                } else {
                    // #f2f2f2
                    let normal_color = vec4(0.949, 0.949, 0.949, 1.0);
                    stage_pill.view(id!(wrapper_view)).apply_over(cx, live!{
                        draw_bg: {
                            border_color: (normal_color)
                        }
                    });
                }

                let _ = stage_pill.draw(cx, scope);
            }
        }
        
        cx.end_turtle();
        DrawStep::done()
    }
}

impl WidgetMatchEvent for StagesPillList {
    fn handle_actions(&mut self, _cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.cast() {
                StagePillAction::StagePillClicked(stage_id) => {
                    self.active_stage = stage_id;
                    self.redraw(_cx);
                }
                _ => {}
            }
        }
    }
}

impl StagesPillList {
    fn update_stages(&mut self, cx: &mut Cx, stages: &Vec<usize>) {
        self.visible = true;
        self.stages = stages.clone();
        self.visible = true;

        for (index, _stage) in stages.iter().enumerate() {
            // Only create new stage pills if they don't exist
            if !self.stage_pills.contains_key(&index) {
                let new_stage_pill = StagePill::new_from_ptr(cx, self.stage_pill_template);
                self.stage_pills.insert(index, new_stage_pill);
            }
        }

        self.redraw(cx);
    }

    fn set_selected_stage(&mut self, cx: &mut Cx, stage_id: usize) {
        self.active_stage = stage_id;
        self.redraw(cx);
    }

    fn set_stage_has_new_content(&mut self, cx: &mut Cx, stage_id: usize) {
        if let Some(stage_pill) = self.stage_pills.get_mut(&stage_id) {
            stage_pill.has_new_content = true;
            self.redraw(cx);
        }
    }
}

impl StagesPillListRef {
    pub fn update_stages(&mut self, cx: &mut Cx, stages: &Vec<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_stages(cx, stages);
        }
    }

    pub fn set_selected_stage(&mut self, cx: &mut Cx, stage_id: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected_stage(cx, stage_id);
        }
    }

    pub fn set_stage_has_new_content(&mut self, cx: &mut Cx, stage_id: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_stage_has_new_content(cx, stage_id);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct StagePill {
    #[deref]
    view: View,

    #[rust]
    id: usize,

    #[rust]
    is_active: bool,

    #[rust]
    has_new_content: bool,
}

impl Widget for StagePill {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let stage_title = format!("Stage {}", self.id + 1);
        self.label(id!(stage_text)).set_text(cx, &stage_title);

        if self.has_new_content {
            self.view(id!(new_content_badge)).set_visible(cx, true);
        } else {
            self.view(id!(new_content_badge)).set_visible(cx, false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for StagePill {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(_fe) = self.view(id!(wrapper_view)).finger_down(actions) {
            self.is_active = true;
            self.has_new_content = false;
            cx.action(StagePillAction::StagePillClicked(self.id));
        }
    }
}

impl StagePill {
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum StagePillAction {
    None,
    StagePillClicked(usize),
}
