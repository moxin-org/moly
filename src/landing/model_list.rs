use crate::chat::entity_button::EntityButtonWidgetRefExt;
use crate::data::chats::AgentsAvailability;
use crate::data::providers::ProviderBot;
use crate::data::search::SearchAction;
use crate::data::store::{Store, StoreAction};
use crate::landing::search_loading::SearchLoadingWidgetExt;
use crate::shared::actions::ChatAction;
use makepad_widgets::*;
use moly_protocol::data::Model;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::landing::model_card::ModelCard;
    use crate::landing::search_loading::SearchLoading;
    use crate::chat::entity_button::*;

    AgentCard = <RoundedView> {
        width: Fill,
        height: 100,
        show_bg: false,
        draw_bg: {
            radius: 5,
            color: #F9FAFB,
        }
        button = <EntityButton> {
            width: Fill,
            height: Fill,
            padding: {left: 15, right: 15},
            spacing: 15,
            align: {x: 0, y: 0.35},
            server_url_visible: true,

            draw_bg: {
                radius: 5,
            }
            agent_avatar = {
                image = {
                    width: 64,
                    height: 64,
                }
            }
            text_layout = {
                height: Fit,
                flow: Down,
                caption = {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 11},
                    }
                }
                description = {
                    label = {
                        draw_text: {
                            wrap: Word,
                            color: #1D2939,
                        }
                    }
                }
            }
        }
    }

    pub ModelList = {{ModelList}} {
        width: Fill,
        height: Fill,

        flow: Overlay,

        content = <View> {
            width: Fill,
            height: Fill,
            list = <PortalList> {
                width: Fill,
                height: Fill,

                // We need this setting because we will have modal dialogs that should
                // "capture" the events, so we don't want to handle them here.
                capture_overload: false,

                AgentRow = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Right,
                    spacing: 15,

                    first = <AgentCard> {}
                    second = <AgentCard> {}
                    third = <AgentCard> {}
                }
                NoAgentsWarning = <Label> {
                    draw_text:{
                        wrap: Word
                        text_style: {font_size: 10},
                        color: #3
                    }
                }
                Header = <Label> {
                    margin: {bottom: 10, top: 35}
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 16},
                        color: #000
                    }
                }
                Model = <ModelCard> {
                    margin: {bottom: 30},
                }
            }
        }

        loading = <View> {
            width: Fill,
            height: Fill,
            visible: false,

            show_bg: true,
            draw_bg: {
                color: #FFFE,
            }
            search_loading = <SearchLoading> {}
        }

        search_error = <View> {
            width: Fill,
            height: Fill,
            visible: false,
            align: {x: 0.5, y: 0.5},

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 13},
                    color: #000
                }
                text: "Error fetching models. Check your connection to MolyServer."
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelList {
    #[deref]
    view: View,

    #[rust]
    loading_delay: Timer,
}

impl Widget for ModelList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.loading_delay.is_event(event).is_some() {
            self.update_loading_and_error_message(cx, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let agents = store.chats.get_mofa_agents_list(true);

        enum Item<'a> {
            AgentRow {
                agents: &'a [ProviderBot],
                margin_bottom: f32,
            },
            NoAgentsWarning(&'static str),
            Header(&'static str),
            Model(&'a Model),
        }

        let mut items = Vec::new();

        if store.search.keyword.is_none() {
            items.push(Item::Header("Featured Agents"));
            let agents_availability = store.chats.agents_availability();
            match agents_availability {
                AgentsAvailability::NoServers => items.push(Item::NoAgentsWarning(
                    agents_availability.to_human_readable(),
                )),
                AgentsAvailability::ServersNotConnected => items.push(Item::NoAgentsWarning(
                    agents_availability.to_human_readable(),
                )),
                AgentsAvailability::Available => {
                    items.extend(agents.chunks(3).map(|chunk| Item::AgentRow {
                        agents: chunk,
                        margin_bottom: 8.0,
                    }));
                    if let Some(Item::AgentRow { margin_bottom, .. }) = items.last_mut() {
                        *margin_bottom = 0.0;
                    }
                }
            }
            items.push(Item::Header("Models"));
        }

        items.extend(store.search.models.iter().map(Item::Model));

        while let Some(view_item) = self.view.draw_walk(cx, &mut Scope::empty(), walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, items.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < items.len() {
                        match items[item_id] {
                            Item::Header(text) => {
                                let item = list.item(cx, item_id, live_id!(Header));
                                item.set_text(cx, text);
                                item.draw_all(cx, &mut Scope::empty());
                            }
                            Item::AgentRow {
                                agents,
                                margin_bottom,
                            } => {
                                let row = list.item(cx, item_id, live_id!(AgentRow));

                                row.apply_over(
                                    cx,
                                    live! {
                                        margin: {bottom: (margin_bottom)},
                                    },
                                );

                                [id!(first), id!(second), id!(third)]
                                    .iter()
                                    .enumerate()
                                    .for_each(|(i, id)| {
                                        if let Some(agent) = agents.get(i) {
                                            let cell = row.view(*id);
                                            cell.apply_over(
                                                cx,
                                                live! {
                                                    show_bg: true,
                                                },
                                            );
                                            let mut button = cell.entity_button(id!(button));
                                            button.set_bot_id(cx, &agent.id);
                                            button.set_description_visible(cx, true);
                                        }
                                    });

                                row.draw_all(cx, &mut Scope::empty());
                            }
                            Item::NoAgentsWarning(text) => {
                                let item = list.item(cx, item_id, live_id!(NoAgentsWarning));
                                item.set_text(cx, text);
                                item.draw_all(cx, &mut Scope::empty());
                            }
                            Item::Model(model) => {
                                let item = list.item(cx, item_id, live_id!(Model));
                                let mut model_with_download_info =
                                    store.add_download_info_to_model(model);
                                item.draw_all(
                                    cx,
                                    &mut Scope::with_data(&mut model_with_download_info),
                                );
                            }
                        }
                    }
                }
            }
        }

        DrawStep::done()
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelListAction {
    None,
    ScrolledAtTop,
    ScrolledNotAtTop,
}

const SCROLLING_AT_TOP_THRESHOLD: f64 = -30.0;

impl WidgetMatchEvent for ModelList {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let portal_list = self.portal_list(id!(list));

        let clicked_entity_button = portal_list
            .items_with_actions(actions)
            .iter()
            .map(|(_, item)| item.entity_button(id!(button)))
            .find(|button| button.clicked(actions));

        if let Some(entity_button) = clicked_entity_button {
            let bot_id = entity_button.get_bot_id();
            if let Some(bot_id) = bot_id {
                cx.action(ChatAction::Start(bot_id));
            }
        }

        for action in actions.iter() {
            if let Some(_) = action.downcast_ref::<SearchAction>() {
                self.loading_delay = cx.start_timeout(0.2);
            }

            match action.cast() {
                StoreAction::Search(_) | StoreAction::ResetSearch => {
                    self.view(id!(search_error)).set_visible(cx, false);
                    self.view(id!(loading)).set_visible(cx, true);
                    self.search_loading(id!(search_loading)).animate(cx);
                    portal_list.set_first_id_and_scroll(0, 0.0);

                    self.redraw(cx);
                }
                _ => {}
            }
        }

        if portal_list.scrolled(actions) {
            if portal_list.first_id() == 0
                && portal_list.scroll_position() > SCROLLING_AT_TOP_THRESHOLD
            {
                cx.action(ModelListAction::ScrolledAtTop);
            } else {
                cx.action(ModelListAction::ScrolledNotAtTop);
            }
        }
    }
}

impl ModelList {
    fn update_loading_and_error_message(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get::<Store>().unwrap();
        let is_loading = store.search.is_pending();
        self.view(id!(loading)).set_visible(cx, is_loading);
        if is_loading {
            self.search_loading(id!(search_loading)).animate(cx);
        } else {
            self.search_loading(id!(search_loading)).stop_animation();
        }

        let is_errored = store.search.was_error();
        self.view(id!(search_error)).set_visible(cx, is_errored);
    }
}
