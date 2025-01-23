use super::chat_history_card::ChatHistoryCardWidgetRefExt;
use crate::chat::entity_button::EntityButtonWidgetRefExt;
use crate::data::chats::chat::ChatID;
use crate::data::chats::AgentsAvailability;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;
use makepad_widgets::*;
use moly_mofa::MofaAgent;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::shared::ChatAgentAvatar;
    use crate::chat::chat_history_card::ChatHistoryCard;
    use crate::chat::shared::ChatModelAvatar;
    use crate::chat::entity_button::*;

    ICON_NEW_CHAT = dep("crate://self/resources/icons/new_chat.svg")

    HeadingLabel = <Label> {
        margin: {left: 4, bottom: 4},
        draw_text:{
            text_style: <BOLD_FONT>{font_size: 10},
            color: #3
        }
    }

    NoAgentsWarning = <Label> {
        margin: {left: 4, bottom: 4},
        width: Fill
        draw_text:{
            text_style: {font_size: 8.5},
            color: #3
        }
    }

    pub ChatHistory = {{ChatHistory}} <MolyTogglePanel> {
        open_content = {
            <View> {
                width: Fill,
                height: Fill,
                show_bg: true
                draw_bg: {
                    color: #F2F4F7
                }

                <View> {
                    width: Fill,
                    height: Fill,

                    margin: { top: 120 }
                    padding: { left: 14, right: 14, bottom: 58 }

                    list = <PortalList> {
                        drag_scrolling: false,
                        AgentHeading = <HeadingLabel> { text: "AGENTS" }
                        NoAgentsWarning = <NoAgentsWarning> {}
                        Agent = <EntityButton> {
                            server_url_visible: true,
                        }
                        ChatsHeading = <HeadingLabel> { text: "CHATS", margin: {top: 10}, }
                        ChatHistoryCard = <ChatHistoryCard> {
                            padding: {top: 4}
                            cursor: Default
                        }
                    }
                }
            }
        }

        persistent_content = {
            margin: { left: -10 },
            default = {
                after = {
                    new_chat_button = <MolyButton> {
                        width: Fit,
                        height: Fit,
                        icon_walk: {margin: { top: -1 }, width: 21, height: 21},
                        draw_icon: {
                            svg_file: (ICON_NEW_CHAT),
                            fn get_color(self) -> vec4 {
                                return #475467;
                            }
                        }
                    }
                }
            }
        }

    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistory {
    #[deref]
    deref: TogglePanel,
}

impl Widget for ChatHistory {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let agents = store.chats.get_agents_list();

        enum Item<'a> {
            ChatsHeader,
            AgentsHeader,
            NoAgentsWarning(&'a str),
            AgentButton(&'a MofaAgent),
            ChatButton(&'a ChatID),
        }

        let mut items: Vec<Item> = Vec::new();

        items.push(Item::AgentsHeader);
        let agents_availability = store.chats.agents_availability();
        match agents_availability {
            AgentsAvailability::NoServers => items.push(Item::NoAgentsWarning(agents_availability.to_human_readable())),
            AgentsAvailability::ServersNotConnected => items.push(Item::NoAgentsWarning(agents_availability.to_human_readable())),
            AgentsAvailability::NoAgents => items.push(Item::NoAgentsWarning(agents_availability.to_human_readable())),
            AgentsAvailability::Available => {
                for agent in &agents {
                    items.push(Item::AgentButton(agent));
                }
            }
        }

        items.push(Item::ChatsHeader);

        let mut chat_ids = store
            .chats
            .saved_chats
            .iter()
            .map(|c| c.borrow().id)
            .collect::<Vec<_>>();

        // Reverse sort chat ids.
        chat_ids.sort_by(|a, b| b.cmp(a));

        items.extend(chat_ids.iter().map(Item::ChatButton));

        while let Some(view_item) = self.deref.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, items.len() - 1);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id >= items.len() {
                        // For some reason, if the range is < 5, the last item some times exceeds the range.
                        continue;
                    }

                    match &items[item_id] {
                        Item::ChatsHeader => {
                            let item = list.item(cx, item_id, live_id!(ChatsHeading));
                            item.draw_all(cx, scope);
                        }
                        Item::AgentsHeader => {
                            let item = list.item(cx, item_id, live_id!(AgentHeading));
                            item.draw_all(cx, scope);
                        }
                        Item::NoAgentsWarning(text) => {
                            let item = list.item(cx, item_id, live_id!(NoAgentsWarning));
                            item.set_text(text);
                            item.draw_all(cx, scope);
                        }
                        Item::AgentButton(agent) => {
                            let item = list.item(cx, item_id, live_id!(Agent));
                            item.as_entity_button().set_agent(agent);
                            item.draw_all(cx, scope);
                        }
                        Item::ChatButton(chat_id) => {
                            let mut item = list
                                .item(cx, item_id, live_id!(ChatHistoryCard))
                                .as_chat_history_card();
                            let _ = item.set_chat_id(**chat_id);
                            item.draw_all(cx, scope);
                        }
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatHistory {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let clicked_entity_button = self
            .portal_list(id!(list))
            .items_with_actions(actions)
            .iter()
            .map(|(_, item)| item.as_entity_button())
            .find(|eb| eb.clicked(actions));

        if let Some(entity_button) = clicked_entity_button {
            let entity_id = entity_button.get_entity_id().unwrap().clone();
            cx.action(ChatAction::Start(entity_id));
        }

        if self.button(id!(new_chat_button)).clicked(&actions) {
            store.chats.create_empty_chat();
            self.redraw(cx);
        }
    }
}
