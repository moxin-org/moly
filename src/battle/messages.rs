use makepad_widgets::*;
use moxin_mae::MaeAgent;

use crate::chat::shared::ChatAgentAvatarWidgetRefExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::chat::chat_line_loading::ChatLineLoading;
    import crate::chat::shared::ChatAgentAvatar;

    Bubble = <RoundedView> {
        height: Fit,
        padding: {left: 16, right: 18, top: 18, bottom: 14},
        margin: {bottom: 16},
        show_bg: true,
        draw_bg: {
            radius: 12.0,
        },
        text = <Label> {
            width: Fill,
            draw_text: {
                text_style: <REGULAR_FONT>{height_factor: (1.3*1.3), font_size: 10},
                color: #000
            }
        }
    }

    UserLine = <View> {
        height: Fit,
        bubble = <Bubble> {
            margin: {left: 100}
            draw_bg: {color: #15859A}
            text = {
                draw_text: {color: #fff}
            }
        }
    }

    AgentLine = <View> {
        flow: Down,
        height: Fit,
        sender = <View> {
            height: Fit,
            spacing: 8,
            align: {y: 0.5}
            avatar = <ChatAgentAvatar> {}
            name = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000
                }
            }
        }
        bubble = <Bubble> {margin: {left: 16}}
    }

    LoadingLine = <AgentLine> {
        bubble = {
            text = <ChatLineLoading> {}
        }
    }

    Messages = {{Messages}} {
        flow: Down,
        width: Fill,
        height: Fill,

        list = <PortalList> {
            UserLine = <UserLine> {}
            AgentLine = <AgentLine> {}
            LoadingLine = <LoadingLine> {}
        }
    }
}

pub enum Message {
    User(String),
    Agent(MaeAgent, String),
    AgentWriting(MaeAgent),
}

#[derive(Live, LiveHook, Widget)]
pub struct Messages {
    #[deref]
    view: View,

    #[rust]
    messages: Vec<Message>,
}

impl Widget for Messages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(widget) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = widget.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.messages.len());
                while let Some(index) = list.next_visible_item(cx) {
                    if index >= self.messages.len() {
                        continue;
                    }

                    match &self.messages[index] {
                        Message::User(text) => {
                            let item = list.item(cx, index, live_id!(UserLine)).unwrap();
                            item.label(id!(text)).set_text(text);
                            item.draw_all(cx, scope);
                        }
                        Message::Agent(agent, text) => {
                            let item = list.item(cx, index, live_id!(AgentLine)).unwrap();
                            item.chat_agent_avatar(id!(avatar)).set_agent(agent);
                            item.label(id!(name)).set_text(&agent.name());
                            item.label(id!(text)).set_text(text);
                            item.draw_all(cx, scope);
                        }
                        Message::AgentWriting(agent) => {
                            let item = list.item(cx, index, live_id!(LoadingLine)).unwrap();
                            item.chat_agent_avatar(id!(avatar)).set_agent(agent);
                            item.label(id!(name)).set_text(&agent.name());
                            item.draw_all(cx, scope);
                        }
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for Messages {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {}
}

impl Messages {
    pub fn add_message(&mut self, message: Message) {
        if let Some(Message::AgentWriting(_)) = self.messages.last() {
            self.messages.pop();
        }

        self.messages.push(message);
    }

    pub fn scroll_to_bottom(&self, cx: &mut Cx) {
        self.portal_list(id!(list))
            .smooth_scroll_to_end(cx, 10, 80.0);
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

impl MessagesRef {
    pub fn add_message(&self, message: Message) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_message(message);
        }
    }

    pub fn scroll_to_bottom(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            inner.scroll_to_bottom(cx);
        }
    }

    pub fn len(&self) -> usize {
        self.borrow().map(|inner| inner.len()).unwrap_or(0)
    }
}
