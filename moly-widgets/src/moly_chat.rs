use crate::{protocol::*, repos::moly::MolyService, utils::asynchronous::spawn, ChatWidgetExt};
use makepad_widgets::*;

live_design!(
    use crate::chat::*;
    pub MolyChat = {{MolyChat}} {
        chat = <Chat> { visible: false }
    }
);

#[derive(Live, Widget)]
pub struct MolyChat {
    // could deref chat directly but setting visible false on it would prevent
    // handling event here
    #[deref]
    deref: View,

    #[live]
    pub url: String,

    #[live]
    pub key: Option<String>,
}

impl Widget for MolyChat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for MolyChat {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // TODO: Ensure syncrhonization on updates.
        let mut repo: BotRepo = MolyService::new(self.url.clone()).into();
        self.chat(id!(chat)).borrow_mut().unwrap().bot_repo = Some(repo.clone());

        let ui = self.ui_runner();
        spawn(async move {
            repo.load().await.expect("TODO: Handle loading better");
            ui.defer_with_redraw(move |me, _cx, _scope| {
                let chat = me.chat(id!(chat));
                let mut chat = chat.borrow_mut().unwrap();
                chat.bot_id = Some(repo.bots().first().unwrap().id);
                chat.visible = true;
            });
        });
    }
}
