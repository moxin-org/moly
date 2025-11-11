use std::sync::{Arc, Mutex};

use makepad_widgets::*;
use moly_kit::controllers::chat::{
    ChatController, ChatControllerPlugin, ChatStateMutation, ChatTask,
};
use moly_kit::mcp::mcp_manager::{McpManagerClient, McpTransport};
use moly_kit::utils::asynchronous::spawn;
use moly_kit::utils::vec::VecMutation;
use moly_kit::*;

const OPEN_AI_KEY: Option<&str> = option_env!("OPEN_AI_KEY");
const OPEN_AI_IMAGE_KEY: Option<&str> = option_env!("OPEN_AI_IMAGE_KEY");
const OPEN_AI_REALTIME_KEY: Option<&str> = option_env!("OPEN_AI_REALTIME_KEY");
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
        self.ui_runner().handle(cx, event, scope, self);
        self.deref.handle_event(cx, event, scope);

        let Event::Actions(_actions) = event else {
            return;
        };
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
    fn fill_selector(&mut self, _cx: &mut Cx, bots: Vec<Bot>) {
        // ════════════════════════════════════════════════════════════════════════════════
        // Bot Filtering Approaches in MolyKit
        // ════════════════════════════════════════════════════════════════════════════════
        //
        // There are 3 ways to filter bots (from earliest to latest in the pipeline):
        //
        // 1. CLIENT-LEVEL (at fetch time)
        //    - Filter bots when fetching from the provider
        //    - Use client.set_map_bots() or filter in fetch_models_with_client()
        //    - Example: Moly's bot_fetcher.rs filters by provider enabled status
        //    - Best for: Provider-level filtering (e.g., only include certain providers)
        //
        // 2. CONTROLLER-LEVEL (anytime after initialization) ← THIS EXAMPLE
        //    - Filter bots before setting them in ChatController
        //    - Use VecMutation::Set(filtered_bots) which becomes ChatStateMutation::MutateBots
        //    - Best for: Simple examples, static whitelists, one-time filtering
        //
        // 3. UI-LEVEL (at display time)
        //    - Keep all bots in ChatController, filter only in the ModelSelector UI
        //    - Implement BotFilter trait and set it on ModelSelectorList
        //    - Example: Moly's MolyBotFilter filters by individual bot enabled status
        //    - Best for: Dynamic filtering (e.g., user toggles), showing disabled bots differently
        //
        // This example uses approach #2 (controller-level) for simplicity.
        // ════════════════════════════════════════════════════════════════════════════════

        // Filter bots to only show whitelisted and local models
        let filtered_bots = bots
            .into_iter()
            .filter(|b| {
                let openai_whitelist = [
                    "gpt-5",
                    "gpt-5-mini",
                    "gpt-5-nano",
                    "o4-mini-high",
                    "o4-mini-deep-research",
                ];

                let openai_image_whitelist = ["dall-e-3", "gpt-image-1-mini", "gpt-image-1"];

                let openai_realtime_whitelist = ["gpt-realtime", "gpt-realtime-mini"];

                let openrouter_whitelist = [
                    "google/gemini-2.5-flash",
                    "google/gemini-2.5-pro",
                    "google/gemini-2.5-flash-lite",
                    "openai/gpt-5",
                    "openai/gpt-5-mini",
                    "openai/gpt-5-nano",
                    "openai/o4-mini-high",
                    "openai/o4-mini-deep-research",
                    "anthropic/claude-sonnet-4.5",
                    "anthropic/claude-haiku-4.5",
                    "deepseek/deepseek-r1-0528",
                    "deepseek/deepseek-chat-v3-0324",
                    "mistralai/mistral-nemo",
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
                    .chain(openai_realtime_whitelist.iter())
                    .chain(openrouter_whitelist.iter())
                    .chain(siliconflow_whitelist.iter())
                    .any(|s| *s == b.name.as_str());

                let is_local_bot =
                    b.id.provider() == "tester" || b.id.provider().contains("://localhost");

                is_whitelisted_bot || is_local_bot
            })
            .collect::<Vec<_>>();

        // Set filtered bots in the controller - the default ModelSelector will use these
        // Note: VecMutation::Set(Vec<Bot>) automatically converts to ChatStateMutation::MutateBots
        let mut controller = self.controller.as_ref().unwrap().lock().unwrap();
        controller.dispatch_mutation(VecMutation::Set(filtered_bots.clone()));

        // Select the first available bot
        if let Some(bot) = filtered_bots.first() {
            controller.dispatch_mutation(ChatStateMutation::SetBotId(Some(bot.id.clone())));
        } else {
            eprintln!("No models available, check your API keys.");
        }

        // ════════════════════════════════════════════════════════════════════════════════
        // Alternative: UI-level filtering with BotFilter trait
        // ════════════════════════════════════════════════════════════════════════════════
        // If you want to keep ALL bots in the controller but filter only in the UI:
        //
        // 1. Implement BotFilter trait:
        //    struct MyBotFilter { whitelist: Vec<String> }
        //    impl BotFilter for MyBotFilter {
        //        fn should_show(&self, bot: &Bot) -> bool {
        //            self.whitelist.contains(&bot.name)
        //        }
        //    }
        //
        // 2. Set filter on ModelSelectorList:
        //    let chat = self.chat(ids!(chat));
        //    let mut list = chat.read().prompt_input_ref()
        //        .widget(ids!(model_selector.options.list_container.list))
        //        .borrow_mut::<ModelSelectorList>().unwrap();
        //    list.filter = Some(Box::new(MyBotFilter { whitelist }));
        //
        // This approach is more flexible for dynamic filtering (e.g., user preferences).
        // By Keeping the bots in the controller, MolyKit can properly include the bot
        // names in the chat messages.
        // ════════════════════════════════════════════════════════════════════════════════
    }

    fn setup_chat_hooks(&self) {
        // self.chat(ids!(chat)).write_with(|chat| {
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

            if let Some(key) = OPEN_AI_REALTIME_KEY {
                let mut openai_realtime =
                    OpenAIRealtimeClient::new("wss://api.openai.com/v1/realtime".into());
                let _ = openai_realtime.set_key(key);
                client.add_client(Box::new(openai_realtime));
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
            .with_plugin_prepend(Plugin {
                ui: self.ui_runner(),
                initialized: false,
            })
            .build_arc();

        controller.lock().unwrap().dispatch_task(ChatTask::Load);

        self.controller = Some(controller.clone());
        self.chat(ids!(chat))
            .write()
            .set_chat_controller(cx, Some(controller));
    }
}

struct Plugin {
    ui: UiRunner<DemoChat>,
    initialized: bool,
}

impl ChatControllerPlugin for Plugin {
    fn on_state_ready(
        &mut self,
        state: &controllers::chat::ChatState,
        _mutations: &[controllers::chat::ChatStateMutation],
    ) {
        self.init(state);
    }
}

impl Plugin {
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
}
