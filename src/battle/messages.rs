use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    ChatLine = <View> {
        height: Fit,
        bubble = <RoundedView> {
            height: Fit,
            padding: {left: 16, right: 18, top: 18, bottom: 14},
            show_bg: true,
            draw_bg: {
                radius: 12.0,
            },
            text = <Label> {
                width: Fill,
                draw_text: {color: #000}
            }
        }
    }

    UserLine = <ChatLine> {
        bubble = {
            margin: {left: 100}
            draw_bg: {color: #15859A}
            text = {
                draw_text: {color: #fff}
            }
        }
    }
    AgentLine = <ChatLine> {}

    Messages = {{Messages}} {
        flow: Down,
        width: Fill,
        height: Fill,

        list = <PortalList> {
            UserLine = <UserLine> {}
            AgentLine = <AgentLine> {}
        }
    }
}

pub enum Message {
    User(String),
    Agent(String),
    AgentWriting,
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
                        Message::Agent(text) => {
                            let item = list.item(cx, index, live_id!(AgentLine)).unwrap();
                            item.label(id!(text)).set_text(text);
                            item.draw_all(cx, scope);
                        }
                        Message::AgentWriting => {
                            let item = list.item(cx, index, live_id!(AgentLine)).unwrap();
                            item.label(id!(text)).set_text("Agent is typing...");
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
        self.messages.push(message);
    }

    pub fn scroll_to_bottom(&self, cx: &mut Cx) {
        self.portal_list(id!(list))
            .smooth_scroll_to_end(cx, 10, 80.0);
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
}
