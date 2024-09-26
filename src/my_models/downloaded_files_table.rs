use makepad_widgets::*;
use moly_protocol::data::DownloadedFile;

use crate::data::store::Store;

use super::{
    downloaded_files_row::{DownloadedFilesRowProps, DownloadedFilesRowWidgetRefExt},
    my_models_screen::MyModelsSearchAction,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    import crate::my_models::downloaded_files_row::DownloadedFilesRow

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

    DownloadedFilesTable = {{DownloadedFilesTable}} <RoundedView> {
        width: Fill,
        height: Fill,
        align: {x: 0.5, y: 0.5}

        list = <PortalList> {
            drag_scrolling: false
            HeaderRow = <HeaderRow> {
                cursor: Default
            }
            ItemRow = <DownloadedFilesRow> {
                cursor: Default
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DownloadedFilesTable {
    #[deref]
    view: View,
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
                    != scope
                        .data
                        .get::<Store>()
                        .unwrap()
                        .downloads
                        .downloaded_files
                        .len()
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

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, last_item_id + 1);
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

                        item.as_downloaded_files_row()
                            .set_file_id(file_data.file.id.clone());

                        let props = DownloadedFilesRowProps {
                            downloaded_file: file_data.clone(),
                        };
                        let mut scope = Scope::with_props(&props);
                        item.draw_all(cx, &mut scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for DownloadedFilesTable {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for action in actions.iter() {
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
        self.current_results = scope
            .data
            .get::<Store>()
            .unwrap()
            .downloads
            .downloaded_files
            .clone();
        self.latest_store_fetch_len = self.current_results.len();
    }

    fn filter_by_keywords(&mut self, cx: &mut Cx, scope: &mut Scope, keywords: &str) {
        let keywords = keywords.to_lowercase();
        self.current_results = scope
            .data
            .get::<Store>()
            .unwrap()
            .downloads
            .downloaded_files
            .clone();
        self.latest_store_fetch_len = self.current_results.len();

        self.current_results.retain(|f| {
            f.file.name.to_lowercase().contains(&keywords)
                || f.model.name.to_lowercase().contains(&keywords)
        });

        self.search_status = SearchStatus::Filtered(keywords);
        self.redraw(cx);
    }

    fn reset_results(&mut self, cx: &mut Cx, scope: &mut Scope) {
        self.current_results = scope
            .data
            .get::<Store>()
            .unwrap()
            .downloads
            .downloaded_files
            .clone();

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
