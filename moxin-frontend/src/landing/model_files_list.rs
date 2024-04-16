use crate::{
    data::store::{Store, StoreAction},
    shared::utils::format_model_size,
};
use makepad_widgets::*;
use moxin_protocol::data::{File, Model};
use std::collections::HashMap;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::landing::shared::*;

    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    ICON_DOWNLOAD_DONE = dep("crate://self/resources/icons/download_done.svg")
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

    ModelFilesListLabel = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        draw_bg: {
            instance radius: 2.0,
            color: #E6F1EC,
        }

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #1C1917
            }
        }
    }

    ModelCardButton = <RoundedView> {
        width: 140,
        height: 32,
        align: {x: 0.5, y: 0.5}
        spacing: 6,

        draw_bg: { color: #099250 }

        button_icon = <Icon> {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
            icon_walk: {width: Fit, height: Fit}
        }

        button_label = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return #fff;
                }
            }
        }
    }

    DownloadButton = <ModelCardButton> {
        cursor: Hand,
        button_label = { text: "Download" }
        button_icon = { draw_icon: {
            svg_file: (ICON_DOWNLOAD),
        }}
    }

    DownloadedButton = <ModelCardButton> {
        draw_bg: { color: #fff, border_color: #099250, border_width: 0.5}
        button_label = {
            text: "Downloaded"
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #099250;
                }
            }
        }
        button_icon = {
            draw_icon: {
                svg_file: (ICON_DOWNLOAD_DONE),
                fn get_color(self) -> vec4 {
                    return #099250;
                }
            }
        }
    }

    ModelFilesTags = {{ModelFilesTags}} {
        width: Fit,
        height: Fit,
        flow: Right,
        spacing: 5,

        template: <ModelFilesListLabel> {}
    }

    ModelFilesRowWithData = <ModelFilesRow> {
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        cell1 = {
            spacing: 10,
            filename = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 9},
                    color: #000
                }
            }
        }

        cell2 = {
            full_size = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 9},
                    color: #000
                }
            }
        }

        cell3 = {
            spacing: 6,
            quantization_tag = <RoundedView> {
                width: Fit,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 10, right: 10}

                draw_bg: {
                    instance radius: 2.0,
                    border_color: #B4B4B4,
                    border_width: 0.5,
                    color: #FFF,
                }

                quantization = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 9},
                        color: #000
                    }
                }
            }
            tags = <ModelFilesTags> {}
        }

        cell4 = {
            align: {x: 0.5, y: 0.5},
        }
    }

    ModelFilesItems = {{ModelFilesItems}} {
        width: Fill,
        height: Fit,
        flow: Down,

        template_downloaded: <ModelFilesRowWithData> {
            cell4 = {
                <DownloadedButton> {}
            }
        }

        template_download: <ModelFilesRowWithData> {
            cell4 = {
                download_button = <DownloadButton> {}
            }
        }
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
                    text: "Model File"
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

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelFileItemsAction {
    Download(File, Model),
    None,
}

#[derive(Live, LiveHook, LiveRegisterWidget, WidgetRef)]
pub struct ModelFilesItems {
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template_downloaded: Option<LivePtr>,
    #[live]
    template_download: Option<LivePtr>,

    #[live(true)]
    show_tags: bool,

    #[live(false)]
    show_featured: bool,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,

    #[rust]
    map_to_files: HashMap<LiveId, File>,

    #[rust]
    model: Option<Model>,
}

impl Widget for ModelFilesItems {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (id, item) in self.items.iter_mut() {
            let actions = cx.capture_actions(|cx| item.handle_event(cx, event, scope));
            if let Some(fd) = item.view(id!(download_button)).finger_down(&actions) {
                if fd.tap_count == 1 {
                    let widget_uid = item.widget_uid();
                    cx.widget_action(
                        widget_uid,
                        &scope.path,
                        ModelFileItemsAction::Download(
                            self.map_to_files.get(id).unwrap().clone(),
                            self.model.clone().unwrap(),
                        ),
                    );
                }
            }
        }

        // When data changes, we need to reset the items hash, so the PortalList
        // can properly update the items (otherwise the used template widget are never changed).
        if let Event::Signal = event {
            self.items.clear();
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let model = scope.data.get::<Model>().unwrap();
        let files = if self.show_featured {
            Store::model_featured_files(model)
        } else {
            Store::model_other_files(model)
        };
        cx.begin_turtle(walk, self.layout);

        self.model.get_or_insert(model.clone());
        self.draw_files(cx, &files);
        cx.end_turtle_with_area(&mut self.area);

        DrawStep::done()
    }
}

impl WidgetNode for ModelFilesItems {
    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.area.redraw(cx)
    }

    fn find_widgets(&mut self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        for item in self.items.values_mut() {
            item.find_widgets(path, cached, results);
        }
    }
}

impl ModelFilesItems {
    fn draw_files(&mut self, cx: &mut Cx2d, files: &Vec<File>) {
        // TODO check if using proper ids in the items collections is better than having this mapping
        self.map_to_files.clear();

        for i in 0..files.len() {
            let template = if files[i].downloaded {
                self.template_downloaded
            } else {
                self.template_download
            };
            let item_id = LiveId(i as u64).into();
            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, template));
            self.map_to_files.insert(item_id, files[i].clone());

            let filename = &files[i].name;
            let size = format_model_size(&files[i].size).unwrap_or("-".to_string());
            let quantization = &files[i].quantization;
            item_widget.apply_over(
                cx,
                live! {
                    cell1 = {
                        filename = { text: (filename) }
                    }
                    cell2 = { full_size = { text: (size) }}
                    cell3 = {
                        quantization_tag = { quantization = { text: (quantization) }}
                     }
                },
            );

            if self.show_tags {
                item_widget
                    .model_files_tags(id!(tags))
                    .set_tags(cx, &files[i].tags);
            }

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
        }
    }
}

impl ModelFilesItemsRef {
    fn get_height(&mut self, cx: &mut Cx) -> f64 {
        let Some(inner) = self.borrow_mut() else {
            return 0.0;
        };
        inner.area.rect(cx).size.y
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesTags {
    #[redraw]
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template: Option<LivePtr>,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelFilesTags {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_id, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        for (_id, item) in self.items.iter_mut() {
            let _ = item.draw_walk(cx, scope, walk);
        }
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl ModelFilesTagsRef {
    pub fn set_tags(&self, cx: &mut Cx, tags: &Vec<String>) {
        let Some(mut tags_widget) = self.borrow_mut() else {
            return;
        };
        tags_widget.items.clear();
        for (i, tag) in tags.iter().enumerate() {
            let item_id = LiveId(i as u64).into();
            let item_widget = WidgetRef::new_from_ptr(cx, tags_widget.template);
            item_widget.apply_over(cx, live! {label = { text: (tag) }});
            tags_widget.items.insert(item_id, item_widget);
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
        let model = scope.data.get::<Model>().unwrap();
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
}
