use chrono::NaiveDate;

pub type FileID = String;
pub type ModelID = String;

#[derive(Debug, Clone, Default)]
pub struct File {
    pub id: FileID,
    pub name: String,
    pub size: String,
    pub quantization: String,
    pub downloaded: bool,
    pub downloaded_path: Option<String>,
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
    pub id: ModelID,
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