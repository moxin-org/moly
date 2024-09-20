use makepad_widgets::*;
use moxin_mae::MaeAgent;

use crate::chat::shared::ChatAgentAvatarWidgetExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::chat::shared::ChatAgentAvatar;

    NoMessages = {{NoMessages}} {
        flow: Down,
        spacing: 10,
        align: {x: 0.5, y: 0.5},
        avatar = <ChatAgentAvatar> {},
        text = <Label> {
            draw_text: {
                text_style: {font_size: 14},
                color: #000
            }
            text: "How can I help you?"
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct NoMessages {
    #[deref]
    view: View,
}

impl Widget for NoMessages {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl NoMessages {
    pub fn set_visible(&mut self, visible: bool) {
        self.view.visible = visible;
    }

    pub fn set_agent(&mut self, agent: MaeAgent) {
        self.chat_agent_avatar(id!(avatar)).set_agent(&agent);
    }
}

impl NoMessagesRef {
    pub fn set_visible(&self, visible: bool) {
        self.borrow_mut().map(|mut s| s.set_visible(visible));
    }

    pub fn set_agent(&self, agent: MaeAgent) {
        self.borrow_mut().map(|mut s| s.set_agent(agent));
    }
}
