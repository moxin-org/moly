use std::sync::{Arc, Mutex};

use moly_kit::{
    BotId, EntityId, Message, OpenAIClient,
    controllers::chat::{
        ChatController, ChatControllerPlugin, ChatState, ChatStateMutation, ChatTask,
    },
    utils::vec::VecMutation,
};

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "app",
        options,
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}

pub struct App {
    prompt: String,
    bot_id: BotId,
    controller: Arc<Mutex<ChatController>>,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        let url = std::env::var("API_URL").unwrap_or_default();
        let key = std::env::var("API_KEY").unwrap_or_default();
        let model = std::env::var("MODEL_ID").unwrap_or_default();

        println!(
            "Using url: {}",
            if url.is_empty() { "(empty)" } else { &url }
        );

        println!(
            "Using key: {}",
            if key.is_empty() { "(empty)" } else { "****" }
        );

        println!(
            "Using model: {}",
            if model.is_empty() { "(empty)" } else { &model }
        );

        let mut client = OpenAIClient::new(url);
        client.set_key(&key).unwrap();

        let plugin = Plugin::new(ctx.clone());

        let controller = ChatController::builder()
            .with_client(client)
            .with_plugin_append(plugin)
            .build_arc();

        Self {
            bot_id: BotId::new(&model, ""),
            prompt: String::new(),
            controller,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_height = ui.available_height();
            let input_height = 60.0;

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(available_height - input_height)
                .show(ui, |ui| {
                    for message in self.controller.lock().unwrap().state().messages.iter() {
                        ui.label(format!(
                            "{}: {}",
                            match message.from {
                                EntityId::User => "User",
                                EntityId::Bot(_) => "Bot",
                                _ => "Unknown",
                            },
                            message.content.text
                        ));
                    }
                });

            ui.horizontal(|ui| {
                ui.text_edit_multiline(&mut self.prompt);
                if ui.button("Send").clicked() {
                    let prompt = std::mem::take(&mut self.prompt);
                    let mut controller = self.controller.lock().unwrap();

                    let mut message = Message::default();
                    message.from = EntityId::User;
                    message.content.text = prompt;

                    controller.dispatch_mutation(VecMutation::Push(message));
                    controller.dispatch_task(ChatTask::Send(self.bot_id.clone()));
                }
            })
        });
    }
}

struct Plugin {
    egui_ctx: egui::Context,
}

impl Plugin {
    pub fn new(egui_ctx: egui::Context) -> Self {
        Self { egui_ctx }
    }
}

impl ChatControllerPlugin for Plugin {
    fn on_state_ready(&mut self, _state: &ChatState, _mutations: &[ChatStateMutation]) {
        self.egui_ctx.request_repaint();
    }
}
