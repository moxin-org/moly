use crossterm::{QueueableCommand, cursor, terminal};
use moly_kit::utils::vec::VecMutation;
use moly_kit::{OpenAIClient, controllers::chat::*, protocol::*};
use std::io::{Write, stdin, stdout};
use std::sync::mpsc::{Sender, channel};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    stdout()
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(0, 0))?
        .flush()?;

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

    let mut client = OpenAIClient::new(url.into());

    if !key.is_empty() {
        client.set_key(&key)?;
    }

    let bot_id = BotId::new(&model, "");
    let (tx, rx) = channel();

    let controller = ChatController::builder()
        .with_client(client)
        .with_plugin_append(Plugin::new(tx))
        .build_arc();

    loop {
        print!("> ");
        stdout().flush()?;
        let input = stdin().lines().next().unwrap().unwrap();

        if input.trim().is_empty() {
            continue;
        }

        if input.trim() == "/exit" {
            break;
        }

        stdout().queue(cursor::SavePosition)?.flush()?;

        let mut message = Message::default();
        message.from = EntityId::User;
        message.content.text = input;
        controller
            .lock()
            .unwrap()
            .dispatch_mutation(VecMutation::Push(message));

        controller
            .lock()
            .unwrap()
            .dispatch_task(ChatTask::Send(bot_id.clone()));

        while let Ok(event) = rx.recv() {
            match event {
                Event::Stream(snapshot) => {
                    stdout()
                        .queue(cursor::RestorePosition)?
                        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                    print!("{}", snapshot);
                    stdout().flush()?;
                }
                Event::End => {
                    println!();
                    break;
                }
            }
        }
    }

    Ok(())
}

enum Event {
    Stream(String),
    End,
}

struct Plugin {
    was_streaming: bool,
    tx: Sender<Event>,
}

impl Plugin {
    fn new(tx: Sender<Event>) -> Self {
        Self {
            was_streaming: false,
            tx,
        }
    }
}

impl ChatControllerPlugin for Plugin {
    fn on_state_ready(&mut self, state: &ChatState, _mutations: &[ChatStateMutation]) {
        if state.is_streaming {
            if let Some(message) = state.messages.last() {
                if message.from != EntityId::User {
                    self.tx
                        .send(Event::Stream(message.content.text.clone()))
                        .unwrap();
                }
            }
        }

        if self.was_streaming && !state.is_streaming {
            self.tx.send(Event::End).unwrap();
        }

        self.was_streaming = state.is_streaming;
    }
}
