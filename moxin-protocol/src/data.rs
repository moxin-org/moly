use chrono::NaiveDate;

#[derive(Debug, Clone, Default)]
pub struct File {
    pub path: String,
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
}