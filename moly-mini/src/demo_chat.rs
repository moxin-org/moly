use makepad_widgets::*;
use moly_kit::clients::moly::*;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::{protocol::*, ChatTask, ChatWidgetExt};

use crate::bot_selector::BotSelectorWidgetExt;

const OPEN_AI_KEY: Option<&str> = option_env!("OPENAI_API_KEY");
const OPENAI_API_URL: Option<&str> = option_env!("OPENAI_API_URL");

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use moly_kit::widgets::chat::Chat;
    use crate::bot_selector::*;

    pub DemoChat = {{DemoChat}} {
        flow: Down,
        padding: 12,
        spacing: 12,
        selector = <BotSelector> {}
        chat = <Chat> { visible: false }
    }
);

#[derive(Live, Widget)]
pub struct DemoChat {
    #[deref]
    deref: View,
}

impl Widget for DemoChat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui_runner().handle(cx, event, scope, self);

        self.chat(id!(chat)).read_with(|chat| {
            chat.hook(event).write_with(|hook| {
                let mut abort = false;

                for task in hook.tasks() {
                    if let ChatTask::CopyMessage(index) = task {
                        abort = true;

                        let text = chat.messages_ref().read_with(|messages| {
                            let text = messages.messages[*index].body.as_str();
                            format!("You copied the following text from Moly (mini): {}", text)
                        });

                        cx.copy_to_clipboard(&text);
                    }
                }

                if abort {
                    hook.abort();
                }
            });
        });

        self.deref.handle_event(cx, event, scope);

        let selector = self.bot_selector(id!(selector));
        let chat = self.chat(id!(chat));

        if let Event::Startup = event {
            // TODO: Ensure syncrhonization on updates.
            let client = {
                let moly = MolyClient::new("http://localhost:8085".into());
                let ollama = MolyClient::new("http://localhost:11434".into());

                let openai_url = OPENAI_API_URL.unwrap_or("https://api.openai.com");
                let mut openai = MolyClient::new(openai_url.into());
                openai.set_key(OPEN_AI_KEY.unwrap_or(""));

                let mut client = MultiBotClient::new();
                client.add_client(Box::new(moly));
                client.add_client(Box::new(ollama));
                client.add_client(Box::new(openai));
                client
            };

            let mut repo: BotRepo = client.into();

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
                            let openai_whitelist = [
                                "gpt-4o",
                                "gpt-4o-mini",
                                "o1",
                                "o1-preview",
                                "o1-mini",
                                "o3-mini",
                                "o3-mini-high",
                            ];

                            let openrouter_whitelist = [
                                "openai/gpt-4o",
                                "openai/gpt-4o-mini",
                                "openai/o1",
                                "openai/o1-preview",
                                "openai/o1-mini",
                                "openai/o3-mini",
                                "openai/o3-mini-high",
                                "perplexity/sonar",
                                "perplexity/sonar-reasoning",
                                "perplexity/r1-1776",
                                "openrouter/auto",
                                "google/gemini-2.0-flash-001",
                                "anthropic/claude-3.5-sonnet",
                                "deepseek/deepseek-r1",
                            ];

                            let ollama_whitelist = [
                                "deepseek-r1:1.5b",
                                "deepseek-r1:8b",
                                "llama3.1:8b",
                                "llama3.2:latest",
                            ];

                            let siliconflow_whitelist = [
                                "Pro/Qwen/Qwen2-1.5B-Instruct",
                                "Pro/deepseek-ai/DeepSeek-R1",
                            ];

                            openai_whitelist
                                .iter()
                                .chain(openrouter_whitelist.iter())
                                .chain(ollama_whitelist.iter())
                                .chain(siliconflow_whitelist.iter())
                                .any(|s| *s == b.id.as_str())
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

impl LiveHook for DemoChat {}
