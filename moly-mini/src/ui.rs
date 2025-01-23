use makepad_widgets::*;
use moly_widgets::*;

use moly_widgets::repos::moly::MolyRepo;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use moly_widgets::chat::Chat;

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
            chat = <Chat> {}
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
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let chat = self.chat(id!(chat));
        let mut chat = chat.borrow_mut().unwrap();

        chat.bot_repo = Some(Box::new(MolyRepo::default()));
        chat.bot_id = Some(BotId::from("moly"));
    }
}
