use anyhow::anyhow;
use makepad_widgets::SignalToUI;
use moxin_protocol::{
    data::FileID,
    protocol::{Command, LoadModelOptions, LoadModelResponse},
};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

/// Immutable handleable error type for the model loader.
///
/// Actually, just a wrapper around `anyhow::Error` to support `Clone`.
#[derive(Debug, Clone)]
pub struct ModelLoaderError(Arc<anyhow::Error>);

impl std::error::Error for ModelLoaderError {}

impl std::fmt::Display for ModelLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModelLoaderError: {}", self.0)
    }
}

impl From<anyhow::Error> for ModelLoaderError {
    fn from(err: anyhow::Error) -> Self {
        Self(Arc::new(err))
    }
}

impl From<String> for ModelLoaderError {
    fn from(err: String) -> Self {
        Self(Arc::new(anyhow!(err)))
    }
}

impl From<&'static str> for ModelLoaderError {
    fn from(err: &'static str) -> Self {
        Self(Arc::new(anyhow!(err)))
    }
}

/// All posible states in which the loader can be.
#[derive(Debug, Default, Clone)]
pub enum ModelLoaderStatus {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Failed(#[allow(dead_code)] ModelLoaderError),
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

    pub fn load_blocking(
        &mut self,
        file_id: FileID,
        command_sender: Sender<Command>,
    ) -> Result<(), ModelLoaderError> {
        match self.status() {
            ModelLoaderStatus::Loading => {
                return Err(anyhow!("ModelLoader is already loading a model").into());
            }
            ModelLoaderStatus::Loaded => {
                if let Some(prev_file_id) = self.file_id() {
                    if prev_file_id == file_id {
                        return Ok(());
                    }
                }
            }
            _ => {}
        };

        self.set_status(ModelLoaderStatus::Loading);
        self.set_file_id(Some(file_id.clone()));

        let response = dispatch_load_command(command_sender, file_id.clone()).recv();

        let result = if let Ok(response) = response {
            match response {
                Ok(LoadModelResponse::Completed(_)) => {
                    self.set_status(ModelLoaderStatus::Loaded);
                    Ok(())
                }
                Ok(response) => {
                    let msg = format!("Unexpected response: {:?}", response);
                    let err = ModelLoaderError::from(msg);
                    self.set_status(ModelLoaderStatus::Failed(err.clone()));
                    Err(err)
                }
                Err(err) => {
                    let err = ModelLoaderError::from(err);
                    self.set_status(ModelLoaderStatus::Failed(err.clone()));
                    Err(err)
                }
            }
        } else {
            let err = ModelLoaderError::from("Internal communication error");
            self.set_status(ModelLoaderStatus::Failed(err.clone()));
            Err(err)
        };

        SignalToUI::set_ui_signal();
        result
    }

    pub fn load(&mut self, file_id: FileID, command_sender: Sender<Command>) {
        let mut self_clone = self.clone();
        thread::spawn(move || {
            if let Err(err) = self_clone.load_blocking(file_id, command_sender) {
                eprintln!("Error loading model: {}", err);
            }
        });
    }

    fn set_status(&mut self, status: ModelLoaderStatus) {
        self.0.lock().unwrap().status = status;
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
        matches!(self.status(), ModelLoaderStatus::Loaded)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Loading)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Failed(_))
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

fn dispatch_load_command(
    command_sender: Sender<Command>,
    file_id: String,
) -> Receiver<Result<LoadModelResponse, anyhow::Error>> {
    let (tx, rx) = channel();
    let cmd = Command::LoadModel(
        file_id,
        LoadModelOptions {
            prompt_template: None,
            gpu_layers: moxin_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            n_batch: None,
            n_ctx: None,
        },
        tx,
    );
    command_sender.send(cmd).unwrap();
    rx
}
