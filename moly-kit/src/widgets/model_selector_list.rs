use super::model_selector_item::ModelSelectorItemWidgetRefExt;
use crate::{
    GroupingFn,
    controllers::chat::ChatController,
    protocol::{Bot, BotId, Picture},
};
use makepad_widgets::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for filtering which bots to show in the model selector
pub trait BotFilter {
    fn should_show(&self, bot: &Bot) -> bool;
}

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::widgets::model_selector_item::ModelSelectorItem;

    pub ModelSelectorList = {{ModelSelectorList}} {
        width: Fill, height: Fit
        flow: Down,

        item_template: <ModelSelectorItem> {}

        section_label_template: <View> {
            width: Fill, height: Fit
            padding: {left: 16, top: 12, bottom: 6}
            align: {x: 0.0, y: 0.5}
            spacing: 8

            icon_view = <View> {
                width: Fit, height: Fit
                visible: false
                icon_image = <Image> {
                    width: 25, height: 25
                }
            }

            icon_fallback_view = <RoundedView> {
                width: 25, height: 25
                visible: false
                show_bg: true
                draw_bg: {
                    color: #344054
                    border_radius: 6.0
                }
                align: {x: 0.5, y: 0.5}

                icon_fallback_label = <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_BOLD>{font_size: 12.0},
                        color: #fff
                    }
                }
            }

            label = <Label> {
                draw_text: {
                    text_style: <THEME_FONT_BOLD>{font_size: 10.0},
                    color: #989898
                }
            }
        }
    }
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
    item_template: Option<LivePtr>,

    #[live]
    section_label_template: Option<LivePtr>,

    #[rust]
    pub items: ComponentMap<LiveId, WidgetRef>,

    #[rust]
    pub search_filter: String,

    #[rust]
    pub total_height: Option<f64>,

    #[rust]
    pub chat_controller: Option<Arc<Mutex<ChatController>>>,

    #[rust]
    pub grouping: Option<GroupingFn>,

    #[rust]
    pub filter: Option<Box<dyn BotFilter>>,

    #[rust]
    pub selected_bot_id: Option<BotId>,
}

impl Widget for ModelSelectorList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);

        // Get bots from chat controller
        let bots = if let Some(chat_controller) = &self.chat_controller {
            chat_controller.lock().unwrap().state().bots.clone()
        } else {
            Vec::new()
        };

        self.draw_items(cx, &bots);

        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl ModelSelectorList {
    fn draw_items(&mut self, cx: &mut Cx2d, bots: &[Bot]) {
        let mut total_height = 0.0;

        // Default grouping function: group by provider from bot ID
        let default_grouping: GroupingFn = Arc::new(|bot: &Bot| {
            let provider = bot.id.provider();
            (
                provider.to_string(),
                provider.to_string(),
                Some(bot.avatar.clone()),
            )
        });

        let grouping_fn = self.grouping.as_ref().unwrap_or(&default_grouping);

        // Filter bots based on search
        let terms = self
            .search_filter
            .split_whitespace()
            .map(|s| s.to_ascii_lowercase())
            .collect::<Vec<_>>();

        let filtered_bots: Vec<&Bot> = bots
            .iter()
            .filter(|bot| {
                // Filter by search terms
                let matches_search = if terms.is_empty() {
                    true
                } else {
                    let name = bot.name.to_ascii_lowercase();
                    let id = bot.id.as_str().to_ascii_lowercase();
                    terms.iter().all(|t| name.contains(t) || id.contains(t))
                };

                // Filter by custom filter function (if provided)
                let passes_filter = self.filter.as_ref().map_or(true, |f| f.should_show(bot));

                matches_search && passes_filter
            })
            .collect();

        // Group bots by their group ID
        let mut groups: HashMap<String, ((String, Option<Picture>), Vec<&Bot>)> = HashMap::new();
        for bot in filtered_bots {
            let (group_id, group_label, group_icon) = grouping_fn(bot);
            groups
                .entry(group_id)
                .or_insert_with(|| ((group_label, group_icon), Vec::new()))
                .1
                .push(bot);
        }

        // Sort groups alphabetically by group ID
        let mut group_list: Vec<_> = groups.into_iter().collect();
        group_list.sort_by(|(a_id, _), (b_id, _)| a_id.cmp(b_id));

        for (group_id, ((group_label, group_icon), mut group_bots)) in group_list {
            // Render section header
            let section_id = LiveId::from_str(&format!("section_{}", group_id));
            let section_label = self.items.get_or_insert(cx, section_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.section_label_template)
            });

            section_label.label(ids!(label)).set_text(cx, &group_label);

            // Display icon if available, otherwise show fallback (first letter)
            if let Some(icon) = &group_icon {
                match icon {
                    Picture::Dependency(dep) => {
                        section_label
                            .view(ids!(icon_fallback_view))
                            .set_visible(cx, false);
                        section_label.view(ids!(icon_view)).set_visible(cx, true);
                        let _ = section_label
                            .image(ids!(icon_image))
                            .load_image_dep_by_path(cx, dep.as_str());
                    }
                    _ => {
                        // For other Picture types (Image, Grapheme), show fallback
                        section_label.view(ids!(icon_view)).set_visible(cx, false);
                        section_label
                            .view(ids!(icon_fallback_view))
                            .set_visible(cx, true);
                        section_label.label(ids!(icon_fallback_label)).set_text(
                            cx,
                            &group_label
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_string()
                                .to_uppercase(),
                        );
                    }
                }
            } else {
                // No icon provided, show fallback
                section_label.view(ids!(icon_view)).set_visible(cx, false);
                section_label
                    .view(ids!(icon_fallback_view))
                    .set_visible(cx, true);
                section_label
                    .label(ids!(icon_fallback_label))
                    .set_text(cx, &group_label.chars().next().unwrap_or('?').to_string());
            }

            let _ = section_label.draw_all(cx, &mut Scope::empty());
            total_height += section_label.area().rect(cx).size.y;

            // Sort bots within group by name
            group_bots.sort_by(|a, b| a.name.cmp(&b.name));

            // Render bot items in this group
            for bot in group_bots {
                let item_id = LiveId::from_str(bot.id.as_str());

                let item_widget = self.items.get_or_insert(cx, item_id, |cx| {
                    WidgetRef::new_from_ptr(cx, self.item_template)
                });

                let mut item = item_widget.as_model_selector_item();
                item.set_bot(bot.clone());
                item.set_selected_bot_id(self.selected_bot_id.clone());

                let _ = item_widget.draw_all(cx, &mut Scope::empty());
                total_height += item_widget.area().rect(cx).size.y;
            }
        }

        self.total_height = Some(total_height);
    }
}

impl ModelSelectorListRef {
    pub fn get_height(&self) -> f64 {
        if let Some(inner) = self.borrow() {
            inner.total_height.unwrap_or(0.0)
        } else {
            0.0
        }
    }

    pub fn set_search_filter(&mut self, cx: &mut Cx, filter: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.search_filter = filter.to_string();
            inner.items.clear();
            inner.total_height = None;
            inner.redraw(cx);
        }
    }

    pub fn clear_search_filter(&mut self, cx: &mut Cx) {
        self.set_search_filter(cx, "");
    }

    pub fn set_chat_controller(&mut self, controller: Option<Arc<Mutex<ChatController>>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.chat_controller = controller;
        }
    }

    pub fn set_grouping(&mut self, grouping: Option<GroupingFn>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.grouping = grouping;
        }
    }

    pub fn set_selected_bot_id(&mut self, selected_bot_id: Option<BotId>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.selected_bot_id = selected_bot_id;
        }
    }
}
