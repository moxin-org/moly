pub mod download_files;
pub mod models;
pub mod remote;

pub mod pending_downloads;

pub use remote::*;

pub fn get_all_download_file(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Vec<moxin_protocol::data::DownloadedFile>> {
    let files = download_files::DownloadedFile::get_finished(&conn)?;
    let models = models::Model::get_all(&conn)?;

    let mut downloaded_files = Vec::with_capacity(files.len());

    for (_id, file) in files {
        let model = if let Some(model) = models.get(&file.model_id) {
            moxin_protocol::data::Model {
                id: model.id.to_string(),
                name: model.name.clone(),
                summary: model.summary.clone(),
                size: model.size.clone(),
                requires: model.requires.clone(),
                architecture: model.architecture.clone(),
                released_at: model.released_at.clone(),
                files: vec![],
                author: moxin_protocol::data::Author {
                    name: model.author.name.clone(),
                    url: model.author.url.clone(),
                    description: model.author.description.clone(),
                },
                like_count: model.like_count,
                download_count: model.download_count,
                metrics: Default::default(),
            }
        } else {
            moxin_protocol::data::Model::default()
        };

        let downloaded_file = moxin_protocol::data::DownloadedFile {
            file: moxin_protocol::data::File {
                id: file.id.to_string(),
                name: file.name,
                size: file.size,
                quantization: file.quantization,
                downloaded: true,
                downloaded_path: file.downloaded_path,
                tags: file.tags,
                featured: false,
            },
            model,
            downloaded_at: file.downloaded_at.unwrap(),
            compatibility_guess: moxin_protocol::data::CompatibilityGuess::PossiblySupported,
            information: String::new(),
        };

        downloaded_files.push(downloaded_file);
    }

    Ok(downloaded_files)
}

pub fn get_all_pending_downloads(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Vec<moxin_protocol::data::PendingDownload>> {
    let pending_downloads = pending_downloads::PendingDownloads::get_all(&conn)?;
    let files = download_files::DownloadedFile::get_all(&conn)?;
    let models = models::Model::get_all(&conn)?;

    let mut result = Vec::with_capacity(pending_downloads.len());

    for item in pending_downloads {
        let Some(file) = files.get(&item.file_id) else {
            // TODO handle error
            continue;
        };
        let result_file = moxin_protocol::data::File {
            id: file.id.to_string(),
            name: file.name.clone(),
            size: file.size.clone(),
            quantization: file.quantization.clone(),
            downloaded: false,
            downloaded_path: None,
            tags: file.tags.clone(),
            featured: false,
        };

        let model = if let Some(model) = models.get(&file.model_id) {
            moxin_protocol::data::Model {
                id: model.id.to_string(),
                name: model.name.clone(),
                summary: model.summary.clone(),
                size: model.size.clone(),
                requires: model.requires.clone(),
                architecture: model.architecture.clone(),
                released_at: model.released_at.clone(),
                files: vec![],
                author: moxin_protocol::data::Author {
                    name: model.author.name.clone(),
                    url: model.author.url.clone(),
                    description: model.author.description.clone(),
                },
                like_count: model.like_count,
                download_count: model.download_count,
                metrics: Default::default(),
            }
        } else {
            moxin_protocol::data::Model::default()
        };

        let pending_download = moxin_protocol::data::PendingDownload {
            file: result_file,
            model,
            progress: item.progress,
            status: moxin_protocol::data::PendingDownloadsStatus::Downloading,
            //status: item.status.into(),
        };

        result.push(pending_download);
    }

    Ok(result)
}
