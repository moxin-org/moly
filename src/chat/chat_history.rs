use super::chat_history_card::ChatHistoryCardWidgetRefExt;
use crate::chat::entity_button::EntityButtonWidgetRefExt;
use crate::data::chats::chat::ChatID;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::chat_history_card::ChatHistoryCard;
    use crate::chat::entity_button::*;

    HeadingLabel = <Label> {
        margin: {left: 4, bottom: 4},
        draw_text:{
            text_style: <BOLD_FONT>{font_size: 10.5},
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

    pub ChatHistory = {{ChatHistory}} {
        width: Fill, height: Fill
        show_bg: true
        draw_bg: {
            color: (MAIN_BG_COLOR)
        }
        padding: { left: 10, right: 10 }

        list = <PortalList> {
            drag_scrolling: false,
            AgentHeading = <HeadingLabel> { text: "AGENTS" }
            NoAgentsWarning = <NoAgentsWarning> {}
            Agent = <EntityButton> {
                server_url_visible: true,
            }
            ChatsHeading = <HeadingLabel> { text: "CHATS", margin: {top: 10}, }
            ChatHistoryCard = <ChatHistoryCard> {
                cursor: Default
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistory {
    #[deref]
    deref: View,
}

impl Widget for ChatHistory {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        // let agents = store.chats.get_mofa_agents_list(true);

        enum Item<'a> {
            ChatsHeader,
            // AgentsHeader,
            // NoAgentsWarning(&'a str),
            // AgentButton(&'a ProviderBot),
            ChatButton(&'a ChatID),
        }

        let mut items: Vec<Item> = Vec::new();

        // TODO: Temporarily disabling the agents section in the chat history.
        // Reusing portal list items ids for different templates (e.g. a ChatsHeader becomes an AgentsHeader when agents are loaded after chats)
        // causes drawlist issues: Drawlist id generation wrong index: 13 current gen:1 in pointer:0 / Drawlist id generation wrong 13 1 0

        // if !agents.is_empty() {
        //     items.push(Item::AgentsHeader);
        //     for agent in &agents {
        //         items.push(Item::AgentButton(agent));
        //     }
        // }

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
                        // Item::AgentsHeader => {
                        //     let item = list.item(cx, item_id, live_id!(AgentHeading));
                        //     item.draw_all(cx, scope);
                        // }
                        // Item::AgentButton(agent) => {
                        //     let item = list.item(cx, item_id, live_id!(Agent));
                        //     item.as_entity_button().set_bot_id(cx, &agent.id);
                        //     item.draw_all(cx, scope);
                        // }
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
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let clicked_entity_button = self
            .portal_list(id!(list))
            .items_with_actions(actions)
            .iter()
            .map(|(_, item)| item.as_entity_button())
            .find(|eb| eb.clicked(actions));

        if let Some(entity_button) = clicked_entity_button {
            let bot_id = entity_button.get_bot_id();
            if let Some(bot_id) = bot_id {
                cx.action(ChatAction::Start(bot_id));
            }
        }
    }
}
