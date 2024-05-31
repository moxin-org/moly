use crate::{
    data::store::ModelWithPendingDownloads,
    shared::utils::format_model_size,
};
use makepad_widgets::*;
use moxin_protocol::data::{File, FileID, Model, PendingDownload, PendingDownloadsStatus};

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
            model_featured_files(model)
        } else {
            model_other_files(model)
        };
        cx.begin_turtle(walk, self.layout);

        self.draw_files(cx, &files, pending_downloads, current_file_id);
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
        files: &Vec<File>,
        pending_downloads: &Vec<PendingDownload>,
        current_file_id: &Option<FileID>,
    ) {
        for i in 0..files.len() {
            let item_id = LiveId(i as u64).into();

            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.template));

            item_widget
                .as_model_files_item()
                .set_file(cx, files[i].clone());

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

            if let Some(download) = pending_downloads.iter().find(|f| f.file.id == files[i].id) {
                let progress = format!("{:.1}%", download.progress);
                let progress_fill_max = 74.0;
                let progress_fill = download.progress * progress_fill_max / 100.0;

                let is_resume_download_visible =
                    matches!(download.status, PendingDownloadsStatus::Paused);
                let is_pause_download_visible =
                    matches!(download.status, PendingDownloadsStatus::Downloading);
                let is_retry_download_visible =
                    matches!(download.status, PendingDownloadsStatus::Error);

                let status_color = match download.status {
                    PendingDownloadsStatus::Downloading => vec3(0.035, 0.572, 0.314), // #099250
                    PendingDownloadsStatus::Paused => vec3(0.4, 0.44, 0.52),          // #667085
                    PendingDownloadsStatus::Error => vec3(0.7, 0.11, 0.09),           // #B42318
                };

                item_widget.apply_over(
                    cx,
                    live! { cell4 = {
                        download_pending_controls = {
                            visible: true
                            progress_text_layout = {
                                progress_text = {
                                    text: (progress)
                                    draw_text: {
                                        color: (status_color)
                                    }
                                }
                            }
                            progress_bar = {
                                progress_fill = {
                                    width: (progress_fill)
                                    draw_bg: {
                                        color: (status_color),
                                    }
                                }
                            }
                            resume_download_button = {
                                visible: (is_resume_download_visible)
                            }
                            retry_download_button = {
                                visible: (is_retry_download_visible)
                            }
                            pause_download_button = {
                                visible: (is_pause_download_visible)
                            }
                        }
                        start_chat_button = { visible: false }
                        resume_chat_button = { visible: false }
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
                            download_pending_controls = { visible: false }
                            start_chat_button = { visible: false }
                            resume_chat_button = { visible: true }
                            download_button = { visible: false }
                        }},
                    );
                } else {
                    item_widget.apply_over(
                        cx,
                        live! { cell4 = {
                            download_pending_controls = { visible: false }
                            start_chat_button = { visible: true }
                            resume_chat_button = { visible: false }
                            download_button = { visible: false }
                        }},
                    );
                }
            } else {
                item_widget.apply_over(
                    cx,
                    live! { cell4 = {
                        download_pending_controls = { visible: false }
                        start_chat_button = { visible: false }
                        resume_chat_button = { visible: false }
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

fn model_featured_files(model: &Model) -> Vec<File> {
    model.files.iter().filter(|f| f.featured).cloned().collect()
}

fn model_other_files(model: &Model) -> Vec<File> {
    model
        .files
        .iter()
        .filter(|f| !f.featured)
        .cloned()
        .collect()
}
