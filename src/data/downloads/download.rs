use makepad_widgets::Cx;
use moly_backend::Backend;
use moly_protocol::data::*;
use moly_protocol::protocol::{Command, FileDownloadResponse};
use std::sync::mpsc::channel;
use std::thread;

#[derive(Debug)]
pub struct DownloadFileAction {
    pub id: FileID,
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
    pub fn new(file: File, progress: f64, backend: &Backend) -> Self {
        let mut download = Self {
            file: file,
            state: DownloadState::Initializing(progress),
            notification_pending: false,
        };

        download.start(backend);
        download
    }

    pub fn start(&mut self, backend: &Backend) {
        let (tx, rx) = channel();

        let cmd = Command::DownloadFile(self.file.id.clone(), tx);
        backend.command_sender.send(cmd).unwrap();
        let file_id = self.file.id.clone();

        thread::spawn(move || loop {
            let mut is_done = false;
            if let Ok(response) = rx.recv() {
                match response {
                    Ok(response) => match response {
                        FileDownloadResponse::Completed(_completed) => {
                            is_done = true;
                            Cx::post_action(DownloadFileAction {
                                id: file_id.clone(),
                                kind: DownloadFileActionKind::StreamingDone,
                            });
                        }
                        FileDownloadResponse::Progress(_file, value) => {
                            Cx::post_action(DownloadFileAction {
                                id: file_id.clone(),
                                kind: DownloadFileActionKind::Progress(value as f64),
                            })
                        }
                    },
                    Err(err) => {
                        Cx::post_action(DownloadFileAction {
                            id: file_id.clone(),
                            kind: DownloadFileActionKind::Error,
                        });

                        eprintln!("Error downloading file: {:?}", err)
                    }
                }
            } else {
                break;
            }

            if is_done {
                break;
            }
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
