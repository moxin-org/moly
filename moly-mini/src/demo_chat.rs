use makepad_widgets::*;
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use crate::bot_selector::BotSelectorWidgetExt;

const OPEN_AI_KEY: Option<&str> = option_env!("OPEN_AI_KEY");
const OPEN_ROUTER_KEY: Option<&str> = option_env!("OPEN_ROUTER_KEY");
const SILICON_FLOW_KEY: Option<&str> = option_env!("SILICON_FLOW_KEY");

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
        let selector = self.bot_selector(id!(selector));
        let chat = self.chat(id!(chat));

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

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

impl LiveHook for DemoChat {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // Setup some hooks as an example of how to use them.
        self.setup_chat_hooks();
        self.setup_chat_repo();
    }
}

impl DemoChat {
    fn fill_selector(&mut self, bots: Vec<Bot>) {
        let chat = self.chat(id!(chat));

        let bots = bots
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
                    "Pro/meta-llama/Meta-Llama-3.1-8B-Instruct",
                    "Qwen/Qwen2-7B-Instruct",
                ];

                openai_whitelist
                    .iter()
                    .chain(openrouter_whitelist.iter())
                    .chain(ollama_whitelist.iter())
                    .chain(siliconflow_whitelist.iter())
                    .any(|s| *s == b.id.as_str())
            })
            .collect::<Vec<_>>();

        if let Some(bot) = bots.first() {
            chat.borrow_mut().unwrap().bot_id = Some(bot.id.clone());
        } else {
            eprintln!("No models available, check your API keys.");
        }

        self.bot_selector(id!(selector)).set_bots(bots);
    }

    fn setup_chat_hooks(&self) {
        self.chat(id!(chat)).write_with(|chat| {
            chat.set_hook_before(|group, chat, cx| {
                let mut abort = false;

                for task in group.iter_mut() {
                    if let ChatTask::CopyMessage(index) = task {
                        abort = true;

                        let text = chat.messages_ref().read_with(|messages| {
                            let text = messages.messages[*index].body.as_str();
                            format!("You copied the following text from Moly (mini): {}", text)
                        });

                        cx.copy_to_clipboard(&text);
                    }

                    if let ChatTask::UpdateMessage(_index, message) = task {
                        message.body = message.body.replace("ello", "3110 (hooked)");

                        if message.body.contains("bad word") {
                            abort = true;
                        }
                    }
                }

                if abort {
                    group.clear();
                }
            });

            chat.set_hook_after(|group, _, _| {
                for task in group.iter() {
                    if let ChatTask::UpdateMessage(_index, message) = task {
                        log!("Message updated after hook: {}", message.body);
                    }
                }
            });
        });
    }

    fn setup_chat_repo(&self) {
        let client = {
            // let moly = MolyClient::new("http://localhost:8085".into());
            let ollama = OpenAIClient::new("http://localhost:11434".into());

            let mut client = MultiClient::new();
            // client.add_client(Box::new(moly));
            client.add_client(Box::new(ollama));

            // Only add OpenAI client if API key is present
            if let Some(key) = OPEN_AI_KEY {
                let openai_url = "https://api.openai.com";
                let mut openai = OpenAIClient::new(openai_url.into());
                openai.set_key(key);
                client.add_client(Box::new(openai));
            }

            // Only add OpenRouter client if API key is present
            if let Some(key) = OPEN_ROUTER_KEY {
                let open_router_url = "https://openrouter.ai/api";
                let mut open_router = OpenAIClient::new(open_router_url.into());
                open_router.set_key(key);
                client.add_client(Box::new(open_router));
            }

            // Only add SiliconFlow client if API key is present
            if let Some(key) = SILICON_FLOW_KEY {
                let siliconflow_url = "https://api.siliconflow.cn";
                let mut siliconflow = OpenAIClient::new(siliconflow_url.into());
                siliconflow.set_key(key);
                client.add_client(Box::new(siliconflow));
            }

            client
        };

        let mut repo: BotRepo = client.into();
        self.chat(id!(chat)).write().bot_repo = Some(repo.clone());

        let ui = self.ui_runner();
        spawn(async move {
            repo.load().await.expect("TODO: Handle loading better");
            ui.defer_with_redraw(move |me, _cx, _scope| {
                me.fill_selector(repo.bots());
                me.chat(id!(chat)).write().visible = true;
            });
        });
    }
}
