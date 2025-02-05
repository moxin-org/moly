use anyhow::anyhow;
use makepad_widgets::Cx;
use moly_protocol::{
    data::FileID,
    protocol::{Command, LoadModelOptions, LoadModelResponse, LoadedModelInfo},
};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::data::moly_client::MolyClient;

/// Message emitted when the model loader status is updated.
#[derive(Debug)]
pub struct ModelLoaderStatusChanged;

/// All posible states in which the loader can be.
#[derive(Debug, Default, Clone)]
pub enum ModelLoaderStatus {
    #[default]
    Unloaded,
    Loading,
    Loaded(LoadedModelInfo),
    Failed,
}

#[derive(Default)]
struct ModelLoaderInner {
    status: ModelLoaderStatus,
    file_id: Option<FileID>,
}

/// Unit for handling the non-blocking loading of models across threads.
#[derive(Clone, Default)]
pub struct ModelLoader(Arc<Mutex<ModelLoaderInner>>);

impl ModelLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(
        &mut self,
        file_id: FileID,
        moly_client: MolyClient,
        override_port: Option<u16>,
    ) -> Result<(), anyhow::Error> {
        match self.status() {
            ModelLoaderStatus::Loading => {
                return Err(anyhow!("ModelLoader is already loading a model"));
            }
            ModelLoaderStatus::Loaded(_) => {
                if override_port.is_none() {
                    if let Some(prev_file_id) = self.file_id() {
                        if prev_file_id == file_id {
                            return Ok(());
                        }
                    }
                }
            }
            _ => {}
        };

        self.set_status(ModelLoaderStatus::Loading);
        self.set_file_id(Some(file_id.clone()));

        let options = LoadModelOptions {
            override_server_address: override_port.map(|port| format!("localhost:{}", port)),
            prompt_template: None,
            gpu_layers: moly_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moly_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            n_batch: None,
            n_ctx: None,
        };

        let (tx, rx) = channel();
        moly_client.load_model(file_id, options, tx);
        
        let result = if let Ok(response) = rx.recv() {
            match response {
                Ok(LoadModelResponse::Completed(info)) => {
                    self.set_status(ModelLoaderStatus::Loaded(info));
                    Ok(())
                }
                Ok(response) => {
                    self.set_status(ModelLoaderStatus::Failed);
                    Err(anyhow!("Unexpected response: {:?}", response))
                }
                Err(err) => {
                    self.set_status(ModelLoaderStatus::Failed);
                    Err(anyhow!(err))
                }
            }
        } else {
            self.set_status(ModelLoaderStatus::Failed);
            Err(anyhow!("Internal communication error"))
        };

        result
    }

    pub fn load_async(
        &mut self,
        file_id: FileID,
        moly_client: MolyClient,
        override_port: Option<u16>,
    ) {
        let mut self_clone = self.clone();
        thread::spawn(move || {
            if let Err(err) = self_clone.load(file_id, moly_client, override_port) {
                eprintln!("Error loading model: {}", err);
            }
        });
    }

    fn set_status(&mut self, status: ModelLoaderStatus) {
        self.0.lock().unwrap().status = status;
        Cx::post_action(ModelLoaderStatusChanged);
    }

    fn set_file_id(&mut self, file_id: Option<FileID>) {
        self.0.lock().unwrap().file_id = file_id;
    }

    pub fn file_id(&self) -> Option<FileID> {
        self.0.lock().unwrap().file_id.clone()
    }

    pub fn status(&self) -> ModelLoaderStatus {
        self.0.lock().unwrap().status.clone()
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Loaded(_))
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Loading)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Failed)
    }

    pub fn is_finished(&self) -> bool {
        self.is_loaded() || self.is_failed()
    }

    pub fn is_pending(&self) -> bool {
        !self.is_finished()
    }

    /// Get the file id of the model that is currently being loaded.
    /// Returns `None` if the model loader is not at a loading state.
    pub fn get_loading_file_id(&self) -> Option<FileID> {
        if self.is_loading() {
            return self.file_id();
        }

        None
    }
}
