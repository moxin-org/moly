use makepad_widgets::Cx;
use moly_protocol::data::*;
use moly_protocol::protocol::{Command, FileDownloadResponse};
use std::sync::mpsc::channel;
use std::thread;

use crate::data::moly_client::MolyClient;

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
    pub fn new(file: File, progress: f64, moly_client: MolyClient) -> Self {
        let mut download = Self {
            file: file,
            state: DownloadState::Initializing(progress),
            notification_pending: false,
        };

        download.start(moly_client);
        download
    }

    pub fn start(&mut self, moly_client: MolyClient) {
        let (tx, rx) = channel();

        // let cmd = Command::DownloadFile(self.file.id.clone(), tx);
        // backend.command_sender.send(cmd).unwrap();
        let file_id = self.file.id.clone();
        moly_client.download_file(self.file.clone(), tx);

        // TODO(Julian): rework progress tracking.
        // If we get a 202 response, we need to poll the status of the download on the
        // downloads/{file_id}/progress endpoint until it's done (moly_client.track_download_progress).

        // thread::spawn(move || loop {
        //     if let Ok(response) = rx.recv() {
        //         if let Ok(()) = response {
        //             moly_client.track_download_progress(file_id, tx);
        //         } else {
        //             // moly_client.cancel_download_file(file_id, tx);
        //             // break;
        //         }
        //     }
        // });
                        
        // thread::spawn(move || loop {
        //     let mut is_done = false;
        //     if let Ok(response) = rx.recv() {
        //         match response {
        //             Ok(response) => match response {
        //                 FileDownloadResponse::Completed(_completed) => {
        //                     is_done = true;
        //                     Cx::post_action(DownloadFileAction {
        //                         file_id: file_id.clone(),
        //                         kind: DownloadFileActionKind::StreamingDone,
        //                     });
        //                 }
        //                 FileDownloadResponse::Progress(_file, value) => {
        //                     Cx::post_action(DownloadFileAction {
        //                         file_id: file_id.clone(),
        //                         kind: DownloadFileActionKind::Progress(value as f64),
        //                     })
        //                 }
        //             },
        //             Err(err) => {
        //                 Cx::post_action(DownloadFileAction {
        //                     file_id: file_id.clone(),
        //                     kind: DownloadFileActionKind::Error,
        //                 });

        //                 eprintln!("Error downloading file: {:?}", err)
        //             }
        //         }
        //     } else {
        //         break;
        //     }

        //     if is_done {
        //         break;
        //     }
        // });
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
