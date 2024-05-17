use crate::{
    data::store::{ModelWithPendingDownloads, Store},
    shared::{modal::ModalAction, utils::format_model_size},
};
use makepad_widgets::*;
use moxin_protocol::data::{File, FileID, Model, PendingDownload};

use super::{model_files_item::ModelFilesItemWidgetRefExt, model_files_tags::ModelFilesTagsWidgetRefExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::landing::shared::*;

    import crate::landing::model_files_item::ModelFilesItem;

    ModelFilesList = {{ModelFilesList}} {
        width: Fill,
        height: Fit,
        flow: Down,

        template: <ModelFilesItem> {}
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelFilesListAction {
    Downloaded(FileID),
    None,
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

    #[live(true)]
    show_tags: bool,

    #[live(false)]
    show_featured: bool,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelFilesList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let widget_uid = self.widget_uid();
        let store: &mut Store = scope.data.get_mut::<Store>().unwrap();

        // Notify of a downloaded file.
        if let Event::Signal = event {
            if let Some(downloaded_file) = store.downloaded_files_to_notify.pop_front() {
                    cx.widget_action(
                        widget_uid,
                        &scope.path,
                        ModalAction::ShowModalView(live_id!(popup_download_success_modal_view)),
                    );
                    cx.widget_action(
                        widget_uid,
                        &scope.path,
                        ModelFilesListAction::Downloaded(downloaded_file.clone()),
                    );
            }
        }

        for (_id, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let ModelWithPendingDownloads {
            model,
            pending_downloads,
        } = scope.data.get::<ModelWithPendingDownloads>().unwrap();
        let files = if self.show_featured {
            Store::model_featured_files(model)
        } else {
            Store::model_other_files(model)
        };
        cx.begin_turtle(walk, self.layout);

        self.draw_files(cx, &model, &files, pending_downloads);
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
    ) {
        for i in 0..files.len() {
            let item_id = LiveId(i as u64).into();

            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.template));

            item_widget.as_model_files_item().set_model_and_file(model.clone(), files[i].clone());

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
                        downloaded_button = { visible: false }
                        download_button = { visible: false }
                    }},
                );
            } else if files[i].downloaded {
                item_widget.apply_over(
                    cx,
                    live! { cell4 = {
                        download_pending_button = { visible: false }
                        downloaded_button = { visible: true }
                        download_button = { visible: false }
                    }},
                );
            } else {
                item_widget.apply_over(
                    cx,
                    live! { cell4 = {
                        download_pending_button = { visible: false }
                        downloaded_button = { visible: false }
                        download_button = { visible: true }
                    }},
                );
            };

            if self.show_tags {
                item_widget
                    .model_files_tags(id!(tags))
                    .set_tags(cx, &files[i].tags);
            }

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