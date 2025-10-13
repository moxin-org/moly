use std::sync::{Arc, Mutex};

use makepad_widgets::*;
use moly_kit::controllers::chat::{ChatController, ChatControllerPlugin, ChatTask};
use moly_kit::mcp::mcp_manager::{McpManagerClient, McpTransport};
use moly_kit::utils::asynchronous::spawn;
use moly_kit::*;

use crate::bot_selector::BotSelectorWidgetExt;
use crate::tester_client::TesterClient;

const OPEN_AI_KEY: Option<&str> = option_env!("OPEN_AI_KEY");
const OPEN_AI_IMAGE_KEY: Option<&str> = option_env!("OPEN_AI_IMAGE_KEY");
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
        chat = <Chat> { }
    }
);

#[derive(Live, Widget)]
pub struct DemoChat {
    #[deref]
    deref: View,

    #[rust]
    pub controller: Option<Arc<Mutex<ChatController>>>,
}

impl Widget for DemoChat {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let selector = self.bot_selector(id!(selector));
        let mut chat = self.chat(id!(chat));

        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        let Event::Actions(actions) = event else {
            return;
        };

        if selector.bot_selected(actions) {
            let id = selector.selected_bot_id().expect("no bot selected");
            chat.write().set_bot_id(cx, Some(id));
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for DemoChat {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        // Setup some hooks as an example of how to use them.
        self.setup_chat_hooks();
        self.setup_chat_controller(cx);
    }
}

impl DemoChat {
    fn fill_selector(&mut self, cx: &mut Cx, bots: Vec<Bot>) {
        let mut chat = self.chat(id!(chat));

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

                let openai_image_whitelist = ["dall-e-3"];

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

                let siliconflow_whitelist = [
                    "Pro/Qwen/Qwen2-1.5B-Instruct",
                    "Pro/deepseek-ai/DeepSeek-R1",
                    "Pro/meta-llama/Meta-Llama-3.1-8B-Instruct",
                    "Qwen/Qwen2-7B-Instruct",
                ];

                let is_whitelisted_bot = openai_whitelist
                    .iter()
                    .chain(openai_image_whitelist.iter())
                    .chain(openrouter_whitelist.iter())
                    .chain(siliconflow_whitelist.iter())
                    .any(|s| *s == b.name.as_str());

                let is_local_bot =
                    b.id.provider() == "tester" || b.id.provider().contains("://localhost");

                is_whitelisted_bot || is_local_bot
            })
            .collect::<Vec<_>>();

        if let Some(bot) = bots.first() {
            chat.write().set_bot_id(cx, Some(bot.id.clone()));
        } else {
            eprintln!("No models available, check your API keys.");
        }

        self.bot_selector(id!(selector)).set_bots(bots);
    }

    fn setup_chat_hooks(&self) {
        // self.chat(id!(chat)).write_with(|chat| {
        //     chat.set_hook_before(|group, chat, cx| {
        //         let mut abort = false;

        //         for task in group.iter_mut() {
        //             if let ChatTask::CopyMessage(index) = task {
        //                 abort = true;

        //                 let text = chat.messages_ref().read_with(|messages| {
        //                     let text = &messages.messages[*index].content.text;
        //                     format!("You copied the following text from Moly (mini): {}", text)
        //                 });

        //                 cx.copy_to_clipboard(&text);
        //             }

        //             if let ChatTask::UpdateMessage(_index, message) = task {
        //                 message.content.text =
        //                     message.content.text.replace("ello", "3110 (hooked)");

        //                 if message.content.text.contains("bad word") {
        //                     abort = true;
        //                 }
        //             }
        //         }

        //         if abort {
        //             group.clear();
        //         }
        //     });

        //     chat.set_hook_after(|group, _, _| {
        //         for task in group.iter() {
        //             if let ChatTask::UpdateMessage(_index, message) = task {
        //                 log!("Message updated after hook: {:?}", message.content);
        //             }
        //         }
        //     });
        // });
    }

    fn setup_chat_controller(&mut self, cx: &mut Cx) {
        let client = {
            let mut client = MultiClient::new();

            let tester = TesterClient;
            client.add_client(Box::new(tester));

            let ollama = OpenAIClient::new("http://localhost:11434/v1".into());
            client.add_client(Box::new(ollama));

            if let Some(key) = OPEN_AI_IMAGE_KEY {
                let mut openai_image = OpenAIImageClient::new("https://api.openai.com/v1".into());
                let _ = openai_image.set_key(key);
                client.add_client(Box::new(openai_image));
            }

            // Only add OpenAI client if API key is present
            if let Some(key) = OPEN_AI_KEY {
                let openai_url = "https://api.openai.com/v1";
                let mut openai = OpenAIClient::new(openai_url.into());
                let _ = openai.set_key(key);
                client.add_client(Box::new(openai));
            }

            // Only add OpenRouter client if API key is present
            if let Some(key) = OPEN_ROUTER_KEY {
                let open_router_url = "https://openrouter.ai/api/v1";
                let mut open_router = OpenAIClient::new(open_router_url.into());
                let _ = open_router.set_key(key);
                client.add_client(Box::new(open_router));
            }

            // Only add SiliconFlow client if API key is present
            if let Some(key) = SILICON_FLOW_KEY {
                let siliconflow_url = "https://api.siliconflow.cn/api/v1";
                let mut siliconflow = OpenAIClient::new(siliconflow_url.into());
                let _ = siliconflow.set_key(key);
                client.add_client(Box::new(siliconflow));
            }

            client
        };

        // Create MCP manager and configure playwright tool
        let tool_manager = {
            let manager = McpManagerClient::new();

            // Configure playwright tool
            let playwright_transport = {
                let mut command = tokio::process::Command::new("zsh");
                command.arg("/Users/wyeworks/mcp/scripts/playwright.sh");
                McpTransport::Stdio(command)
            };

            let manager_clone = manager.clone();
            spawn(async move {
                if let Err(e) = manager_clone
                    .add_server("playwright", playwright_transport)
                    .await
                {
                    eprintln!("Failed to add playwright server: {}", e);
                }
            });

            manager
        };

        let controller = ChatController::builder()
            .with_client(client)
            .with_tool_manager(tool_manager)
            .with_plugin(DemoChatPlugin {
                ui: self.ui_runner(),
                initialized: false,
            })
            .build_arc();

        self.controller = Some(controller.clone());
        self.chat(id!(chat))
            .write()
            .set_chat_controller(cx, Some(controller));
    }
}

struct DemoChatPlugin {
    ui: UiRunner<DemoChat>,
    initialized: bool,
}

impl ChatControllerPlugin for DemoChatPlugin {
    fn on_state_change(&mut self, state: &controllers::chat::ChatState) {
        self.init(state);
        self.tools(state);
    }
}

impl DemoChatPlugin {
    fn init(&mut self, state: &controllers::chat::ChatState) {
        if self.initialized {
            return;
        }

        if !state.bots.is_empty() {
            let bots = state.bots.clone();
            self.ui.defer_with_redraw(move |widget, cx, _scope| {
                widget.fill_selector(cx, bots);
            });

            self.initialized = true;
            // TODO: Unsuscribe?
        }
    }

    fn tools(&mut self, state: &controllers::chat::ChatState) {
        let Some(message) = state.messages.last() else {
            return;
        };

        if message.content.tool_results.iter().any(|tr| !tr.is_error) {
            self.ui.defer(|widget, _, _| {
                let bot_id = widget.chat(id!(chat)).read().bot_id().cloned();
                widget
                    .controller
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .dispatch_task(ChatTask::Send(bot_id.unwrap()));
            });
        }
    }
}
