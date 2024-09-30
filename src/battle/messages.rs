use crate::data::battle;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::agent_markdown::AgentMarkdown;

    Bubble = <RoundedView> {
        height: Fit,
        padding: {left: 16, right: 18, top: 18, bottom: 14},
        margin: {bottom: 16},
        show_bg: true,
        draw_bg: {
            radius: 12.0,
        }
    }

    UserLine = <View> {
        height: Fit,
        bubble = <Bubble> {
            margin: {left: 100}
            draw_bg: {color: #15859A}
            text = <Label> {
                width: Fill,
                draw_text: {
                    text_style: <REGULAR_FONT>{height_factor: (1.3*1.3), font_size: 10},
                    color: #fff
                }
            }
        }
    }

    AgentLine = <View> {
        height: Fit,
        bubble = <Bubble> {
            margin: {left: 16}
            text = <AgentMarkdown> {}
        }
    }

    Messages = {{Messages}} {
        flow: Down,
        width: Fill,
        height: Fill,

        list = <PortalList> {
            scroll_bar: {
                bar_size: 0.0,
            }
            UserLine = <UserLine> {}
            AgentLine = <AgentLine> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Messages {
    #[deref]
    view: View,

    #[rust]
    messages: Vec<battle::Message>,
}

impl Widget for Messages {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(widget) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = widget.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.messages.len());
                while let Some(index) = list.next_visible_item(cx) {
                    if index >= self.messages.len() {
                        continue;
                    }

                    let message = &self.messages[index];
                    let template = match message.sender {
                        battle::Sender::User => live_id!(UserLine),
                        battle::Sender::Agent => live_id!(AgentLine),
                    };

                    let item = list.item(cx, index, template).unwrap();
                    item.label(id!(text)).set_text(&message.body);
                    item.draw_all(cx, scope);
                }
            }
        }

        DrawStep::done()
    }
}

impl Messages {
    pub fn set_messages(&mut self, messages: Vec<battle::Message>) {
        self.messages = messages;
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
    pub fn set_messages(&self, messages: Vec<battle::Message>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_messages(messages);
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
