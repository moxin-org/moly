use crate::{
    data::{chats::chat_entity::ChatEntityId, remote_servers::RemoteModel, store::Store},
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
        section_label_template: <Label> {
            padding: {left: 4, top: 4., bottom: 4.}
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 10.0},
                color: #98A2B3
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
    agent_template: Option<LivePtr>,
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
        
        // Keep track of the current item index for LiveId generation
        let mut current_index = 0;

        let chat_entity = store
            .chats
            .get_current_chat()
            .and_then(|c| c.borrow().associated_entity.clone());

        // Get non-agent models
        let non_agent_models = store.chats.get_non_mofa_models_list(true);

        // Group models by provider
        let mut models_by_provider: HashMap<String, (String, Vec<RemoteModel>)> = HashMap::new();
        
        for model in non_agent_models.iter() {
            // Get provider name from the providers map
            let provider_name = store.chats.providers
                .get(&model.provider_url)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown Provider".to_string());
                
            models_by_provider
                .entry(model.provider_url.clone())
                .or_insert_with(|| (provider_name, Vec::new()))
                .1
                .push(model.clone());
        }

        // Convert to a vector and sort by provider name for consistent ordering
        let mut providers: Vec<(String, String, Vec<RemoteModel>)> = models_by_provider
            .into_iter()
            .map(|(url, (name, models))| (url, name, models))
            .collect();
        
        // Sort providers alphabetically by name
        providers.sort_by(|a, b| a.1.cmp(&b.1));

        // Add models grouped by provider
        for (_provider_url, provider_name, mut provider_models) in providers {
            if provider_models.is_empty() {
                continue;
            }

            // Sort models within each provider by name for consistent ordering
            provider_models.sort_by(|a, b| a.name.cmp(&b.name));

            // Add provider section label
            let section_id = LiveId(current_index as u64).into();
            current_index += 1;
            
            let section_label = self.items.get_or_insert(cx, section_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.section_label_template)
            });
            section_label.set_text(cx, &provider_name);
            let _ = section_label.draw_all(cx, &mut Scope::empty());
            total_height += section_label.as_view().area().rect(cx).size.y;

            // Add models for this provider
            for remote_model in provider_models.iter() {
                let item_id = LiveId(current_index as u64).into();
                current_index += 1;
                
                let item_widget = self.items.get_or_insert(cx, item_id, |cx| {
                    WidgetRef::new_from_ptr(cx, self.model_template)
                });

                // Remove the starting 'models/' at the beginning of the name (happens with Gemini)
                let caption = &remote_model.name.trim_start_matches("models/");
                let size_visible = false;

                let icon_tick_visible = match &chat_entity {
                    Some(ChatEntityId::RemoteModel(model_id)) => model_id.0 == remote_model.id.0,
                    _ => false,
                };

                item_widget.apply_over(
                    cx,
                    live! {
                        content = {
                            label = { text: (caption) }
                            architecture_tag = { visible: false }
                            params_size_tag = { visible: false }
                            file_size_tag = { visible: (size_visible), caption = { text: "" } }
                            icon_tick_tag = { visible: (icon_tick_visible) }
                        }
                    },
                );

                item_widget
                    .as_model_selector_item()
                    .set_remote_model(remote_model.clone());
                
                let _ = item_widget.draw_all(cx, &mut Scope::empty());
                total_height += item_widget.view(id!(content)).area().rect(cx).size.y;
            }
        }

        // Add agents section if we have any
        let mut agents = store.chats.get_mofa_agents_list(true);
        // Sort agents by name for consistent ordering
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        
        if !agents.is_empty() {
            let section_id = LiveId(current_index as u64).into();
            current_index += 1;
            
            let section_label = self.items.get_or_insert(cx, section_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.section_label_template)
            });
            section_label.set_text(cx, "Agents");
            let _ = section_label.draw_all(cx, &mut Scope::empty());
            total_height += section_label.as_view().area().rect(cx).size.y;
        }

        // Add agents
        for agent in agents.iter() {
            let item_id = LiveId(current_index as u64).into();
            current_index += 1;
            
            let item_widget = self.items.get_or_insert(cx, item_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.agent_template)
            });

            let agent_name = &agent.name;
            let current_agent_name = match &chat_entity {
                Some(ChatEntityId::Agent(agent_id)) => {
                    store.chats.remote_models.get(agent_id).map(|m| &m.name)
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

        // Add local models section header if we have any models
        if !models.is_empty() {
            // Add local models section header
            let section_id = LiveId(current_index as u64).into();
            current_index += 1;
            
            let section_label = self.items.get_or_insert(cx, section_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.section_label_template)
            });
            section_label.set_text(cx, "Local models");
            let _ = section_label.draw_all(cx, &mut Scope::empty());
            total_height += section_label.as_view().area().rect(cx).size.y;
        }

        // Draw local models
        for model in models.iter() {
            let item_id = LiveId(current_index as u64).into();
            current_index += 1;
            
            let item_widget = self.items.get_or_insert(cx, item_id, |cx| {
                WidgetRef::new_from_ptr(cx, self.model_template)
            });
            self.map_to_downloaded_files
                .insert(item_id, model.clone());

            let caption = &model.file.name;

            let architecture = &model.model.architecture;
            let architecture_visible = !architecture.trim().is_empty();

            let param_size = &model.model.size;
            let param_size_visible = !param_size.trim().is_empty();

            let size = format_model_size(&model.file.size).unwrap_or("".to_string());
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
                .set_model(model.clone());

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
            total_height += item_widget.view(id!(content)).area().rect(cx).size.y;
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
