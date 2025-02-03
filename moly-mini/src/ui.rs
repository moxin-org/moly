use makepad_widgets::*;
use moly_widgets::repos::moly::*;
use moly_widgets::utils::asynchronous::spawn;
use moly_widgets::{protocol::*, ChatWidgetExt};

use crate::bot_selector::BotSelectorWidgetExt;

// load from env var at compile time using macro
const OPEN_AI_KEY: &str = env!("OPENAI_API_KEY");

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
            // let mut repo: BotRepo = MolyService::new("http://localhost:8085".into()).into();
            // let mut repo: BotRepo = MolyService::new("http://localhost:11434".into()).into();
            let mut service = MolyService::new("https://api.openai.com".into());
            service.set_key(OPEN_AI_KEY);
            let mut repo: BotRepo = service.into();

            chat.borrow_mut().unwrap().bot_repo = Some(repo.clone());

            let ui = self.ui_runner();
            spawn(async move {
                repo.load().await.expect("TODO: Handle loading better");
                ui.defer_with_redraw(move |me, _cx, _scope| {
                    let chat = me.chat(id!(chat));

                    let bots = repo
                        .bots()
                        .into_iter()
                        .filter(|b| {
                            // Try to forcefully exclude some bots that will not work
                            // as open ai gives you a long list without telling you what
                            // which works with which endpoint.
                            let name = b.name.as_str();
                            let excluded = [
                                "-latest",
                                "-embedding",
                                "-audio",
                                "-20",
                                "-realtime",
                                "davinci",
                                "dall-e",
                                "whisper",
                                "babbage",
                                "tts",
                            ];

                            !excluded.iter().any(|ex| name.contains(ex))
                        })
                        .collect::<Vec<_>>();

                    chat.borrow_mut().unwrap().bot_id = Some(bots.first().unwrap().id.clone());
                    me.bot_selector(id!(selector)).set_bots(bots);

                    chat.borrow_mut().unwrap().visible = true;
                });
            });
        }

        let Event::Actions(actions) = event else {
            return;
        };

        if selector.bot_selected(actions) {
            let id = selector.selected_bot_id().expect("no bot selected");
            chat.borrow_mut().unwrap().bot_id = Some(id);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for Ui {}
