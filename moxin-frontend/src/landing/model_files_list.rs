use crate::data::store::{ModelWithPendingDownloads, StoreAction};
use makepad_widgets::*;

use super::model_files_items::ModelFilesItemsWidgetExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::landing::shared::*;

    import crate::landing::model_files_items::ModelFilesItems;
    import crate::landing::model_files_items::ModelFilesRow;
    import crate::landing::model_files_tags::ModelFilesTags;

    ICON_ADD = dep("crate://self/resources/icons/add.svg")
    ICON_REMOVE = dep("crate://self/resources/icons/remove.svg")

    ModelFilesRow = <RoundedYView> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #00f
            radius: vec2(1.0, 1.0)
        }

        cell1 = <View> { width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell2 = <View> { width: 140, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell3 = <View> { width: 340, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell4 = <View> { width: 250, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
    }

    FooterLink = <View> {
        cursor: Hand,
        align: {x: 0.0, y: 0.5},
        spacing: 10,
        icon = <Icon> {
            draw_icon: {
                svg_file: (ICON_ADD),
                fn get_color(self) -> vec4 {
                    return #667085;
                }
            }
            icon_walk: {width: 14, height: 14}
        }
        link = <Label> {
            width: Fit,
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 9},
                color: #667085,
            }
        }
    }

    ModelFilesList = {{ModelFilesList}}<RoundedView> {
        width: Fill,
        height: Fit,
        flow: Down,

        show_bg: true,
        draw_bg: {
            color: #EAECF0
            radius: 3.0
        }

        <ModelFilesRow> {
            show_bg: true,
            draw_bg: {
                color: #F2F4F7
                radius: vec2(3.0, 0.5)
            }

            cell1 = {
                height: 40
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #667085
                    }
                    text: "File name"
                }
            }

            cell2 = {
                height: 40
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #667085
                    }
                    text: "Full Size"
                }
            }

            cell3 = {
                height: 40
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 9},
                        color: #667085
                    }
                    text: "Quantization"
                }
            }
            cell4 = {
                height: 40
            }
        }

        <ModelFilesItems> { show_featured: true}
        remaining_files_wrapper = <View> {
            width: Fill,
            height: 0,
            remaining_files = <ModelFilesItems> { show_featured: false}
        }

        footer = <RoundedYView> {
            width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5},

            show_bg: true,
            draw_bg: {
                color: #fff
                radius: vec2(1.0, 3.0)
            }

            all_files_link = <FooterLink> {
                icon = { draw_icon: { svg_file: (ICON_ADD) }}
                link = { text: "Show All Files (12)" }
            }

            only_recommended_link = <FooterLink> {
                visible: false,
                icon = {
                    padding: { top: 10 }
                    draw_icon: { svg_file: (ICON_REMOVE) }
                }
                link = { text: "Show Only Recommended Files" }
            }
        }

        show_all_animation_progress: 0.0,
        animator: {
            show_all = {
                default: hide,
                show = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {show_all_animation_progress: 1.0}
                }
                hide = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {show_all_animation_progress: 0.0}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesList {
    #[deref]
    view: View,

    #[live]
    show_all_animation_progress: f64,

    #[animator]
    animator: Animator,

    #[rust]
    actual_height: Option<f64>,
}

impl Widget for ModelFilesList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            if let Some(total_height) = self.actual_height {
                let height = self.show_all_animation_progress * total_height;
                self.view(id!(remaining_files_wrapper))
                    .apply_over(cx, live! {height: (height)});
                self.redraw(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let model = &scope.data.get::<ModelWithPendingDownloads>().unwrap().model;
        let files_count = model.files.len();
        let all_files_link = self.view(id!(all_files_link.link));
        all_files_link.set_text(&format!("Show All Files ({})", files_count));

        let _ = self.view.draw_walk(cx, scope, walk);

        // Let's remember the actual rendered height of the remaining_files element.
        if self.actual_height.is_none() {
            self.actual_height = Some(self.model_files_items(id!(remaining_files)).get_height(cx))
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ModelFilesList {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(fe) = self.view(id!(all_files_link)).finger_up(&actions) {
            if fe.was_tap() {
                self.apply_links_visibility(cx, true);
                self.animator_play(cx, id!(show_all.show));
                self.redraw(cx);
            }
        }

        if let Some(fe) = self.view(id!(only_recommended_link)).finger_up(&actions) {
            if fe.was_tap() {
                self.apply_links_visibility(cx, false);
                self.animator_play(cx, id!(show_all.hide));
                self.redraw(cx);
            }
        }

        for action in actions.iter() {
            match action.as_widget_action().cast() {
                StoreAction::Search(_) | StoreAction::ResetSearch | StoreAction::Sort(_) => {
                    self.hide_immediate(cx);
                    self.actual_height = None;
                }
                _ => {}
            }
        }
    }
}

impl ModelFilesList {
    fn apply_links_visibility(&mut self, cx: &mut Cx, show_all: bool) {
        self.view(id!(all_files_link))
            .apply_over(cx, live! {visible: (!show_all)});
        self.view(id!(only_recommended_link))
            .apply_over(cx, live! {visible: (show_all)});
    }

    fn hide_immediate(&mut self, cx: &mut Cx) {
        self.apply_links_visibility(cx, false);
        self.view(id!(remaining_files_wrapper))
            .apply_over(cx, live! {height: 0});
        self.show_all_animation_progress = 0.0;
        self.redraw(cx);
    }
}
