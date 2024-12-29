use crate::{
    data::{chats::chat_entity::ChatEntityId, store::Store},
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
        model_template: <ModelSelectorItem> { content = <ModelInfo> {} }
        agent_template: <ModelSelectorItem> { content = <AgentInfo> {} }
        separator_template: <Line> {}
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
    agent_template: Option<LivePtr>,
    #[live]
    separator_template: Option<LivePtr>,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,

    #[rust]
    map_to_downloaded_files: HashMap<LiveId, DownloadedFile>,

    #[rust]
    total_height: Option<f64>,
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
        models.sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));

        self.map_to_downloaded_files = HashMap::new();
        let mut total_height = 0.0;
        let models_count = models.len();

        let chat_entity = store
            .chats
            .get_current_chat()
            .and_then(|c| c.borrow().associated_entity.clone());

        for i in 0..models.len() {
            let item_id = LiveId(i as u64).into();
            let item_widget = self.items.get_or_insert(cx, item_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.model_template)
            });
            self.map_to_downloaded_files
                .insert(item_id, models[i].clone());

            let caption = &models[i].file.name;

            let architecture = &models[i].model.architecture;
            let architecture_visible = !architecture.trim().is_empty();

            let param_size = &models[i].model.size;
            let param_size_visible = !param_size.trim().is_empty();

            let size = format_model_size(&models[i].file.size).unwrap_or("".to_string());
            let size_visible = !size.trim().is_empty();

            let current_file_id = match chat_entity {
                Some(ChatEntityId::ModelFile(ref file_id)) => Some(file_id.clone()),
                Some(ChatEntityId::Agent(_)) => None,
                _ => store.chats.loaded_model.as_ref().map(|m| m.id.clone()),
            };
            let icon_tick_visible = current_file_id.as_ref()
                == Some(&self.map_to_downloaded_files.get(&item_id).unwrap().file.id);

            item_widget.apply_over(
                cx,
                live! {
                    content = {
                        label = { text: (caption) }
                        architecture_tag = { visible: (architecture_visible), caption = { text: (architecture) } }
                        params_size_tag = { visible: (param_size_visible), caption = { text: (param_size) } }
                        file_size_tag = { visible: (size_visible), caption = { text: (size) } }
                        icon_tick_tag = { visible: (icon_tick_visible) }
                    }
                },
            );

            item_widget
                .as_model_selector_item()
                .set_model(models[i].clone());

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
            total_height += item_widget.view(id!(content)).area().rect(cx).size.y;
        }

        if models_count > 0 {
            let separator_id = LiveId(models_count as u64).into();
            let separator_widget = self.items.get_or_insert(cx, separator_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.separator_template)
            });
            if moly_mofa::should_be_visible() {
                let _ = separator_widget.draw_all(cx, &mut Scope::empty());
                total_height += separator_widget.as_view().area().rect(cx).size.y;
            }
        }

        if moly_mofa::should_be_visible() {
            let agents = store.chats.get_agents_list();
            for (i, agent) in agents.iter().enumerate() {
                let item_id = LiveId((models_count + 1 + i) as u64).into();
                let item_widget = self.items.get_or_insert(cx, item_id, |cx| {
                    WidgetRef::new_from_ptr(cx, self.agent_template)
                });

                let agent_name = &agent.name;
                let current_agent_name = match &chat_entity {
                    Some(ChatEntityId::Agent(agent_id)) => {
                        store.chats.available_agents.get(agent_id).map(|a| &a.name)
                    },
                    _ => None,
                };
                let icon_tick_visible = current_agent_name == Some(agent_name);

                item_widget.apply_over(
                    cx,
                    live! {
                        content = {
                            label = { text: (agent_name) }
                            icon_tick_tag = { visible: (icon_tick_visible) }
                        }
                    },
                );
                
                item_widget
                    .as_model_selector_item()
                    .set_agent(agent.clone());

                let _ = item_widget.draw_all(cx, &mut Scope::empty());
                total_height += item_widget.view(id!(content)).area().rect(cx).size.y;
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
}
