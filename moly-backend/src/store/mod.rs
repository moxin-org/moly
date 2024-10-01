pub mod download_files;
pub mod models;
pub mod remote;

pub mod model_cards;

use std::path::Path;

use moly_protocol::data::FileID;

pub use remote::*;

pub fn get_all_download_file(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Vec<moly_protocol::data::DownloadedFile>> {
    let files = download_files::DownloadedFile::get_finished(&conn)?;
    let models = models::Model::get_all(&conn)?;

    let mut downloaded_files = Vec::with_capacity(files.len());

    for (_id, file) in files {
        let model = if let Some(model) = models.get(&file.model_id) {
            moly_protocol::data::Model {
                id: model.id.to_string(),
                name: model.name.clone(),
                summary: model.summary.clone(),
                size: model.size.clone(),
                requires: model.requires.clone(),
                architecture: model.architecture.clone(),
                released_at: model.released_at.clone(),
                files: vec![],
                author: moly_protocol::data::Author {
                    name: model.author.name.clone(),
                    url: model.author.url.clone(),
                    description: model.author.description.clone(),
                },
                like_count: model.like_count,
                download_count: model.download_count,
                metrics: Default::default(),
            }
        } else {
            moly_protocol::data::Model::default()
        };

        let downloaded_path = Path::new(&file.download_dir)
            .join(&file.model_id)
            .join(&file.name);

        let downloaded_path = downloaded_path.to_str().map(|s| s.to_string());

        let downloaded_file = moly_protocol::data::DownloadedFile {
            file: moly_protocol::data::File {
                id: file.id.to_string(),
                name: file.name,
                size: file.size,
                quantization: file.quantization,
                downloaded: true,
                downloaded_path,
                tags: file.tags,
                featured: false,
            },
            model,
            downloaded_at: file.downloaded_at,
            compatibility_guess: moly_protocol::data::CompatibilityGuess::PossiblySupported,
            information: String::new(),
        };

        downloaded_files.push(downloaded_file);
    }

    Ok(downloaded_files)
}

pub fn get_all_pending_downloads(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Vec<moly_protocol::data::PendingDownload>> {
    let files = download_files::DownloadedFile::get_pending(&conn)?;

    let models = models::Model::get_all(&conn)?;

    let mut result = Vec::with_capacity(files.len());

    for (_file_id, file) in files {
        let result_file = moly_protocol::data::File {
            id: file.id.to_string(),
            name: file.name.clone(),
            size: file.size.clone(),
            quantization: file.quantization.clone(),
            downloaded: false,
            downloaded_path: None,
            tags: file.tags.clone(),
            featured: file.featured,
        };

        let model = if let Some(model) = models.get(&file.model_id) {
            moly_protocol::data::Model {
                id: model.id.to_string(),
                name: model.name.clone(),
                summary: model.summary.clone(),
                size: model.size.clone(),
                requires: model.requires.clone(),
                architecture: model.architecture.clone(),
                released_at: model.released_at.clone(),
                files: vec![],
                author: moly_protocol::data::Author {
                    name: model.author.name.clone(),
                    url: model.author.url.clone(),
                    description: model.author.description.clone(),
                },
                like_count: model.like_count,
                download_count: model.download_count,
                metrics: Default::default(),
            }
        } else {
            moly_protocol::data::Model::default()
        };

        let file_path = Path::new(&file.download_dir)
            .join(&file.model_id)
            .join(&file.name);

        let downloaded = if let Ok(file_meta) = std::fs::metadata(file_path) {
            file_meta.len()
        } else {
            0
        };
        let progress = (downloaded as f64 / file.file_size as f64) * 100.0;

        let pending_download = moly_protocol::data::PendingDownload {
            file: result_file,
            model,
            progress,
            status: moly_protocol::data::PendingDownloadsStatus::Paused,
            //status: item.status.into(),
        };

        result.push(pending_download);
    }

    Ok(result)
}

pub fn remove_downloaded_file(models_dir: String, file_id: FileID) -> anyhow::Result<()> {
    let (model_id, file) = file_id
        .split_once("#")
        .ok_or_else(|| anyhow::anyhow!("Illegal file_id"))?;

    let filename = format!("{}/{}/{}", models_dir, model_id, file);

    log::info!("Removing file {}", filename);
    Ok(std::fs::remove_file(filename)?)
}
