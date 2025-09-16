use makepad_widgets::*;
use moly_kit::{protocol::*, *};

use crate::demo_chat::DemoChatWidgetExt;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::demo_chat::*;

    pub Ui = {{Ui}} <Window> {
        align: {x: 0.5, y: 0.5}
        pass: { clear_color: #fff }

        // caption_bar = {
        //     caption_label = {
        //         // remove the default label
        //         label = <Label> {}
        //         <View> {
        //             width: Fill,
        //             align: {x: 0.5, y: 0.5},
        //             <Label> {
        //                 text: "moly-mini"
        //                 draw_text: {
        //                     color: #000
        //                 }
        //             }
        //         }
        //     }

        //     visible: true,
        // }

        body = <View> {
            // chat_1 = <DemoChat> {}
            chat_2 = <DemoChat> {}
        }
    }
);

#[derive(Live, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,
}

impl Widget for Ui {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);

        if let Event::Startup = event {
            let bot_id = BotId::new("unknown_bot", "unknown_provider");

            let messages = std::iter::repeat([
                Message {
                    from: EntityId::User,
                    content: MessageContent {
                        text: "Hello".to_string(),
                        citations: vec![],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Message {
                    from: EntityId::Bot(bot_id),
                    content: MessageContent {
                        text: "World".to_string(),
                        attachments: vec![
                            Attachment::from_bytes(
                                "text.txt".into(),
                                Some("text/plain".into()),
                                b"Hello, world!",
                            ),
                            Attachment::from_bytes(
                                "image.png".into(),
                                Some("image/png".into()),
                                include_bytes!("../../packaging/Moly macOS dmg background.png"),
                            ),
                        ],
                        citations: vec![
                            "https://github.com/ZhangHanDong/url-preview/issues/2".to_string(),
                            "https://3.basecamp.com/5400951/buckets/28531977/messages/8467029657"
                                .to_string(),
                            "https://en.wikipedia.org/wiki/ICO_(file_format)".to_string(),
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ])
            .take(1)
            .flatten()
            .collect();

            self.demo_chat(id!(chat_2))
                .chat(id!(chat))
                .borrow()
                .unwrap()
                .messages_ref()
                .borrow_mut()
                .unwrap()
                .messages = messages;
        }
    }
}

impl LiveHook for Ui {}
