use crate::data::*;
use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub enum ContextOverflowPolicy {
    StopAtLimit,
    TruncateMiddle,
    TruncatePastMessages,
}

#[derive(Clone, Debug)]
pub struct ModelExecutionConfig {
    pub cpu_threads: u32,
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
pub struct LocalServerConfig {
    pub port: u16,
    pub cors: bool,
    pub request_queuing: bool,
    pub verbose_server_logs: bool,
    pub apply_prompt_formatting: bool,
}

#[derive(Clone, Debug)]
pub enum Command {
    GetFeaturedModels,

    // The argument is a string with the keywords to search for.
    SearchModels(String),

    DownloadFile(FileID),

    LoadModel(FileID),
    EjectModel(FileID),
    GetLoadedModel(),

    // The argument is the chat message in JSON format, following https://platform.openai.com/docs/api-reference/chat/create
    GetChatCompletion(String),

    // Command to stop the current chat completion
    StopChatCompletion,

    // Command to set global settings for the backend
    ConfigureModelExecution(ModelExecutionConfig),

    // Command to start a local server to interact with chat models
    StartLocalServer(LocalServerConfig),

    // Command to stop the local server
    StopLocalServer,

    GetDownloadedFiles,
}

#[derive(Clone, Debug)]
pub struct LoadedModelInfo {
    pub file_id: FileID,
    pub model_id: ModelID,

    // JSON formatted string with the model information. See "Model Inspector" in LMStudio.
    pub information: String,
}

#[derive(Clone, Debug)]
pub struct ModelSystemInfo {
    ram_usage: f32,
    cpu_usage: f32,
}

#[derive(Clone, Debug)]
pub enum StopReason {
    Completed,
    Stopped
}

#[derive(Clone, Debug)]
pub struct ChatCompletionData {
    // The response from the model in JSON format, following https://platform.openai.com/docs/api-reference/chat/create
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
pub enum CompatibilityGuess {
    PossiblySupported,
    NotSupported,
}

#[derive(Clone, Debug)]
pub struct DownloadedFile {
    pub file: File,
    pub model: Model,
    pub downloaded_at: NaiveDate,
    pub compatibility_guess: CompatibilityGuess,
}

#[derive(Clone, Debug)]
pub enum Response {
    // Response to the GetFeaturedModels command
    FeaturedModels(Vec<Model>),

    // Response to the SearchModels command
    ModelsSearchResults(Vec<Model>),

    // Responses related with the DownloadFile command
    FileDownloadProgress(FileID, f32), // Progress value from 0.0 to 1.0
    FileDownloaded(File), // The downloaded_path field is filled

    LoadModelProgress(FileID, f32), // Progress value from 0.0 to 1.0

    // Response to the GetLoadedModel command, but also sent when the model has been loaded
    LoadedModel(Option<LoadedModelInfo>),

    // This response is an update that is sent periodically to the client.
    // It is not a response to a specific command, but they will arrive when the
    // model is loaded and running.
    ModelSystemInfoUpdate(ModelSystemInfo),

    // Responses to the GetChatCompletion command.
    ChatCompletion(ChatCompletionData), // Final response. It also is emitten when the chat is stopped with the StopChatCompletion command
    ChatCompletionChunk(String), // Streamed chunk of the response in JSON format (following https://platform.openai.com/docs/api-reference/chat/create)

    // Response to the GetDownloadedFiles command
    DownloadFilesResults(Vec<DownloadedFile>),

    // Response to the StartLocalServer command
    LocalServerStarted,

    // Chunk of the local server log
    LocalServerLog(String),
}