use crate::data::store::{FileWithDownloadInfo, ModelWithDownloadInfo};
use makepad_widgets::*;

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
        let model = &scope.data.get::<ModelWithDownloadInfo>().unwrap();
        let files = if self.show_featured {
            model_featured_files(model)
        } else {
            model_other_files(model)
        };
        cx.begin_turtle(walk, self.layout);

        self.draw_files(cx, &files);
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

    fn find_widgets(&self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        for item in self.items.values() {
            item.find_widgets(path, cached, results);
        }
    }

    fn uid_to_widget(&self, ui: WidgetUid) -> WidgetRef {
        self.items.values()
            .map(|item| item.uid_to_widget(ui))
            .find(|x| !x.is_empty())
            .unwrap_or(WidgetRef::empty())
    }
}

impl ModelFilesList {
    fn draw_files(&mut self, cx: &mut Cx2d, files_info: &Vec<FileWithDownloadInfo>) {
        for i in 0..files_info.len() {
            let item_id = LiveId(i as u64).into();

            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.template));

            item_widget
                .as_model_files_item()
                .set_file(cx, files_info[i].file.clone());

            let mut scope = Scope::with_props(&files_info[i]);
            let _ = item_widget.draw_all(cx, &mut scope);
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

fn model_featured_files(model: &ModelWithDownloadInfo) -> Vec<FileWithDownloadInfo> {
    model
        .files
        .iter()
        .filter(|f| f.file.featured)
        .cloned()
        .collect()
}

fn model_other_files(model: &ModelWithDownloadInfo) -> Vec<FileWithDownloadInfo> {
    model
        .files
        .iter()
        .filter(|f| !f.file.featured)
        .cloned()
        .collect()
}
