use anyhow::Result;

pub const BYTES_PER_MB: f64 = 1_048_576.0; // (1024^2)

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
