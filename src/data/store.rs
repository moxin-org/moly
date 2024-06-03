use super::chat::ChatID;
use super::download::DownloadState;
use super::filesystem::{project_dirs, setup_model_downloads_folder};
use super::preferences::Preferences;
use super::search::SortCriteria;
use super::{chat::Chat, download::Download, search::Search};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use makepad_widgets::{DefaultNone, SignalToUI};
use moxin_backend::Backend;
use moxin_protocol::data::{
    Author, DownloadedFile, File, FileID, Model, ModelID, PendingDownload, PendingDownloadsStatus,
};
use moxin_protocol::protocol::{Command, LoadModelOptions, LoadModelResponse};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap, path::PathBuf, sync::mpsc::channel};

pub const DEFAULT_MAX_DOWNLOAD_THREADS: usize = 3;

#[derive(Clone, DefaultNone, Debug)]
pub enum StoreAction {
    Search(String),
    ResetSearch,
    Sort(SortCriteria),
    None,
}

#[derive(Clone, Debug)]
pub struct FileWithDownloadInfo {
    pub file: File,
    pub download: Option<PendingDownload>,
    pub is_current_chat: bool,
}

#[derive(Clone, Debug)]
pub struct ModelWithDownloadInfo {
    pub model_id: ModelID,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    pub author: Author,
    pub like_count: u32,
    pub download_count: u32,
    pub files: Vec<FileWithDownloadInfo>,
}

pub enum DownloadPendingNotification {
    DownloadedFile(File),
    DownloadErrored(File),
}

#[derive(Default)]
pub struct Store {
    /// This is the backend representation, including the sender and receiver ends of the channels to
    /// communicate with the backend thread.
    pub backend: Rc<Backend>,

    pub search: Search,

    /// Local cache of search results and downloaded files
    //pub models: Vec<Model>,
    pub downloaded_files: Vec<DownloadedFile>,
    pub pending_downloads: Vec<PendingDownload>,

    /// Locally saved chats
    pub saved_chats: Vec<RefCell<Chat>>,
    pub current_chat_id: Option<ChatID>,
    pub current_downloads: HashMap<FileID, Download>,

    pub preferences: Preferences,
    pub downloaded_files_dir: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let downloaded_files_dir = setup_model_downloads_folder();
        let app_data_dir = project_dirs().data_dir();

        let backend = Rc::new(Backend::new(
            app_data_dir,
            downloaded_files_dir.clone(),
            DEFAULT_MAX_DOWNLOAD_THREADS,
        ));

        let mut store = Self {
            backend: backend.clone(),

            // TODO we should unify those two into a single struct
            downloaded_files: vec![],
            pending_downloads: vec![],

            search: Search::new(backend),
            saved_chats: vec![],
            current_chat_id: None,
            current_downloads: HashMap::new(),

            preferences: Preferences::load(),
            downloaded_files_dir,
        };
        store.load_downloaded_files();
        store.load_pending_downloads();

        store.search.load_featured_models();
        store
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // Functions related with models search                                          //
    ///////////////////////////////////////////////////////////////////////////////////

    /// This function combines the search results information for a given model
    /// with the download information for the files of that model.
    pub fn add_download_info_to_model(&self, model: &Model) -> ModelWithDownloadInfo {
        let files = model
            .files
            .iter()
            .map(|file| {
                let download = self
                    .pending_downloads
                    .iter()
                    .find(|d| d.file.id == file.id)
                    .cloned();
                let is_current_chat = self
                    .get_current_chat()
                    .map_or(false, |c| c.borrow().file_id == file.id);

                FileWithDownloadInfo {
                    file: file.clone(),
                    download,
                    is_current_chat,
                }
            })
            .collect();

        ModelWithDownloadInfo {
            model_id: model.id.clone(),
            name: model.name.clone(),
            summary: model.summary.clone(),
            size: model.size.clone(),
            requires: model.requires.clone(),
            architecture: model.architecture.clone(),
            like_count: model.like_count,
            download_count: model.download_count,
            released_at: model.released_at,
            author: model.author.clone(),
            files: files,
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // Functions related with model file downloads                                   //
    ///////////////////////////////////////////////////////////////////////////////////

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

    pub fn download_file(&mut self, file_id: FileID) {
        let (model, file) = self.get_model_and_file_download(&file_id);
        let mut current_progress = 0.0;

        if let Some(pending) = self
            .pending_downloads
            .iter_mut()
            .find(|d| d.file.id == file_id)
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
            file_id.clone(),
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

        self.search
            .update_downloaded_file_in_search_results(&file_id, false);
        self.load_downloaded_files();
        self.load_pending_downloads();
        SignalToUI::set_ui_signal();
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

    fn get_model_and_file_download(&self, file_id: &str) -> (Model, File) {
        if let Some(result) = self.get_model_and_file_for_pending_download(file_id) {
            result
        } else {
            self.search
                .get_model_and_file_from_search_results(file_id)
                .unwrap()
        }
    }

    fn get_model_and_file_for_pending_download(&self, file_id: &str) -> Option<(Model, File)> {
        self.pending_downloads.iter().find_map(|d| {
            if d.file.id == file_id {
                Some((d.model.clone(), d.file.clone()))
            } else {
                None
            }
        })
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // Functions related with model's chat                                           //
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn load_model(&mut self, file: &File) {
        let (tx, rx) = channel();
        let cmd = Command::LoadModel(
            file.id.clone(),
            LoadModelOptions {
                prompt_template: None,
                gpu_layers: moxin_protocol::protocol::GPULayers::Max,
                use_mlock: false,
                n_batch: 512,
                n_ctx: 512,
                rope_freq_scale: 0.0,
                rope_freq_base: 0.0,
                context_overflow_policy:
                    moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            },
            tx,
        );

        self.backend.as_ref().command_sender.send(cmd).unwrap();

        if let Ok(response) = rx.recv() {
            match response {
                Ok(response) => {
                    let LoadModelResponse::Completed(_) = response else {
                        eprintln!("Error loading model");
                        return;
                    };
                    // TODO: Creating a new chat, maybe put in a method and save on disk or smth.
                    let new_chat = RefCell::new(Chat::new(file.name.clone(), file.id.clone()));
                    self.current_chat_id = Some(new_chat.borrow().id);
                    self.saved_chats.push(new_chat);

                    self.preferences.set_current_chat_model(file.id.clone());
                }
                Err(err) => eprintln!("Error loading model: {:?}", err),
            }
        };
    }

    pub fn get_current_chat(&self) -> Option<&RefCell<Chat>> {
        if let Some(current_chat_id) = self.current_chat_id {
            self.saved_chats
                .iter()
                .find(|c| c.borrow().id == current_chat_id)
        } else {
            None
        }
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut()
                .send_message_to_model(prompt, &self.backend.as_ref());
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().cancel_streaming(&self.backend.as_ref());
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
        }
    }

    pub fn edit_chat_message(
        &mut self,
        message_id: usize,
        updated_message: String,
        regenerate: bool,
    ) {
        if let Some(chat) = &mut self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            if regenerate {
                if chat.is_streaming {
                    chat.cancel_streaming(&self.backend.as_ref());
                }

                chat.remove_messages_from(message_id);
                chat.send_message_to_model(updated_message, &self.backend.as_ref());
            } else {
                chat.edit_message(message_id, updated_message);
            }
        }
    }

    pub fn eject_model(&mut self) -> Result<()> {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::EjectModel(tx))
            .context("Failed to send eject model command")?;

        let _ = rx
            .recv()
            .context("Failed to receive eject model response")?
            .context("Eject model operation failed");

        self.current_chat_id = None;
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // The following functions are used to process the information after events are  //
    // completed from the backend.                                                   //
    // We only have a single event signal in Makepad, so we update everything (it is //
    // not the most efficient way to do it, it is the simplest for now)              //
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn process_event_signal(&mut self) {
        self.update_downloads();
        self.update_chat_messages();
        self.update_search_results();
    }

    fn update_search_results(&mut self) {
        match self.search.process_results() {
            Ok(Some(models)) => {
                self.search.set_models(models);
            }
            Ok(None) => {
                // No results arrived, do nothing
            }
            Err(_) => {
                self.search.set_models(vec![]);
            }
        }
    }

    fn update_chat_messages(&mut self) {
        let Some(chat) = self.get_current_chat() else {
            return;
        };
        chat.borrow_mut().update_messages();
    }

    fn update_downloads(&mut self) {
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

        // Reload downloaded files and pending downloads from the backend
        if !completed_download_ids.is_empty() {
            self.load_downloaded_files();
            self.load_pending_downloads();
        }

        // For search results let's trust on our local cache, but updating
        // the downloaded state of the files
        for file_id in completed_download_ids {
            self.search
                .update_downloaded_file_in_search_results(&file_id, true);
        }
    }
}
