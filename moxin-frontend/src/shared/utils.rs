use anyhow::{Context, Result};
use std::process::Command;

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

pub fn open_folder(path: &str) -> Result<()> {
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else {
        // Defaulting to Linux and other Unix-like OS,
        // this assumes that 'xdg-open' is available.
        Command::new("xdg-open").arg(path).spawn()
    };

    result.context(format!("Failed to open folder: {}", path))?;

    Ok(())
}
