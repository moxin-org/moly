use anyhow::Result;

pub const BYTES_PER_MB: f64 = 1_048_576.0; // (1024^2)
pub const HUGGING_FACE_BASE_URL: &str = "https://huggingface.co";

pub fn format_model_size(size: &str) -> Result<String> {
    let size_mb = size.parse::<f64>()? / BYTES_PER_MB;

    if size_mb >= 1024.0 {
        Ok(format!("{:.2} GB", size_mb / 1024.0))
    } else {
        Ok(format!("{:.2} MB", size_mb as i32))
    }
}

pub fn format_model_downloaded_size(size: &str, progress: f64) -> Result<String> {
    let size_mb = (size.parse::<f64>()? / BYTES_PER_MB) * progress / 100.0;

    if size_mb >= 1024.0 {
        Ok(format!("{:.2} GB", size_mb / 1024.0))
    } else {
        Ok(format!("{:.2} MB", size_mb as i32))
    }
}

pub fn hugging_face_model_url(model_id: &str) -> String {
    format!("{}/{}", HUGGING_FACE_BASE_URL, model_id)
}

/// Removes dashes, file extension, and capitalizes the first letter of each word.
pub fn human_readable_model_name(name: &str) -> String {
    let name = name
        .to_lowercase()
        .replace("-", " ")
        .replace("gguf", "")
        .replace(".gguf", "")
        .replace("chat", "");

    let name = name
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    name
}
