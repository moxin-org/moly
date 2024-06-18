pub mod download;

use anyhow::{Context, Result};
use download::{Download, DownloadState};
use moxin_backend::Backend;
use moxin_protocol::{
    data::{DownloadedFile, File, FileID, Model, PendingDownload, PendingDownloadsStatus},
    protocol::Command,
};
use std::{collections::HashMap, rc::Rc, sync::mpsc::channel};

pub enum DownloadPendingNotification {
    DownloadedFile(File),
    DownloadErrored(File),
}
pub struct Downloads {
    pub backend: Rc<Backend>,
    pub downloaded_files: Vec<DownloadedFile>,
    pub pending_downloads: Vec<PendingDownload>,
    pub current_downloads: HashMap<FileID, Download>,
}

impl Downloads {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            downloaded_files: Vec::new(),
            pending_downloads: Vec::new(),
            current_downloads: HashMap::new(),
        }
    }

    pub fn load_downloaded_files(&mut self) {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::GetDownloadedFiles(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(files) => {
                    self.downloaded_files = files;
                }
                Err(err) => eprintln!("Error fetching downloaded files: {:?}", err),
            }
        };
    }

    pub fn load_pending_downloads(&mut self) {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::GetCurrentDownloads(tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(files) => {
                    self.pending_downloads = files;

                    self.pending_downloads.sort_by(|a, b| b.file.id.cmp(&a.file.id));

                    // There is a issue with the backend response where all pending
                    // downloads come with status `Paused` even if they are downloading.
                    self.pending_downloads.iter_mut().for_each(|d| {
                        if self.current_downloads.contains_key(&d.file.id) {
                            d.status = PendingDownloadsStatus::Downloading;
                        }
                    });
                }
                Err(err) => eprintln!("Error fetching pending downloads: {:?}", err),
            }
        };
    }

    pub fn download_file(&mut self, model: Model, file: File) {
        let mut current_progress = 0.0;

        if let Some(pending) = self
            .pending_downloads
            .iter_mut()
            .find(|d| d.file.id == file.id)
        {
            current_progress = pending.progress;
            pending.status = PendingDownloadsStatus::Downloading;
        } else {
            let pending_download = PendingDownload {
                file: file.clone(),
                model: model.clone(),
                progress: 0.0,
                status: PendingDownloadsStatus::Downloading,
            };
            self.pending_downloads.push(pending_download);
        }

        self.current_downloads.insert(
            file.id.clone(),
            Download::new(file, model, current_progress, &self.backend.as_ref()),
        );
    }

    pub fn pause_download_file(&mut self, file_id: FileID) {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::PauseDownload(file_id.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.current_downloads.remove(&file_id);
                    self.load_pending_downloads();
                }
                Err(err) => eprintln!("Error pausing download: {:?}", err),
            }
        };
    }

    pub fn cancel_download_file(&mut self, file_id: FileID) {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::CancelDownload(file_id.clone(), tx))
            .unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.current_downloads.remove(&file_id);
                    self.load_pending_downloads();
                }
                Err(err) => eprintln!("Error cancelling download: {:?}", err),
            }
        };
    }

    pub fn delete_file(&mut self, file_id: FileID) -> Result<()> {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::DeleteFile(file_id.clone(), tx))
            .context("Failed to send delete file command")?;

        rx.recv()
            .context("Failed to receive delete file response")?
            .context("Delete file operation failed")?;

        
        self.load_downloaded_files();
        self.load_pending_downloads();
        Ok(())
    }

    pub fn next_download_notification(&mut self) -> Option<DownloadPendingNotification> {
        self.current_downloads
            .iter_mut()
            .filter_map(|(_, download)| {
                if download.must_show_notification() {
                    if download.is_errored() {
                        return Some(DownloadPendingNotification::DownloadErrored(
                            download.file.clone(),
                        ));
                    } else if download.is_complete() {
                        return Some(DownloadPendingNotification::DownloadedFile(
                            download.file.clone(),
                        ));
                    } else {
                        return None;
                    }
                }
                None
            })
            .next()
    }

    pub fn get_model_and_file_for_pending_download(&self, file_id: &str) -> Option<(Model, File)> {
        self.pending_downloads.iter().find_map(|d| {
            if d.file.id == file_id {
                Some((d.model.clone(), d.file.clone()))
            } else {
                None
            }
        })
    }

    /// This function is invoked when the Makepad signal is received. It updates the
    /// download progress and state of the downloads, based in the active downloads
    /// but also retrieving fresh data from the backend.
    pub fn refresh_downloads_data(&mut self) -> Vec<FileID> {
        let mut completed_download_ids = Vec::new();

        for (id, download) in &mut self.current_downloads {
            if let Some(pending) = self
                .pending_downloads
                .iter_mut()
                .find(|d| d.file.id == id.to_string())
            {
                match download.state {
                    DownloadState::Downloading(_) => {
                        pending.status = PendingDownloadsStatus::Downloading
                    }
                    DownloadState::Errored(_) => pending.status = PendingDownloadsStatus::Error,
                    DownloadState::Completed => (),
                };
                pending.progress = download.get_progress();
            }

            download.process_download_progress();
            if download.is_complete() {
                completed_download_ids.push(id.clone());
            }
        }

        for id in &completed_download_ids {
            self.current_downloads.remove(id);
        }

        // Reload downloaded files and pending downloads from the backend
        if !completed_download_ids.is_empty() {
            self.load_downloaded_files();
            self.load_pending_downloads();
        }

        completed_download_ids
    }
}
