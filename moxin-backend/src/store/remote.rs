use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use moxin_protocol::data::Model;
use moxin_protocol::protocol::FileDownloadResponse;

use super::pending_downloads::{PendingDownloads, PendingDownloadsStatus};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RemoteFile {
    pub name: String,
    pub size: String,
    pub quantization: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RemoteModel {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    pub files: Vec<RemoteFile>,
    pub prompt_template: String,
    pub reverse_prompt: String,
    pub author: Author,
    pub like_count: u32,
    pub download_count: u32,
    #[serde(default)]
    pub metrics: Option<HashMap<String, f32>>,
}

impl RemoteModel {
    pub fn search(search_text: &str, limit: usize, offset: usize) -> reqwest::Result<Vec<Self>> {
        let url = format!("https://code.flows.network/webhook/DsbnEK45sK3NUzFUyZ9C/models?status=published&trace_status=tracing&order=most_likes&offset={offset}&limit={limit}&search={search_text}");
        let response = reqwest::blocking::get(&url)?;
        response.json()
    }

    pub fn get_featured_model(limit: usize, offset: usize) -> reqwest::Result<Vec<Self>> {
        let url = format!("https://code.flows.network/webhook/DsbnEK45sK3NUzFUyZ9C/models?status=published&trace_status=tracing&order=most_likes&offset={offset}&limit={limit}&featured=featured");
        let response = reqwest::blocking::get(&url)?;
        response.json()
    }

    pub fn to_model(
        remote_models: &[Self],
        conn: &rusqlite::Connection,
    ) -> rusqlite::Result<Vec<moxin_protocol::data::Model>> {
        let model_ids = remote_models
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>();
        let files = super::download_files::DownloadedFile::get_by_models(conn, &model_ids)?;

        fn to_file(
            model_id: &str,
            remote_files: &[RemoteFile],
            save_files: &HashMap<Arc<String>, super::download_files::DownloadedFile>,
        ) -> rusqlite::Result<Vec<moxin_protocol::data::File>> {
            let mut files = vec![];
            for remote_f in remote_files {
                let file_id = format!("{}#{}", model_id, remote_f.name);
                let downloaded_path = save_files
                    .get(&file_id)
                    .map(|f| f.downloaded_path.clone())
                    .unwrap_or(None);

                let file = moxin_protocol::data::File {
                    id: file_id,
                    name: remote_f.name.clone(),
                    size: remote_f.size.clone(),
                    quantization: remote_f.quantization.clone(),
                    downloaded: downloaded_path.is_some(),
                    downloaded_path,
                    tags: remote_f.tags.clone(),
                    featured: false,
                };

                files.push(file);
            }

            Ok(files)
        }

        let mut models = Vec::with_capacity(remote_models.len());

        for remote_m in remote_models {
            let model = Model {
                id: remote_m.id.clone(),
                name: remote_m.name.clone(),
                summary: remote_m.summary.clone(),
                size: remote_m.size.clone(),
                requires: remote_m.requires.clone(),
                architecture: remote_m.architecture.clone(),
                released_at: remote_m.released_at.clone(),
                files: to_file(&remote_m.id, &remote_m.files, &files)?,
                author: moxin_protocol::data::Author {
                    name: remote_m.author.name.clone(),
                    url: remote_m.author.url.clone(),
                    description: remote_m.author.description.clone(),
                },
                like_count: remote_m.like_count.clone(),
                download_count: remote_m.download_count.clone(),
                metrics: remote_m.metrics.clone().unwrap_or_default(),
            };

            models.push(model);
        }

        Ok(models)
    }
}

fn get_file_content_length(client: &reqwest::blocking::Client, url: &str) -> reqwest::Result<u64> {
    let response = client.head(url).send()?;

    let content_length = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0);

    Ok(content_length)
}

fn download_file(
    client: &reqwest::blocking::Client,
    content_length: u64,
    url: &str,
    local_path: &str,
    step: f64,
    report_fn: &mut dyn FnMut(f64),
) -> io::Result<f64> {
    use std::path::Path;
    let path: &Path = local_path.as_ref();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = File::options()
        .write(true)
        .create(true)
        .open(local_path)
        .unwrap();
    let file_length = file.metadata()?.len();

    if file_length < content_length {
        file.seek(io::SeekFrom::End(0))?;

        let range = format!("bytes={}-", file_length);
        let mut resp = client
            .get(url)
            .header("Range", range)
            .send()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut buffer = vec![0; (content_length as usize) / 100];
        let mut downloaded: u64 = file_length;
        let mut last_progress = 0.0;
        loop {
            let len = match resp.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(len) => len,
                Err(e) => return Err(e),
            };
            file.write_all(&buffer[..len])?;
            downloaded += len as u64;

            let progress = (downloaded as f64 / content_length as f64) * 100.0;
            if progress > last_progress + step {
                last_progress = progress;
                report_fn(progress)
            }
        }
        Ok((downloaded as f64 / content_length as f64) * 100.0)
    } else {
        Ok(100.0)
    }
}

pub fn download_file_from_remote(
    client: &reqwest::blocking::Client,
    model_id: &str,
    file: &str,
    local_path: &str,
    step: f64,
    report_fn: &mut dyn FnMut(f64),
) -> io::Result<f64> {
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}?download=true",
        model_id, file
    );

    let content_length = get_file_content_length(client, &url)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    download_file(&client, content_length, &url, local_path, step, report_fn)
}

pub fn download_file_loop(
    sql_conn: Arc<Mutex<rusqlite::Connection>>,
    rx: Arc<
        crossbeam::channel::Receiver<(
            super::models::Model,
            super::download_files::DownloadedFile,
            Sender<anyhow::Result<FileDownloadResponse>>,
        )>,
    >,
) {
    let client = reqwest::blocking::Client::new();

    while let Ok((model, mut file, tx)) = rx.recv() {
        let file_id = file.id.clone();
        let conn = sql_conn.lock().unwrap();

        let mut send_progress = |progress| {
            let _ = tx.send(Ok(FileDownloadResponse::Progress(
                file_id.as_ref().clone(),
                progress as f32,
            )));

            // Update our local database
            let pending_download = PendingDownloads {
                file_id: file_id.clone(),
                progress: progress,
                status: PendingDownloadsStatus::Downloading,
            };
            pending_download.save_to_db(&conn).unwrap();
        };

        let _ = PendingDownloads::insert_if_not_exists(file_id.clone(), &conn);
        let _ = file.insert_into_db(&conn);
        // TODO rename to insert_if_not_exists or update model
        let _ = model.save_to_db(&conn);

        let r = download_file_from_remote(
            &client,
            &model.id,
            &file.name,
            &file.downloaded_path.clone().unwrap(),
            0.5,
            &mut send_progress,
        );

        match r {
            Ok(_) => {
                file.downloaded_at = Some(Utc::now());
                let _ = file.save_to_db(&conn);
                let _ = PendingDownloads::mark_as_downloaded(file_id.clone(), &conn);

                let _ = tx.send(Ok(FileDownloadResponse::Completed(
                    moxin_protocol::data::DownloadedFile {
                        file: moxin_protocol::data::File {
                            id: file_id.as_ref().clone(),
                            name: file.name.clone(),
                            size: file.size.clone(),
                            quantization: file.quantization.clone(),
                            downloaded: true,
                            downloaded_path: file.downloaded_path,
                            tags: file.tags,
                            featured: false,
                        },
                        model: Model::default(),
                        downloaded_at: file.downloaded_at.unwrap(),
                        compatibility_guess:
                            moxin_protocol::data::CompatibilityGuess::PossiblySupported,
                        information: String::new(),
                    },
                )));
            }
            Err(e) => tx
                .send(Err(anyhow::anyhow!("Download failed: {e}")))
                .unwrap(),
        }
    }
}

#[test]
fn test_download_file_from_huggingface() {
    let client = reqwest::blocking::Client::new();
    download_file_from_remote(
        &client,
        "TheBloke/Llama-2-7B-Chat-GGUF",
        "llama-2-7b-chat.Q3_K_M.gguf",
        "/home/csh/ai/models/TheBloke/Llama-2-7B-Chat-GGUF/llama-2-7b-chat.Q3_K_M.gguf",
        0.5,
        &mut |progress| {
            println!("Download progress: {:.2}%", progress);
        },
    )
    .unwrap();
}

#[test]
fn test_search() {
    let models = RemoteModel::search("llama", 100, 0).unwrap();
    println!("{:?}", models);
}
