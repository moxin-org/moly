use std::collections::HashMap;

use makepad_widgets::*;
use moxin_protocol::data::DownloadedFile;

use crate::data::store::Store;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    ICON_PLAY = dep("crate://self/resources/icons/play_arrow.svg")
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

    ActionButton = {{ActionButton}} {
        align: {x: 0.5, y: 0.5}
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
        // Heads-up: the spacing and row header widths match the row values
        spacing: 30,
        show_bg: true
        draw_bg: {
            color: #F2F4F7;
        }

        <RowHeaderLabel> { width: 150, label = {text: "Model Name"} }
        <RowHeaderLabel> { width: 80, label = {text: "Parameters"} }
        <RowHeaderLabel> { width: 245, label = {text: "Model Version"} }
        <RowHeaderLabel> { width: 80, label = {text: "Quantization"} }
        <RowHeaderLabel> { width: 80, label = {text: "File Size"} }
        <RowHeaderLabel> { width: 130, label = {text: "Compatibility Guess"} }
        <RowHeaderLabel> { width: 80, label = {text: "Date Added"} }
        <RowHeaderLabel> { width: 80, label = {text: ""} }
    }

    Row = <View> {
        // Heads-up: rows break the Portal List without fixed height
        height: 55, 
        flow: Down
        width: Fill
        align: {x: 0.0, y: 0.5}

        show_bg: true
        draw_bg: {
            color: #FFF;
        }

        separator_line = <Line> {}
        wrapper = <View> {
            flow: Right
            width: Fit
            padding: {top: 10, bottom: 10, left: 20, right: 20}
            spacing: 30,

            show_bg: true
            draw_bg: {
                color: #FFF;
            }

            name_tag = <View> {
                width: 150
                align: {x: 0.0, y: 0.5}
                name = <Label> {
                    width: Fill
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }

            parameters_tag = <View> {
                width: 80
                align: {x: 0.0, y: 0.5}
                parameters = <AttributeTag> {
                    draw_bg: { color: #44899A },
                }
            }
            model_version_tag = <View> {
                width: 245
                align: {x: 0.0, y: 0.5}
                version = <Label> {
                    width: Fill
                    draw_text: {
                        wrap: Ellipsis
                        text_style: <REGULAR_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }
            quantization_tag = <View> {
                width: 80
                align: {x: 0.0, y: 0.5}
                quantization = <AttributeTag> {
                    draw_bg: {
                        color: #FFF,
                        border_color: #B4B4B4,
                        border_width: 1.0,
                        instance radius: 2.0,
                    }
                    attr_name = {
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 9}
                            color: #x0
                        }
                    }
                }
            }
            file_size_tag = <View> {
                width: 80
                align: {x: 0.0, y: 0.5}
                file_size = <Label> {
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }
            compatibility_guess_tag = <View> {
                width: 130
                align: {x: 0.0, y: 0.5}
                compatibility = <AttributeTag> {
                    draw_bg: { color: #E6F1EC },
                    attr_name = { draw_text: { color: #101828} }
                }
            }
            date_added_tag = <View> {
                width: 80
                align: {x: 0.0, y: 0.5}
                date_added = <Label> {
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }

            actions = <View> {
                width: 80
                flow: Right
                spacing: 10
                align: {x: 0.0, y: 0.5}

                <ActionButton> { type_: Play, icon = { draw_icon: { svg_file: (ICON_PLAY) } } }
                <ActionButton> { type_: Info, icon = { draw_icon: { svg_file: (ICON_INFO) fn get_color(self) -> vec4 { return #0099FF;} } } }
                <ActionButton> { type_: Delete, icon = { draw_icon: { svg_file: (ICON_DELETE) fn get_color(self) -> vec4 { return #B42318;} } } }
            }
        }
    }

    ModelsTable = {{ModelsTable}} <RoundedView> {
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
pub struct ModelsTable {
    #[deref]
    view: View,
    #[rust]
    model_item_map: HashMap<u64, String>,
}

impl Widget for ModelsTable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let widget_uid = self.widget_uid();
        for list_action in cx.capture_actions(|cx| self.view.handle_event(cx, event, scope)) {
            if let Some(action) = list_action.as_widget_action() {
                if let Some(group) = &action.group {
                    match list_action.as_widget_action().cast() {
                        RowAction::PlayClicked => {
                            if let Some(item_id) = self.model_item_map.get(&group.item_uid.0) {
                                let downloaded_files =
                                    &scope.data.get::<Store>().unwrap().downloaded_files;

                                // TODO: error handling
                                let file = downloaded_files
                                    .iter()
                                    .find(|f| f.file.id.eq(item_id))
                                    .unwrap();

                                cx.widget_action(
                                    widget_uid,
                                    &scope.path,
                                    ModelAction::StartChat(file.clone()),
                                );
                            }
                        }
                        RowAction::InfoClicked => {
                            let widget_action = list_action.as_widget_action().unwrap();

                            if let Some(_item_id) =
                                self.model_item_map.get(&widget_action.widget_uid.0)
                            {
                            }
                        }
                        RowAction::DeleteClicked => {
                            let widget_action = list_action.as_widget_action().unwrap();

                            if let Some(_item_id) =
                                self.model_item_map.get(&widget_action.widget_uid.0)
                            {
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut downloaded_files = scope.data.get::<Store>().unwrap().downloaded_files.clone();
        downloaded_files.sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));

        let entries_count = downloaded_files.len();
        let last_item_id = if entries_count > 0 { entries_count } else { 0 };

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

                        let model_data = &downloaded_files[item_id - 1];

                        self.model_item_map
                            .insert(item.widget_uid().0, model_data.file.id.clone());

                        // Name tag
                        let model_name = human_readable_name(&model_data.model.name);
                        item.label(id!(wrapper.name_tag.name))
                            .set_text(&model_name);

                        // Parameters tag
                        let parameters = dash_if_empty(&model_data.model.size);
                        item.label(id!(wrapper.parameters_tag.parameters.attr_name))
                            .set_text(&parameters);

                        // Version tag
                        // let filename = human_readable_name(&model_data.file.name);
                        let filename = &model_data.file.name.replace(".gguf", "").replace(".GGUF", "");
                        item.label(id!(wrapper.model_version_tag.version))
                        .set_text(&filename);

                        // Quantization tag
                        let quantization = dash_if_empty(&model_data.file.quantization);
                        item.label(id!(wrapper.quantization_tag.quantization.attr_name))
                            .set_text(quantization);

                        // File size tag
                        let file_size = dash_if_empty(&model_data.file.size);
                        item.label(id!(wrapper.file_size_tag.file_size))
                            .set_text(file_size);

                        // Compatibility guess tag
                        item.label(id!(wrapper.compatibility_guess_tag.compatibility.attr_name))
                            .set_text(model_data.compatibility_guess.as_str());

                        // Added date tag
                        let formatted_date =
                            model_data.downloaded_at.format("%d/%m/%Y").to_string();
                        item.label(id!(wrapper.date_added_tag.date_added))
                            .set_text(&formatted_date);

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

#[derive(Clone, DefaultNone, Debug)]
pub enum RowAction {
    PlayClicked,
    InfoClicked,
    DeleteClicked,
    None,
}

#[derive(Clone, Debug, Live, LiveHook)]
#[live_ignore]
pub enum ButtonType {
    #[pick]
    Play,
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

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelAction {
    StartChat(DownloadedFile),
    None,
}

/// Removes dashes, file extension, and capitalizes the first letter of each word.
fn human_readable_name(name: &str) -> String {
    let name = name.to_lowercase()
        .replace("-", " ")
        .replace("gguf", "")
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