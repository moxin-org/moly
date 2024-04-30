use std::collections::HashMap;

use makepad_widgets::*;
use moxin_protocol::data::{DownloadedFile, FileID};

use crate::{
    data::store::Store,
    shared::{modal::ModalAction, utils::format_model_size},
};

use super::{
    delete_model_modal::DeleteModelAction, model_info_modal::ModelInfoAction,
    my_models_screen::MyModelsSearchAction,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    ICON_START_CHAT = dep("crate://self/resources/icons/start_chat.svg")
    ICON_GO_TO = dep("crate://self/resources/icons/go_to.svg")
    ICON_INFO = dep("crate://self/resources/icons/info.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")

    RowHeaderLabel = <View> {
        width: 100
        height: Fit
        align: {x: 0.0, y: 0.5}
        label = <Label> {
            width: Fit
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 9}
                color: #667085
            }
        }
    }

    ActionButton = {{ActionButton}}<RoundedView> {
        align: {x: 0.5, y: 0.5}
        flow: Right
        width: Fit, height: Fit
        padding: { top: 15, bottom: 15, left: 8, right: 13}
        spacing: 10
        draw_bg: {
            radius: 2.0,
            color: #fff,
            border_width: 1.0,
            border_color: #ccc,
        }

        icon = <Icon> {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    return #087443;
                }
            }
            icon_walk: {width: 12, height: 12}
        }
    }

    HeaderRow = <View> {
        align: {x: 0.0, y: 0.5}
        width: Fill
        height: Fit
        padding: {top: 10, bottom: 10, left: 20, right: 20}
        // Heads-up: the spacing and row header widths need to match the row values
        spacing: 30,
        show_bg: true
        draw_bg: {
            color: #F2F4F7;
        }

        <RowHeaderLabel> { width: 600, label = {text: "Model File"} }
        <RowHeaderLabel> { width: 100, label = {text: "File Size"} }
        <RowHeaderLabel> { width: 100, label = {text: "Added Date"} }
        <RowHeaderLabel> { width: 250, label = {text: ""} }
    }

    Row = <View> {
        // Heads-up: rows break the Portal List without fixed height
        height: 85,
        flow: Down
        width: Fill
        align: {x: 0.0, y: 0.5}

        show_bg: true
        draw_bg: {
            color: #FFF;
        }

        separator_line = <Line> {}
        h_wrapper = <View> {
            flow: Right
            width: Fit
            padding: {top: 10, bottom: 10, left: 20, right: 20}
            spacing: 30
            show_bg: true
            draw_bg: {
                color: #FFF;
            }

            model_file = <View> {
                flow: Down
                width: 600

                h_wrapper = <View> {
                    flow: Right
                    width: Fill
                    spacing: 15
                    name_tag = <View> {
                        width: Fit
                        align: {x: 0.0, y: 0.5}
                        name = <Label> {
                            width: Fit
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 9}
                                color: #x0
                            }
                        }
                    }

                    base_model_tag = <View> {
                        width: Fit
                        align: {x: 0.0, y: 0.5}
                        base_model = <AttributeTag> {
                            draw_bg: { color: #F9E1FF },
                        }
                    }
                    parameters_tag = <View> {
                        width: Fit
                        align: {x: 0.0, y: 0.5}
                        parameters = <AttributeTag> {
                            draw_bg: { color: #44899A },
                        }
                    }
                }
                model_version_tag = <View> {
                    width: Fit
                    align: {x: 0.0, y: 0.5}
                    version = <Label> {
                        width: Fit
                        draw_text: {
                            wrap: Ellipsis
                            text_style: <REGULAR_FONT>{font_size: 9}
                            color: #667085
                        }
                    }
                }
            }

            file_size_tag = <View> {
                width: 100
                align: {x: 0.0, y: 0.5}
                file_size = <Label> {
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }

            date_added_tag = <View> {
                width: 100
                align: {x: 0.0, y: 0.5}
                date_added = <Label> {
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }

            actions = <View> {
                width: 250
                flow: Right
                spacing: 10
                align: {x: 0.0, y: 0.5}

                start_chat = <ActionButton> {
                    width: 140
                    type_: Play,
                    label = <Label> {
                        text: "Chat with Model",
                        draw_text: {
                            color: #087443
                            text_style: <REGULAR_FONT>{font_size: 9}
                        }
                    }
                    icon = { draw_icon: { svg_file: (ICON_START_CHAT) } }
                }

                resume_chat = <ActionButton> {
                    width: 140
                    visible: false
                    type_: Resume,
                    show_bg: true
                    draw_bg: {
                        color: #087443
                    }
                    label = <Label> {
                        text: "Resume Chat",
                        draw_text: {
                            color: #fff
                            text_style: <BOLD_FONT>{font_size: 9}
                        }
                    }
                    icon = {
                        draw_icon: { svg_file: (ICON_GO_TO) fn get_color(self) -> vec4 { return #fff;} }
                        icon_walk: {width: 10, height: 10}
                    }
                }

                <View> { width: Fill, height: Fit }

                <ActionButton> {
                    type_: Info,
                    icon = { draw_icon: { svg_file: (ICON_INFO) fn get_color(self) -> vec4 { return #0099FF;} } }
                }

                <ActionButton> {
                    type_: Delete,
                    icon = { draw_icon: { svg_file: (ICON_DELETE) fn get_color(self) -> vec4 { return #B42318;} } }
                }
            }
        }
    }

    DownloadedFilesTable = {{DownloadedFilesTable}} <RoundedView> {
        width: Fill,
        height: Fill,
        align: {x: 0.5, y: 0.5}

        list = <PortalList>{
            drag_scrolling: false
            HeaderRow = <HeaderRow> {}
            ItemRow = <Row> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DownloadedFilesTable {
    #[deref]
    view: View,
    #[rust]
    file_item_map: HashMap<u64, String>,
    #[rust]
    current_results: Vec<DownloadedFile>,
    #[rust]
    latest_store_fetch_len: usize,
    #[rust]
    search_status: SearchStatus,
}

impl Widget for DownloadedFilesTable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let filter = match &self.search_status {
            SearchStatus::Filtered(keywords) => Some(keywords.clone()),
            _ => None,
        };

        // If we're already filtering, re-apply filter over the new store data
        // only re-filtering if there are new downloads in store
        match filter {
            Some(keywords) => {
                if self.latest_store_fetch_len
                    != scope.data.get::<Store>().unwrap().downloaded_files.len()
                {
                    self.filter_by_keywords(cx, scope, &keywords)
                }
            }
            None => self.fetch_results(scope),
        };

        self.current_results
            .sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));

        let entries_count = self.current_results.len();
        let last_item_id = if entries_count > 0 { entries_count } else { 0 };

        let mut current_chat_file_id = None;
        if let Some(current_chat) = &scope.data.get::<Store>().unwrap().current_chat {
            current_chat_file_id = Some(current_chat.file_id.clone());
        }

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, last_item_id);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id <= last_item_id {
                        let template;
                        if item_id == 0 {
                            // Draw header row
                            template = live_id!(HeaderRow);
                            let item = list.item(cx, item_id, template).unwrap();
                            item.draw_all(cx, scope);
                            continue;
                        }

                        template = live_id!(ItemRow);
                        let item = list.item(cx, item_id, template).unwrap();

                        let file_data = &self.current_results[item_id - 1];

                        self.file_item_map
                            .insert(item.widget_uid().0, file_data.file.id.clone());

                        // Name tag
                        let name = human_readable_name(&file_data.file.name);
                        item.label(id!(h_wrapper.model_file.h_wrapper.name_tag.name))
                            .set_text(&name);

                        // Base model tag
                        let base_model = dash_if_empty(&file_data.model.architecture);
                        item.label(id!(h_wrapper
                            .model_file
                            .base_model_tag
                            .base_model
                            .attr_name))
                            .set_text(&base_model);

                        // Parameters tag
                        let parameters = dash_if_empty(&file_data.model.size);
                        item.label(id!(h_wrapper
                            .model_file
                            .parameters_tag
                            .parameters
                            .attr_name))
                            .set_text(&parameters);

                        // Version tag
                        let filename = format!("{}/{}", file_data.model.name, file_data.file.name);
                        item.label(id!(h_wrapper.model_file.model_version_tag.version))
                            .set_text(&filename);

                        // File size tag
                        let file_size =
                            format_model_size(&file_data.file.size).unwrap_or("-".to_string());
                        item.label(id!(h_wrapper.file_size_tag.file_size))
                            .set_text(&file_size);

                        // Added date tag
                        let formatted_date = file_data.downloaded_at.format("%d/%m/%Y").to_string();
                        item.label(id!(h_wrapper.date_added_tag.date_added))
                            .set_text(&formatted_date);

                        // Wether to show show a start chat or resume chat button
                        let mut start_chat_button =
                            item.action_button(id!(h_wrapper.actions.start_chat));
                        let mut resume_chat_button =
                            item.action_button(id!(h_wrapper.actions.resume_chat));
                        let mut show_resume_button = false;

                        if let Some(file_id) = &current_chat_file_id {
                            if *file_id == file_data.file.id {
                                show_resume_button = true;
                            }
                        }

                        if show_resume_button {
                            resume_chat_button.set_visible(true);
                            start_chat_button.set_visible(false);
                        } else {
                            start_chat_button.set_visible(true);
                            resume_chat_button.set_visible(false);
                        }

                        // Don't draw separator line on first row
                        if item_id == 1 {
                            item.view(id!(separator_line)).set_visible(false);
                        }

                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for DownloadedFilesTable {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        for action in actions.iter() {
            if let Some(action) = action.as_widget_action() {
                if let Some(group) = &action.group {
                    match action.cast() {
                        RowAction::PlayClicked => {
                            if let Some(item_id) = self.file_item_map.get(&group.item_uid.0) {
                                let store = scope.data.get::<Store>().unwrap();
                                let downloaded_files = &store.downloaded_files;
                                let downloaded_file =
                                    downloaded_files.iter().find(|f| f.file.id.eq(item_id));

                                if let Some(df) = downloaded_file {
                                    cx.widget_action(
                                        widget_uid,
                                        &scope.path,
                                        DownloadedFileAction::StartChat(df.file.id.clone()),
                                    );
                                } else {
                                    error!("A play action was dispatched for a model that does not longer exist in the local store");
                                }
                            }
                        }
                        RowAction::ResumeClicked => {
                            if let Some(item_id) = self.file_item_map.get(&group.item_uid.0) {
                                let store = scope.data.get::<Store>().unwrap();
                                let downloaded_files = &store.downloaded_files;

                                let downloaded_file =
                                    downloaded_files.iter().find(|f| f.file.id.eq(item_id));

                                if let Some(df) = downloaded_file {
                                    cx.widget_action(
                                        widget_uid,
                                        &scope.path,
                                        DownloadedFileAction::ResumeChat(df.file.id.clone()),
                                    );
                                } else {
                                    error!("A play action was dispatched for a model that does not longer exist in the local store");
                                }
                            }
                        }
                        RowAction::InfoClicked => {
                            if let Some(item_id) = self.file_item_map.get(&group.item_uid.0) {
                                cx.widget_action(
                                    widget_uid,
                                    &scope.path,
                                    ModelInfoAction::FileSelected(item_id.clone()),
                                );
                                cx.widget_action(
                                    widget_uid,
                                    &scope.path,
                                    ModalAction::ShowModalView(live_id!(model_info_modal_view)),
                                );
                            }
                        }
                        RowAction::DeleteClicked => {
                            if let Some(item_id) = self.file_item_map.get(&group.item_uid.0) {
                                cx.widget_action(
                                    widget_uid,
                                    &scope.path,
                                    DeleteModelAction::FileSelected(item_id.clone()),
                                );
                                cx.widget_action(
                                    widget_uid,
                                    &scope.path,
                                    ModalAction::ShowModalView(live_id!(delete_model_modal_view)),
                                );
                            }
                        }
                        _ => (),
                    }
                }
            }

            match action.cast() {
                MyModelsSearchAction::Search(keywords) => {
                    self.filter_by_keywords(cx, scope, &keywords);
                }
                MyModelsSearchAction::Reset => {
                    self.reset_results(cx, scope);
                }
                _ => {}
            }
        }
    }
}

impl DownloadedFilesTable {
    fn fetch_results(&mut self, scope: &mut Scope) {
        self.current_results = scope.data.get::<Store>().unwrap().downloaded_files.clone();
        self.latest_store_fetch_len = self.current_results.len();
    }

    fn filter_by_keywords(&mut self, cx: &mut Cx, scope: &mut Scope, keywords: &str) {
        let keywords = keywords.to_lowercase();
        self.current_results = scope.data.get::<Store>().unwrap().downloaded_files.clone();
        self.latest_store_fetch_len = self.current_results.len();

        self.current_results.retain(|f| {
            f.file.name.to_lowercase().contains(&keywords)
                || f.model.name.to_lowercase().contains(&keywords)
        });

        self.search_status = SearchStatus::Filtered(keywords);
        self.redraw(cx);
    }

    fn reset_results(&mut self, cx: &mut Cx, scope: &mut Scope) {
        self.current_results = scope.data.get::<Store>().unwrap().downloaded_files.clone();

        self.search_status = SearchStatus::Idle;
        self.redraw(cx);
    }
}

#[derive(Clone, Default, Debug)]
enum SearchStatus {
    #[default]
    Idle,
    Filtered(String),
}

#[derive(Clone, DefaultNone, Debug)]
pub enum RowAction {
    PlayClicked,
    InfoClicked,
    ResumeClicked,
    DeleteClicked,
    None,
}

#[derive(Clone, Debug, Live, LiveHook)]
#[live_ignore]
pub enum ButtonType {
    #[pick]
    Play,
    Resume,
    Info,
    Delete,
}

#[derive(Live, LiveHook, Widget)]
pub struct ActionButton {
    #[deref]
    view: View,
    #[live]
    type_: ButtonType,
}

impl Widget for ActionButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let uid = self.widget_uid().clone();
        match event.hits(cx, self.view.area()) {
            Hit::FingerDown(_) => {
                cx.set_key_focus(self.view.area());
            }
            Hit::FingerUp(fe) => {
                if fe.was_tap() {
                    let action = match self.type_ {
                        ButtonType::Play => RowAction::PlayClicked,
                        ButtonType::Resume => RowAction::ResumeClicked,
                        ButtonType::Info => RowAction::InfoClicked,
                        ButtonType::Delete => RowAction::DeleteClicked,
                    };
                    cx.widget_action(uid, &scope.path, action);
                }
            }
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
            }
            Hit::FingerHoverOut(_) => {
                cx.set_cursor(MouseCursor::Arrow);
            }
            _ => (),
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ActionButtonRef {
    pub fn set_visible(&mut self, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.visible = visible;
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadedFileAction {
    StartChat(FileID),
    ResumeChat(FileID),
    None,
}

/// Removes dashes, file extension, and capitalizes the first letter of each word.
fn human_readable_name(name: &str) -> String {
    let name = name
        .to_lowercase()
        .replace("-", " ")
        .replace(".gguf", "")
        .replace("chat", "");

    let name = name
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    name
}

fn dash_if_empty(input: &str) -> &str {
    if input.is_empty() {
        "-"
    } else {
        input
    }
}
