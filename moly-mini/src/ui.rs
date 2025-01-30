use makepad_widgets::*;
use moly_widgets::repos::moly::*;
use moly_widgets::utils::asynchronous::spawn;
use moly_widgets::{protocol::*, ChatWidgetExt};

use crate::bot_selector::BotSelectorWidgetExt;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use moly_widgets::chat::*;
    use crate::bot_selector::*;

    pub Ui = {{Ui}} <Window> {
        align: {x: 0.5, y: 0.5}
        pass: { clear_color: #fff }

        caption_bar = {
            caption_label = {
                // remove the default label
                label = <Label> {}
                <View> {
                    width: Fill,
                    align: {x: 0.5, y: 0.5},
                    <Label> {
                        text: "moly-mini"
                        draw_text: {
                            color: #000
                        }
                    }
                }
            }

            visible: true,
        }

        body = <View> {
            flow: Down,
            padding: 12,
            spacing: 12,
            selector = <BotSelector> {}
            chat = <Chat> { /*url: "http://localhost:8085"*/ visible: false }
        }
    }
);

#[derive(Live, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        let selector = self.bot_selector(id!(selector));
        let chat = self.chat(id!(chat));

        if let Event::Startup = event {
            // TODO: Ensure syncrhonization on updates.
            // let mut repo: BotRepo = MolyService::new("http://localhost:8085".into(), None).into();
            let mut repo: BotRepo = MolyService::new("http://localhost:11434".into(), None).into();
            chat.borrow_mut().unwrap().bot_repo = Some(repo.clone());

            let ui = self.ui_runner();
            spawn(async move {
                repo.load().await.expect("TODO: Handle loading better");
                ui.defer_with_redraw(move |me, _cx, _scope| {
                    let chat = me.chat(id!(chat));
                    chat.borrow_mut().unwrap().bot_id = Some(repo.bots().first().unwrap().id);

                    me.bot_selector(id!(selector)).set_bots(repo.bots());

                    chat.borrow_mut().unwrap().visible = true;
                });
            });
        }

        let Event::Actions(actions) = event else {
            return;
        };

        if selector.bot_selected(actions) {
            chat.borrow_mut().unwrap().bot_id =
                Some(selector.selected_bot_id().expect("no bot selected"));
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {}
