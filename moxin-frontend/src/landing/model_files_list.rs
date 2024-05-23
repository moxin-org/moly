use crate::{
    data::store::{ModelWithPendingDownloads, Store},
    shared::utils::format_model_size,
};
use makepad_widgets::*;
use moxin_protocol::data::{File, FileID, Model, PendingDownload};

use super::model_files_item::ModelFilesItemWidgetRefExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::landing::model_files_item::ModelFilesItem;

    ModelFilesList = {{ModelFilesList}} {
        width: Fill,
        height: Fit,
        flow: Down,

        template: <ModelFilesItem> {}
    }
}

#[derive(Live, LiveHook, LiveRegisterWidget, WidgetRef)]
pub struct ModelFilesList {
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template: Option<LivePtr>,

    #[live(false)]
    show_featured: bool,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelFilesList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_id, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let ModelWithPendingDownloads {
            model,
            pending_downloads,
            current_file_id,
        } = scope.data.get::<ModelWithPendingDownloads>().unwrap();
        let files = if self.show_featured {
            Store::model_featured_files(model)
        } else {
            Store::model_other_files(model)
        };
        cx.begin_turtle(walk, self.layout);

        self.draw_files(cx, &model, &files, pending_downloads, current_file_id);
        cx.end_turtle_with_area(&mut self.area);

        DrawStep::done()
    }
}

impl WidgetNode for ModelFilesList {
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

impl ModelFilesList {
    fn draw_files(
        &mut self,
        cx: &mut Cx2d,
        model: &Model,
        files: &Vec<File>,
        pending_downloads: &Vec<PendingDownload>,
        current_file_id: &Option<FileID>,
    ) {
        for i in 0..files.len() {
            let item_id = LiveId(i as u64).into();

            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.template));

            item_widget.as_model_files_item().set_model_and_file(
                cx,
                model.clone(),
                files[i].clone(),
            );

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

            if pending_downloads
                .iter()
                .find(|f| f.file.id == files[i].id)
                .is_some()
            {
                item_widget.apply_over(
                    cx,
                    live! { cell4 = {
                        download_pending_button = { visible: true }
                        start_chat_button = { visible: false }
                        download_button = { visible: false }
                    }},
                );
            } else if files[i].downloaded {
                if current_file_id
                    .as_ref()
                    .map_or(false, |id| *id == files[i].id)
                {
                    item_widget.apply_over(
                        cx,
                        live! { cell4 = {
                            download_pending_button = { visible: false }
                            start_chat_button = { visible: true }
                            resume_chat_button = { visible: false }
                            download_button = { visible: false }
                        }},
                    );
                } else {
                    item_widget.apply_over(
                        cx,
                        live! { cell4 = {
                            download_pending_button = { visible: false }
                            start_chat_button = { visible: false }
                            resume_chat_button = { visible: true }
                            download_button = { visible: false }
                        }},
                    );
                }
            } else {
                item_widget.apply_over(
                    cx,
                    live! { cell4 = {
                        download_pending_button = { visible: false }
                        start_chat_button = { visible: false }
                        download_button = { visible: true }
                    }},
                );
            };

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
        }
    }
}

impl ModelFilesListRef {
    pub fn get_height(&mut self, cx: &mut Cx) -> f64 {
        let Some(inner) = self.borrow_mut() else {
            return 0.0;
        };
        inner.area.rect(cx).size.y
    }
}
