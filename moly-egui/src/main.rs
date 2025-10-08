use std::sync::{Arc, Mutex};

use moly_kit::{
    BotId, EntityId, Message, MessageContent, OpenAIClient,
    controllers::chat::{ChatController, ChatTask},
};

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native("app", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}

pub struct App {
    prompt: String,
    bot_id: BotId,
    controller: Arc<Mutex<ChatController>>,
}

impl App {
    fn new() -> Self {
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

        let controller = ChatController::builder().with_client(client).build_arc();

        Self {
            bot_id: BotId::new(&model, ""),
            prompt: String::new(),
            controller,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_height = ui.available_height();
            let input_height = 60.0; // Reserve space for the input area

            // Chat messages area - takes remaining space minus input area
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

            // Input area at the bottom
            ui.horizontal(|ui| {
                ui.text_edit_multiline(&mut self.prompt);
                if ui.button("Send").clicked() {
                    let prompt = std::mem::take(&mut self.prompt);
                    self.controller
                        .lock()
                        .unwrap()
                        .dispatch_state_mutation(|state| {
                            state.messages.push(Message {
                                from: EntityId::User,
                                content: MessageContent {
                                    text: prompt.clone(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        });

                    self.controller
                        .lock()
                        .unwrap()
                        .dispatch_task(ChatTask::Send(self.bot_id.clone()));
                }
            })
        });
    }
}
