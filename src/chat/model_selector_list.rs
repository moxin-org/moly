use crate::{
    data::{chats::chat::ChatID, providers::ProviderBot, store::Store},
    shared::utils::format_model_size,
};
use makepad_widgets::*;
use moly_protocol::data::DownloadedFile;
use std::collections::HashMap;

use super::model_selector_item::ModelSelectorItemWidgetRefExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::model_info::ModelInfo;
    use crate::chat::model_info::AgentInfo;
    use crate::chat::model_selector_item::ModelSelectorItem;

    pub ModelSelectorList = {{ModelSelectorList}} {
        flow: Down,
        model_template: <ModelSelectorItem> {
            content = <ModelInfo> {
                padding: {left: 24, right: 10, top: 12, bottom: 12}
            }
        }

        section_label_template: <View> {
            width: Fit, height: Fit,
            padding: {left: 6, top: 4., bottom: 4.}
            spacing: 8
            align: {x: 0.0, y: 0.5},
            image_view = <View> {
                width: Fit, height: Fit
                visible: false
                image = <Image> {
                    width: 22, height: 22
                    source: dep("crate://self/resources/images/globe_icon.png")
                }
            }
            provider_initial_view = <RoundedView> {
                width: Fit, height: Fit
                padding: {left: 6, right: 6, top: 3, bottom: 3}
                visible: false
                show_bg: true
                draw_bg: {
                    color: #37567d,
                    border_radius: 3.0
                }
                provider_initial_label = <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 10.0},
                        color: #f
                    }
                }
            }

            label = <Label> {
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 11.0},
                    color: #989898
                }
            }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelSelectorListAction {
    AddedOrDeletedModel,
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
    model_template: Option<LivePtr>,

    #[live]
    section_label_template: Option<LivePtr>,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,

    #[rust]
    map_to_downloaded_files: HashMap<LiveId, DownloadedFile>,

    #[rust]
    total_height: Option<f64>,

    #[rust]
    chat_id: ChatID,

    #[rust]
    search_filter: String,
}

impl Widget for ModelSelectorList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope)
        }
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        cx.begin_turtle(walk, self.layout);

        if self.visible {
            self.draw_items(cx, &store);
        }

        cx.end_turtle_with_area(&mut self.area);

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ModelSelectorList {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions.iter() {
            if let ModelSelectorListAction::AddedOrDeletedModel = action.cast() {
                self.items.clear();
                self.total_height = None;
                self.redraw(cx);
            }
        }
    }
}

impl ModelSelectorList {
    fn draw_items(&mut self, cx: &mut Cx2d, store: &Store) {
        let mut models = store.downloads.downloaded_files.clone();
        // Sort local models consistently by name
        models.sort_by(|a, b| a.file.name.cmp(&b.file.name));

        self.map_to_downloaded_files = HashMap::new();
        let mut total_height = 0.0;

        let current_bot_id = store
            .chats
            .get_chat_by_id(self.chat_id)
            .and_then(|c| c.borrow().associated_bot.clone());

        // Get non-agent models
        let all_bots = store.chats.get_all_bots(true);

        // Group models by provider
        let mut models_by_provider: HashMap<String, (String, String, Vec<ProviderBot>)> =
            HashMap::new();
        for model in all_bots.iter() {
            // Get provider from the providers map using URL
            let provider = store.chats.providers.get(&model.provider_id);
            let provider_name = provider
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown Provider".to_string());
            let provider_id = provider
                .as_ref()
                .map(|p| p.id.clone())
                .unwrap_or_else(|| model.provider_id.clone());

            models_by_provider
                .entry(provider_id.clone())
                .or_insert_with(|| (provider_id, provider_name, Vec::new()))
                .2
                .push(model.clone());
        }

        let mut providers: Vec<(String, String, Vec<ProviderBot>)> = models_by_provider
            .into_iter()
            .map(|(_, (id, name, models))| (id, name, models))
            .collect();

        // Sort providers alphabetically by name
        providers.sort_by(|a, b| a.1.cmp(&b.1));

        let terms = self
            .search_filter
            .split_whitespace()
            .map(|s| s.to_ascii_lowercase())
            .collect::<Vec<_>>();

        // Add models grouped by provider
        for (provider_id, provider_name, mut provider_bots) in providers {
            if provider_bots.is_empty() {
                continue;
            }

            // Sort models within each provider by name for consistent ordering
            provider_bots.sort_by(|a, b| a.name.cmp(&b.name));

            let provider_bots: Vec<ProviderBot> = provider_bots
                .into_iter()
                .filter(|bot| {
                    if terms.is_empty() {
                        true
                    } else {
                        let name = bot.human_readable_name().to_ascii_lowercase();
                        let id = bot.name.to_ascii_lowercase();
                        terms.iter().all(|t| name.contains(t) || id.contains(t))
                    }
                })
                .collect();

            if provider_bots.is_empty() {
                continue;
            }

            // Add provider section label
            let section_id = LiveId::from_str(&provider_id);

            let section_label = self.items.get_or_insert(cx, section_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.section_label_template)
            });
            section_label
                .label(ids!(label))
                .set_text(cx, &provider_name);

            let provider_icon = store.get_provider_icon(&provider_name);
            if let Some(provider_icon) = provider_icon {
                section_label
                    .view(ids!(provider_initial_view))
                    .set_visible(cx, false);
                section_label.view(ids!(image_view)).set_visible(cx, true);
                let _ = section_label
                    .image(ids!(image))
                    .load_image_dep_by_path(cx, provider_icon.as_str());
            } else {
                section_label.view(ids!(image_view)).set_visible(cx, false);
                section_label
                    .view(ids!(provider_initial_view))
                    .set_visible(cx, true);
                section_label
                    .label(ids!(provider_initial_label))
                    .set_text(cx, &provider_name.chars().next().unwrap().to_string());
            }
            let _ = section_label.draw_all(cx, &mut Scope::empty());
            total_height += section_label.as_label().area().rect(cx).size.y;

            // Add models for this provider
            for provider_bot in provider_bots.iter() {
                let bot_item_id =
                    LiveId::from_str(format!("{}{}", provider_id, provider_bot.id).as_str());

                let item_widget = self.items.get_or_insert(cx, bot_item_id, |cx| {
                    WidgetRef::new_from_ptr(cx, self.model_template)
                });

                let mut caption = provider_bot.human_readable_name();

                let icon_tick_visible = match &current_bot_id {
                    Some(bot_id) => bot_id == &provider_bot.id,
                    _ => false,
                };

                // If the model is a local model get the different tag values
                let mut architecture = None;
                let mut param_size = None;
                let mut size = None;

                if let Some(downloaded_file) = store
                    .downloads
                    .downloaded_files
                    .iter()
                    .find(|f| f.file.id == provider_bot.name)
                {
                    // Only set the value as some if the string isn't empty
                    architecture = if !downloaded_file.model.architecture.is_empty() {
                        Some(downloaded_file.model.architecture.clone())
                    } else {
                        None
                    };
                    param_size = if !downloaded_file.model.size.is_empty() {
                        Some(downloaded_file.model.size.clone())
                    } else {
                        None
                    };
                    size = Some(
                        format_model_size(&downloaded_file.file.size).unwrap_or("".to_string()),
                    );

                    // Override the caption with the local file name
                    caption = &downloaded_file.file.name;
                }

                item_widget.apply_over(
                    cx,
                    live! {
                        content = {
                            label = { text: (caption) }
                            architecture_tag = { visible: (architecture.is_some()), caption = { text: (architecture.unwrap_or("".to_string())) } }
                            params_size_tag = { visible: (param_size.is_some()), caption = { text: (param_size.unwrap_or("".to_string())) } }
                            file_size_tag = { visible: (size.is_some()), caption = { text: (size.unwrap_or("".to_string())) } }
                            icon_tick_tag = { visible: (icon_tick_visible) }
                        }
                    },
                );

                item_widget
                    .as_model_selector_item()
                    .set_bot(provider_bot.clone());

                item_widget
                    .as_model_selector_item()
                    .set_chat_id(self.chat_id);

                let _ = item_widget.draw_all(cx, &mut Scope::empty());
                total_height += item_widget.view(ids!(content)).area().rect(cx).size.y;
            }
        }

        self.total_height = Some(total_height);
    }
}

impl ModelSelectorListRef {
    pub fn get_height(&self) -> f64 {
        let Some(inner) = self.borrow_mut() else {
            return 0.0;
        };
        inner.total_height.unwrap_or(0.0)
    }

    pub fn set_chat_id(&mut self, chat_id: ChatID) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.chat_id = chat_id;
    }

    pub fn set_search_filter(&mut self, cx: &mut Cx, filter: &str) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.search_filter = filter.to_string();
        inner.items.clear();
        inner.total_height = None;
        inner.redraw(cx);
    }

    pub fn clear_search_filter(&mut self, cx: &mut Cx) {
        self.set_search_filter(cx, "");
    }
}
