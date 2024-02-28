use crate::data::*;
use std::sync::mpsc::Sender;
use chrono::NaiveDate;
use anyhow::Result;

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
    pub n_batch: u32,
    pub n_ctx: u32,
    pub rope_freq_scale: f32,
    pub rope_freq_base: f32,

    // TBD Not really sure if this is something backend manages or if it is matter of
    // the client (if it is done by tweaking the JSON payload for the chat completition)
    pub context_overflow_policy: ContextOverflowPolicy
}

#[derive(Clone, Debug)]
pub struct LoadedModelInfo {
    pub file_id: FileID,
    pub model_id: ModelID,

    // JSON formatted string with the model information. See "Model Inspector" in LMStudio.
    pub information: String,
}

#[derive(Clone, Debug)]
pub struct ModelResourcesInfo {
    ram_usage: f32,
    cpu_usage: f32,
}

#[derive(Clone, Debug)]
pub enum LoadModelResponse {
    Progress(FileID, f32),
    Completed(LoadedModelInfo),
    ModelResoucesUsage(ModelResourcesInfo),
}

pub enum ChatError {
    BackendNotRun,
    EndOfSequence,
    ContextFull,
    PromptTooLong,
    TooLarge,
    InvalidEncoding,
    Other,
}

#[derive(Clone, Debug)]
pub enum StopReason {
    Completed,
    Stopped
}

#[derive(Clone, Debug)]
pub struct ChatCompletionData {
    // The response from the model in JSON format
    // https://platform.openai.com/docs/api-reference/chat/object
    response: String,

    // The remaining fields are stats about the chat completion process
    time_to_first_token: f32,
    time_to_generate: f32,
    speed: f32,
    gpu_layers: u32,
    cpu_threads: u32,
    mlock: bool,
    token_count: u32,
    token_limit: u32,
    stop_reason: StopReason,
}

#[derive(Clone, Debug)]
pub enum ChatResponse {
    ChatCompletion(ChatCompletionData),
    // https://platform.openai.com/docs/api-reference/chat/streaming
    ChatCompletionChunk(String),
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

    // The argument is a string with the keywords to search for.
    SearchModels(String, Sender<Result<Vec<Model>>>),

    DownloadFile(FileID, Sender<Result<FileDownloadResponse>>),
    GetDownloadedFiles(Sender<Result<Vec<DownloadedFile>>>),

    LoadModel(FileID, LoadModelOptions, Sender<Result<LoadModelResponse>>),

    // Eject currently loaded model, if any is provided
    EjectModel(Sender<Result<()>>),

    // The argument is the chat message in JSON format, following https://platform.openai.com/docs/api-reference/chat/create
    Chat(String, Sender<Result<ChatResponse, ChatError>>),
    // Command to stop the current chat completion
    StopChatCompletion(Sender<Result<()>>),

    // Command to start a local server to interact with chat models
    StartLocalServer(LocalServerConfig, Sender<Result<LocalServerResponse>>),
    // Command to stop the local server
    StopLocalServer(Sender<Result<()>>),
}