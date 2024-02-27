use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatBody {
    pub messages: Vec<Message>,
    pub channel_id: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Options {
    pub ctx_size: Option<u64>,
    pub n_predict: Option<u64>,
    pub n_gpu_layers: Option<u64>,
    pub batch_size: Option<u64>,
    pub temp: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub reverse_prompt: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoadModel {
    pub model: String,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub options: Options,
}

#[derive(Debug)]
pub enum TokenError {
    BackendNotRun,
    EndOfSequence,
    ContextFull,
    PromptTooLong,
    TooLarge,
    InvalidEncoding,
    Other,
}

pub struct Token {
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct File {
    pub name: String,
    pub size: String,
    pub quantization: String,
    pub downloaded: bool,
    pub tags: Vec<String>,
    pub featured: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub description: String,
}

// We're using the HuggingFace identifier as the model ID for now
// We should consider using a different identifier in the future if more
// models sources are added.
#[derive(Debug, Clone, Default)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: NaiveDate,
    pub files: Vec<File>,
    pub author: Author,
    pub like_count: u32,
    pub download_count: u32,
}
