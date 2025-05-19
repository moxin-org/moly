use crate::data::moly_client::MolyClient;
use futures::{channel::mpsc::unbounded, StreamExt};
use makepad_widgets::Cx;
use moly_kit::utils::asynchronous::spawn;
use moly_protocol::{data::*, protocol::FileDownloadResponse};
use std::sync::Arc;

#[derive(Debug)]
pub struct DownloadFileAction {
    pub file_id: FileID,
    kind: DownloadFileActionKind,
}

#[derive(Debug)]
enum DownloadFileActionKind {
    Progress(f64),
    Error,
    StreamingDone,
}

#[derive(Clone, Copy, Debug)]
pub enum DownloadState {
    Initializing(f64),
    Downloading(f64),
    Errored(f64),
    Completed,
}

#[derive(Debug)]
pub struct Download {
    pub file: File,
    pub state: DownloadState,
    pub notification_pending: bool,
}

impl Download {
    pub fn new(file: File, progress: f64, moly_client: Arc<MolyClient>) -> Self {
        let mut download = Self {
            file,
            state: DownloadState::Initializing(progress),
            notification_pending: false,
        };

        download.start(moly_client);
        download
    }

    pub fn start(&mut self, moly_client: Arc<MolyClient>) {
        let (tx, mut rx) = unbounded();
        let file_id = self.file.id.clone();
        let moly_client_clone = moly_client.clone();

        spawn(async move {
            if let Some(Ok(())) = rx.next().await {
                // Download started successfully, now track progress
                let (progress_tx, mut progress_rx) = unbounded();

                // Create a new task for tracking progress
                let yamcc = moly_client_clone.clone();
                let file_id_clone = file_id.clone();

                spawn(async move {
                    yamcc
                        .track_download_progress(file_id_clone, progress_tx)
                        .await
                });

                // Wait for progress updates
                loop {
                    match progress_rx.next().await {
                        Some(result) => match result {
                            Ok(response) => match response {
                                FileDownloadResponse::Completed(_completed) => {
                                    Cx::post_action(DownloadFileAction {
                                        file_id: file_id.clone(),
                                        kind: DownloadFileActionKind::StreamingDone,
                                    });
                                    break;
                                }
                                FileDownloadResponse::Progress(_file, value) => {
                                    Cx::post_action(DownloadFileAction {
                                        file_id: file_id.clone(),
                                        kind: DownloadFileActionKind::Progress(value as f64),
                                    })
                                }
                            },
                            Err(err) => {
                                Cx::post_action(DownloadFileAction {
                                    file_id: file_id.clone(),
                                    kind: DownloadFileActionKind::Error,
                                });
                                eprintln!("Error downloading file: {:?}", err);
                                break;
                            }
                        },
                        None => break,
                    }
                }
            } else {
                // Initial download request failed
                Cx::post_action(DownloadFileAction {
                    file_id: file_id.clone(),
                    kind: DownloadFileActionKind::Error,
                });
            }
        });

        // Start the download
        let file = self.file.clone();
        let moly_client_clone = moly_client.clone();
        spawn(async move {
            let result = moly_client_clone.download_file(file).await;
            tx.unbounded_send(result).unwrap();
        });
    }

    pub fn handle_action(&mut self, action: &DownloadFileAction) {
        match action.kind {
            DownloadFileActionKind::StreamingDone => {
                self.state = DownloadState::Completed;
                self.notification_pending = true;
            }
            DownloadFileActionKind::Progress(value) => {
                self.state = DownloadState::Downloading(value)
            }
            DownloadFileActionKind::Error => {
                let current_progress = self.get_progress();
                self.state = DownloadState::Errored(current_progress);
                self.notification_pending = true;
            }
        }
    }

    pub fn is_initializing(&self) -> bool {
        matches!(self.state, DownloadState::Initializing(..))
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, DownloadState::Completed)
    }

    pub fn get_progress(&self) -> f64 {
        match self.state {
            DownloadState::Initializing(progress) => progress,
            DownloadState::Downloading(progress) => progress,
            DownloadState::Errored(progress) => progress,
            DownloadState::Completed => 1.0,
        }
    }

    pub fn must_show_notification(&mut self) -> bool {
        if self.notification_pending {
            self.notification_pending = false;
            true
        } else {
            false
        }
    }
}
