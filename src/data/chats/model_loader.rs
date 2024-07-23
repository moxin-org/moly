use std::{sync::mpsc::{channel, Receiver, Sender}, thread};
use anyhow::Result;
use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::{data::File, protocol::{Command, LoadModelOptions, LoadModelResponse}};

pub struct ModelLoader {
    pub complete: bool,
    pub file: File,
    load_sender: Sender<Result<()>>,
    load_receiver: Receiver<Result<()>>,
}

impl ModelLoader {
    pub fn new(file: File) -> Self {
        let (tx, rx) = channel();
        Self {
            complete: false,
            file,
            load_sender: tx,
            load_receiver: rx,
        }
    }

    pub fn load_model(&self, backend: &Backend) {
        let (tx, rx) = channel();
        let cmd = Command::LoadModel(
            self.file.id.clone(),
            LoadModelOptions {
                prompt_template: None,
                gpu_layers: moxin_protocol::protocol::GPULayers::Max,
                use_mlock: false,
                rope_freq_scale: 0.0,
                rope_freq_base: 0.0,
                context_overflow_policy:
                    moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            },
            tx,
        );

        let load_model_tx = self.load_sender.clone();

        backend.command_sender.send(cmd).unwrap();

        thread::spawn(move || {
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(LoadModelResponse::Completed(_)) => {
                        load_model_tx.send(Ok(())).unwrap();
                    }
                    Ok(_) => {
                        eprintln!("Error loading model: Unexpected response");
                        load_model_tx.send(
                            Err(anyhow::anyhow!("Error loading model: Unexpected response"))
                        ).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error loading model: {:?}", err);
                        load_model_tx.send(Err(err)).unwrap();
                    }
                }
            } else {
                load_model_tx.send(
                    Err(anyhow::anyhow!("Error loading model"))
                ).unwrap();
            }

            SignalToUI::set_ui_signal();
        });
    }

    pub fn check_load_response(&mut self) -> Result<()> {
        for msg in self.load_receiver.try_iter() {
            match msg {
                Ok(_) => {
                    self.complete = true;
                    return Ok(())
                }
                Err(err) => {
                    self.complete = true;
                    return Err(err.into())
                }
            }
        };

        Ok(())
    }
}