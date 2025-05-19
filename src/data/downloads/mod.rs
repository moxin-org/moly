pub mod download;

use download::{Download, DownloadFileAction, DownloadState};
use makepad_widgets::Action;
use moly_kit::utils::asynchronous::spawn;
use moly_protocol::data::{
    DownloadedFile, File, FileID, Model, PendingDownload, PendingDownloadsStatus,
};
use std::collections::HashMap;

use crate::app::app_runner;

use super::moly_client::MolyClient;

#[derive(Debug)]
pub enum DownloadPendingNotification {
    DownloadedFile(File),
    DownloadErrored(File),
}
pub struct Downloads {
    pub moly_client: MolyClient,
    pub downloaded_files: Vec<DownloadedFile>,
    pub pending_downloads: Vec<PendingDownload>,
    pub current_downloads: HashMap<FileID, Download>,
    pub pending_notifications: Vec<DownloadPendingNotification>,
}

impl Downloads {
    pub fn new(moly_client: MolyClient) -> Self {
        Self {
            moly_client,
            downloaded_files: Vec::new(),
            pending_downloads: Vec::new(),
            current_downloads: HashMap::new(),
            pending_notifications: Vec::new(),
        }
    }

    pub fn load_downloaded_files(&mut self) {
        let moly_client = self.moly_client.clone();
        spawn(async move {
            let response = moly_client.get_downloaded_files().await;
            app_runner().defer(|app, _, _| {
                let me = &mut app.store.downloads;
                match response {
                    Ok(files) => {
                        me.downloaded_files = files;
                    }
                    Err(_err) => {
                        eprintln!(
                            "Failed to fetch downloaded files. Couldn't connect to MolyServer."
                        )
                    }
                }
            });
        });
    }

    pub fn load_pending_downloads(&mut self) {
        let moly_client = self.moly_client.clone();
        spawn(async move {
            let response = moly_client.get_current_downloads().await;

            app_runner().defer(|app, _, _| {
                let me = &mut app.store.downloads;
                match response {
                    Ok(files) => {
                        me.pending_downloads = files;

                        me.pending_downloads
                            .sort_by(|a, b| b.file.id.cmp(&a.file.id));

                        // There is a issue with the backend response where all pending
                        // downloads come with status `Paused` even if they are downloading.
                        me.pending_downloads.iter_mut().for_each(|d| {
                            if let Some(current) = me.current_downloads.get(&d.file.id) {
                                if current.is_initializing() {
                                    d.status = PendingDownloadsStatus::Initializing;
                                } else {
                                    d.status = PendingDownloadsStatus::Downloading;
                                }
                            }
                        });
                    }
                    Err(_err) => {
                        eprintln!(
                            "Failed to fetch pending downloads. Couldn't connect to MolyServer."
                        )
                    }
                }
            });
        });
    }

    pub fn download_file(&mut self, model: Model, file: File) {
        let mut current_progress = 0.0;

        if let Some(pending) = self
            .pending_downloads
            .iter_mut()
            .find(|d| d.file.id == file.id)
        {
            current_progress = pending.progress;
            pending.status = PendingDownloadsStatus::Initializing;
        } else {
            let pending_download = PendingDownload {
                file: file.clone(),
                model: model.clone(),
                progress: 0.0,
                status: PendingDownloadsStatus::Initializing,
            };
            self.pending_downloads.push(pending_download);
        }

        self.current_downloads.insert(
            file.id.clone(),
            Download::new(file, current_progress, self.moly_client.clone()),
        );
    }

    /// Get a known file. No matter it's status.
    pub fn get_file(&self, file_id: &FileID) -> Option<&File> {
        // Bet this should not be different things just because they have attached status specific data.

        self.downloaded_files
            .iter()
            .find(|f| f.file.id == *file_id)
            .map(|f| &f.file)
            .or_else(|| {
                self.pending_downloads
                    .iter()
                    .find(|d| d.file.id == *file_id)
                    .map(|d| &d.file)
            })
        // probably unnecessary
        // .or_else(|| self.current_downloads.get(file_id).map(|d| &d.file))
    }

    /// Get a known model. No matter the status of it's related file.
    pub fn get_model_by_file_id(&self, file_id: &FileID) -> Option<&Model> {
        self.downloaded_files
            .iter()
            .find(|f| f.file.id == *file_id)
            .map(|f| &f.model)
            .or_else(|| {
                self.pending_downloads
                    .iter()
                    .find(|d| d.file.id == *file_id)
                    .map(|d| &d.model)
            })
        // .or_else(|| self.current_downloads.get(file_id).map(|d| &d.model))
    }

    pub fn pause_download_file(&mut self, file_id: &FileID) {
        let Some(current_download) = self.current_downloads.get(file_id) else {
            return;
        };
        if current_download.is_initializing() {
            return;
        }

        let file_id = file_id.clone();
        let moly_client = self.moly_client.clone();
        spawn(async move {
            let response = moly_client.pause_download_file(file_id.clone()).await;

            match response {
                Ok(()) => {
                    app_runner().defer(move |app, _, _| {
                        let me = &mut app.store.downloads;
                        me.current_downloads.remove(&file_id);
                        me.pending_downloads.iter_mut().for_each(|d| {
                            if d.file.id == *file_id {
                                d.status = PendingDownloadsStatus::Paused;
                            }
                        });
                    });
                }
                Err(err) => eprintln!("Error pausing download: {:?}", err),
            }
        });
    }

    pub fn cancel_download_file(&mut self, file_id: &FileID) {
        if let Some(current_download) = self.current_downloads.get(file_id) {
            if current_download.is_initializing() {
                return;
            }
        };

        let file_id = file_id.clone();
        let moly_client = self.moly_client.clone();

        spawn(async move {
            let response = moly_client.cancel_download_file(file_id.clone()).await;
            app_runner().defer(move |app, _, _| {
                let me = &mut app.store.downloads;
                match response {
                    Ok(()) => {
                        me.current_downloads.remove(&file_id);
                        me.pending_downloads.retain(|d| d.file.id != *file_id);
                    }
                    Err(err) => eprintln!("Error cancelling download: {:?}", err),
                }
            });
        });
    }

    pub fn next_download_notification(&mut self) -> Option<DownloadPendingNotification> {
        self.pending_notifications.pop()
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

    pub fn handle_action(&mut self, action: &Action) {
        if let Some(action) = action.downcast_ref::<DownloadFileAction>() {
            if let Some(download) = self.current_downloads.get_mut(&action.file_id) {
                download.handle_action(action);
            }
        }
    }

    /// This function is invoked after handling a download file action. It updates the
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
                    DownloadState::Initializing(_) => {
                        pending.status = PendingDownloadsStatus::Initializing;
                    }
                    DownloadState::Downloading(_) => {
                        pending.status = PendingDownloadsStatus::Downloading;
                    }
                    DownloadState::Errored(_) => {
                        pending.status = PendingDownloadsStatus::Error;
                        if download.must_show_notification() {
                            self.pending_notifications.push(
                                DownloadPendingNotification::DownloadErrored(download.file.clone()),
                            );
                        }
                    }
                    DownloadState::Completed => {
                        if download.must_show_notification() {
                            self.pending_notifications.push(
                                DownloadPendingNotification::DownloadedFile(download.file.clone()),
                            );
                        }
                    }
                };
                pending.progress = download.get_progress();
            }

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
