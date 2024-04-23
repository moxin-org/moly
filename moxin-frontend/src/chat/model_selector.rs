use crate::{
    data::store::Store, my_models::downloaded_files_table::DownloadedFileAction,
    shared::utils::format_model_size,
};
use makepad_widgets::*;
use moxin_protocol::data::DownloadedFile;
use std::collections::HashMap;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            radius: 2.0,
        }

        caption = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #1D2939
            }
        }
    }

    ModelInfo = <View> {
        width: Fill,
        height: Fit,
        padding: 16,
        spacing: 10,
        align: {x: 0.0, y: 0.5},

        show_bg: true,
        draw_bg: {
            instance hover: 0.0,
            instance down: 0.0,
            color: #fff,
            instance color_hover: #F9FAFB,

            fn pixel(self) -> vec4 {
                return mix(self.color, self.color_hover, self.hover);
            }
        }

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
        }

        architecture_tag = <ModelAttributeTag> {
            draw_bg: {
                color: #DDD7FF,
            }
        }

        params_size_tag = <ModelAttributeTag> {
            draw_bg: {
                color: #D1F4FC,
            }
        }

        file_size_tag = <ModelAttributeTag> {
            caption = {
                draw_text:{
                    color: #000
                }
            }
            draw_bg: {
                color: #fff,
                border_color: #B4B4B4,
                border_width: 1.0,
            }
        }
    }

    ModelSelectorList = {{ModelSelectorList}} {
        flow: Down,
        template: <ModelInfo> {
            // This is mandatory to listen for touch/click events
            cursor: Hand,

            animator: {
                hover = {
                    default: off
                    off = {
                        from: {all: Forward {duration: 0.2}}
                        apply: {
                            draw_bg: {hover: 0.0}
                        }
                    }

                    on = {
                        from: {all: Snap}
                        apply: {
                            draw_bg: {hover: 1.0}
                        },
                    }
                }
                down = {
                    default: off
                    off = {
                        from: {all: Forward {duration: 0.5}}
                        ease: OutExp
                        apply: {
                            draw_bg: {down: 0.0}
                        }
                    }
                    on = {
                        ease: OutExp
                        from: {
                            all: Forward {duration: 0.2}
                        }
                        apply: {
                            draw_bg: {down: 1.0}
                        }
                    }
                }
            }
        }

        no_models_message: <View> {
            width: Fill,
            height: Fit,
            padding: 14,
            spacing: 5,
            align: {x: 0.5, y: 0.5},

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 11},
                    color: #000
                }
                text: "No models available. Download a model to get started."
            }
        }
    }

    ModelSelector = {{ModelSelector}} {
        width: Fill,
        height: Fit,

        flow: Down,

        button = <RoundedView> {
            width: Fill,
            height: 54,

            align: {x: 0.0, y: 0.5},
            padding: 16,

            draw_bg: {
                instance radius: 3.0,
                color: #F9FAFB,
                border_color: #DFDFDF,
                border_width: 1.0,
            }

            cursor: Hand,

            choose = <View> {
                width: Fill,
                height: Fit,

                align: {x: 0.5, y: 0.5},

                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 11},
                        color: #000
                    }
                    text: "Choose a model"
                }
            }
            selected = <ModelInfo> {
                width: Fit,
                height: Fit,
                show_bg: false,
                visible: false,

                label = {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 11},
                    }
                }
            }
        }

        options = <RoundedView> {
            width: Fill,
            height: Fit,
            visible: false

            margin: { top: 5 },
            padding: 5,

            draw_bg: {
                instance radius: 3.0,
                color: #fff,
                border_color: #B6B6B6,
                border_width: 1.0,
            }

            <ModelSelectorList> {
                width: Fill,
                height: Fit,
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelector {
    #[deref]
    view: View,

    #[rust]
    open: bool,
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(fd) = self.view(id!(button)).finger_down(&actions) {
            if fd.tap_count == 1 {
                self.open = !self.open;
                self.view(id!(options)).apply_over(
                    cx,
                    live! {
                        visible: (self.open)
                    },
                );
                self.redraw(cx);
            }
        }

        for action in actions {
            match action.as_widget_action().cast() {
                ModelSelectorAction::Selected(downloaded_file) => {
                    self.update_ui_with_file(cx, downloaded_file);
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                DownloadedFileAction::StartChat(downloaded_file) => {
                    self.update_ui_with_file(cx, downloaded_file);
                }
                _ => {}
            }
        }
    }
}

impl ModelSelector {
    fn update_ui_with_file(&mut self, cx: &mut Cx, downloaded_file: DownloadedFile) {
        self.open = false;
        self.view(id!(options)).apply_over(
            cx,
            live! {
                visible: (self.open)
            },
        );
        self.view(id!(choose)).apply_over(
            cx,
            live! {
                visible: false
            },
        );
        let filename = downloaded_file.file.name;

        let architecture = downloaded_file.model.architecture;
        let architecture_visible = !architecture.trim().is_empty();

        let param_size = downloaded_file.model.size;
        let param_size_visible = !param_size.trim().is_empty();

        let size = format_model_size(&downloaded_file.file.size).unwrap_or("".to_string());
        let size_visible = !size.trim().is_empty();

        self.view(id!(selected)).apply_over(
            cx,
            live! {
                visible: true
                label = { text: (filename) }
                architecture_tag = { visible: (architecture_visible), caption = { text: (architecture) }}
                params_size_tag = { visible: (param_size_visible), caption = { text: (param_size) }}
                file_size_tag = { visible: (size_visible), caption = { text: (size) }}
            },
        );
        self.redraw(cx);
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelSelectorAction {
    Selected(DownloadedFile),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorList {
    #[redraw]
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template: Option<LivePtr>,
    #[live]
    no_models_message: Option<LivePtr>,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,

    #[rust]
    map_to_downloaded_files: HashMap<LiveId, DownloadedFile>,
}

impl Widget for ModelSelectorList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        if let Event::Startup = event {
            let store = scope.data.get_mut::<Store>().unwrap();
            if let Some(filename) = &store.preferences.current_chat_model {
                if let Some(downloaded_file) = store
                    .downloaded_files
                    .iter()
                    .find(|file| &file.file.id == filename)
                {
                    cx.widget_action(
                        widget_uid,
                        &scope.path,
                        ModelSelectorAction::Selected(downloaded_file.to_owned()),
                    );
                }
            }
        }

        for (id, item) in self.items.iter_mut() {
            let actions = cx.capture_actions(|cx| item.handle_event(cx, event, scope));
            if let Some(fd) = item.as_view().finger_down(&actions) {
                if fd.tap_count == 1 {
                    cx.widget_action(
                        widget_uid,
                        &scope.path,
                        ModelSelectorAction::Selected(
                            self.map_to_downloaded_files.get(id).unwrap().clone(),
                        ),
                    );
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        cx.begin_turtle(walk, self.layout);

        if self.visible {
            self.draw_items(cx, &store.downloaded_files);
        }
        cx.end_turtle_with_area(&mut self.area);

        DrawStep::done()
    }
}

impl ModelSelectorList {
    fn draw_items(&mut self, cx: &mut Cx2d, items: &Vec<DownloadedFile>) {
        if items.is_empty() {
            let item_widget = WidgetRef::new_from_ptr(cx, self.no_models_message);
            let _ = item_widget.draw_all(cx, &mut Scope::empty());
            return;
        }

        self.map_to_downloaded_files = HashMap::new();
        for i in 0..items.len() {
            let item_id = LiveId(i as u64).into();
            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.template));
            self.map_to_downloaded_files
                .insert(item_id, items[i].clone());

            let caption = &items[i].file.name;

            let architecture = &items[i].model.architecture;
            let architecture_visible = !architecture.trim().is_empty();

            let param_size = &items[i].model.size;
            let param_size_visible = !param_size.trim().is_empty();

            let size = format_model_size(&items[i].file.size).unwrap_or("".to_string());
            let size_visible = !size.trim().is_empty();

            item_widget.apply_over(
                cx,
                live! {
                    label = { text: (caption) }
                    architecture_tag = { visible: (architecture_visible), caption = { text: (architecture) } }
                    params_size_tag = { visible: (param_size_visible), caption = { text: (param_size) } }
                    file_size_tag = { visible: (size_visible), caption = { text: (size) } }
                },
            );

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
        }
    }
}
