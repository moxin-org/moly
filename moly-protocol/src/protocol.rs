use crate::data::*;
use crate::open_ai::*;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum FileDownloadResponse {
    Progress(FileID, f32),
    Completed(DownloadedFile),
}

#[derive(Clone, Debug)]
pub enum ContextOverflowPolicy {
    StopAtLimit,
    TruncateMiddle,
    TruncatePastMessages,
}

#[derive(Clone, Debug)]
pub enum GPULayers {
    Specific(u32),
    Max,
}

#[derive(Clone, Debug)]
pub struct LoadModelOptions {
    pub prompt_template: Option<String>,
    pub gpu_layers: GPULayers,
    pub use_mlock: bool,
    pub n_batch: Option<u32>,
    pub n_ctx: Option<u32>,
    pub rope_freq_scale: f32,
    pub rope_freq_base: f32,
    // TBD Not really sure if this is something backend manages or if it is matter of
    // the client (if it is done by tweaking the JSON payload for the chat completition)
    pub context_overflow_policy: ContextOverflowPolicy,
}

#[derive(Clone, Debug)]
pub struct LoadedModelInfo {
    pub file_id: FileID,
    pub model_id: ModelID,

    // The port where the local server is listening for the model.
    // if 0, the server is not running.
    pub listen_port: u16,

    // JSON formatted string with the model information. See "Model Inspector" in LMStudio.
    pub information: String,
}

#[derive(Clone, Debug)]
pub struct ModelResourcesInfo {
    pub ram_usage: f32,
    pub cpu_usage: f32,
}

#[derive(Clone, Debug)]
pub enum LoadModelResponse {
    Progress(FileID, f32),
    Completed(LoadedModelInfo),
    ModelResourcesUsage(ModelResourcesInfo),
}

#[derive(Clone, Debug)]
pub struct LocalServerConfig {
    pub port: u16,
    pub cors: bool,
    pub request_queuing: bool,
    pub verbose_server_logs: bool,
    pub apply_prompt_formatting: bool,
}

#[derive(Clone, Debug)]
pub enum LocalServerResponse {
    Started,
    Log(String),
}

#[derive(Clone, Debug)]
pub enum Command {
    GetFeaturedModels(Sender<Result<Vec<Model>>>),

    // Change DowanloadFiles Location
    ChangeModelsDir(PathBuf),

    // The argument is a string with the keywords to search for.
    SearchModels(String, Sender<Result<Vec<Model>>>),

    DownloadFile(FileID, Sender<Result<FileDownloadResponse>>),
    PauseDownload(FileID, Sender<Result<()>>),
    CancelDownload(FileID, Sender<Result<()>>),
    DeleteFile(FileID, Sender<Result<()>>),

    GetCurrentDownloads(Sender<Result<Vec<PendingDownload>>>),
    GetDownloadedFiles(Sender<Result<Vec<DownloadedFile>>>),

    LoadModel(FileID, LoadModelOptions, Sender<Result<LoadModelResponse>>),

    // Eject currently loaded model, if any is provided
    EjectModel(Sender<Result<()>>),

    Chat(ChatRequestData, Sender<Result<ChatResponse>>),
    StopChatCompletion(Sender<Result<()>>),

    // Command to start a local server to interact with chat models
    StartLocalServer(LocalServerConfig, Sender<Result<LocalServerResponse>>),
    // Command to stop the local server
    StopLocalServer(Sender<Result<()>>),
}
