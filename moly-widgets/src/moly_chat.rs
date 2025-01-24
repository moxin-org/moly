use crate::{protocol::*, repos::moly::MolyRepo, Chat};
use makepad_widgets::*;

live_design!(
    use crate::chat::*;
    pub MolyChat = {{MolyChat}} <Chat> {}
);

#[derive(Live, Widget)]
pub struct MolyChat {
    #[deref]
    deref: Chat,

    #[live]
    pub url: String,

    #[live]
    pub key: Option<String>,
}

impl Widget for MolyChat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for MolyChat {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // TODO: Ensure syncrhonization on updates.

        self.bot_repo = Some(Box::new(MolyRepo::new(self.url.clone(), self.key.clone())));
        // TODO: Allow selecting this.
        self.bot_id = Some(BotId::from("moly"));
    }
}
