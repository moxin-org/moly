use std::{
    collections::HashMap, path::{Path, PathBuf}, sync::{
        Arc, Mutex,
    }
};

use chrono::Utc;
use moly_protocol::{
    data::{DownloadedFile, FileID, PendingDownload},
    open_ai::{ChatRequestData, ChatResponse},
    protocol::{
        FileDownloadResponse, LoadModelOptions, LoadModelResponse,
    },
};
use tokio::sync::{mpsc::{UnboundedSender, Sender, Receiver}, RwLock};

use crate::store::{
    self,
    model_cards::{self, ModelCard, ModelCardManager},
    ModelFileDownloader,
};

mod api_server;
// Commenting out this implementation as it is not being used anywhere and requires 
// a lot of reworking to be used with the http server.
// mod chat_ui;

#[derive(Debug, Clone)]
pub enum DownloadControlCommand {
    Stop(FileID),
}

// TODO(Julian): Should we just remove ChatBotModel? is not being used anywhere
// pub type ChatModelBackend = BackendImpl<chat_ui::ChatBotModel>;
pub type LlamaEdgeApiServerBackend = BackendImpl<api_server::LLamaEdgeApiServer>;

pub trait BackendModel: Sized {
    /// Creates a new model or reloads an existing one.
    async fn new_or_reload(
        old_model: Option<Self>,
        file: store::download_files::DownloadedFile,
        options: LoadModelOptions,
        embedding: Option<(PathBuf, u64)>,
    ) -> Result<(Self, LoadModelResponse), anyhow::Error>;
    /// Starts a chat with the model.
    fn chat(
        &self,
        data: ChatRequestData,
        tx: tokio::sync::mpsc::Sender<anyhow::Result<ChatResponse>>,
    ) -> bool;
    /// Stops the model, freeing its resources.
    async fn stop(self);
}

/// The main backend implementation.
pub struct BackendImpl<Model: BackendModel> {
    /// The manager for the model cards.
    model_indexs: ModelCardManager,
    /// The directory where the app data is stored.
    app_data_dir: PathBuf,
    /// The directory where the models are stored.
    models_dir: PathBuf,
    /// The currently loaded model.
    model: Option<Model>,
    /// A channel for sending model download requests to the downloader thread.
    download_tx: UnboundedSender<(
        store::models::Model,
        store::download_files::DownloadedFile,
        model_cards::RemoteFile,
        Sender<anyhow::Result<FileDownloadResponse>>,
    )>,
    /// A channel for sending control commands to the downloader thread.
    control_tx: tokio::sync::broadcast::Sender<DownloadControlCommand>,
    /// A map of file IDs to their progress channels.
    download_progress: Arc<RwLock<HashMap<String, Receiver<anyhow::Result<FileDownloadResponse>>>>>,
}

impl<Model: BackendModel + Send + 'static> BackendImpl<Model> {
    /// Builds a backend instance by initializing the model indexes and the file downloader.
    pub async fn build<A: AsRef<Path>, M: AsRef<Path>>(app_data_dir: A, models_dir: M, max_download_threads: usize) -> Self {
        let app_data_dir = app_data_dir.as_ref().to_path_buf();

        log::info!("build by app_data_dir: {:?}", app_data_dir);

        wasmedge_sdk::plugin::PluginManager::load(None).unwrap();
        std::fs::create_dir_all(&app_data_dir).unwrap_or_else(|_| {
            panic!(
                "Failed to create the Moly app data directory at {:?}",
                app_data_dir
            )
        });

        let model_indexs = store::model_cards::sync_model_cards_repo(&app_data_dir).await;
        let model_indexs = match model_indexs {
            Ok(model_indexs) => {
                log::info!("sync model cards repo success");
                model_indexs
            }
            Err(e) => {
                log::error!("sync model cards repo error: {e}");
                ModelCardManager::empty(app_data_dir.clone())
            }
        };

        let sql_conn = open_sqlite_conn(&app_data_dir);

        // TODO Reorganize these bunch of functions, needs a little more of thought
        let _ = store::models::create_table_models(&sql_conn).unwrap();
        let _ = store::download_files::create_table_download_files(&sql_conn).unwrap();

        let sql_conn = Arc::new(Mutex::new(sql_conn));

        let (control_tx, _control_rx) = tokio::sync::broadcast::channel(100);
        let (download_tx, download_rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let client = reqwest::Client::new();
            let downloader =
                ModelFileDownloader::new(client, sql_conn.clone(), control_tx.clone(),model_indexs.country_code.clone(), 0.1);

            tokio::spawn(ModelFileDownloader::run_loop(
                downloader,
                max_download_threads.max(3),
                download_rx,
            ));
        }

        let backend = Self {
            model_indexs: model_indexs,
            app_data_dir,
            models_dir: models_dir.as_ref().into(),
            download_tx,
            model: None,
            control_tx,
            download_progress: Arc::new(RwLock::new(HashMap::new())),
        };

        backend
    }

    /// Starts downloading a model file and creates a progress channel for it.
    pub async fn start_download(&mut self, file_id: String) -> Result<(), anyhow::Error> {
        let (model, file, remote_file) = {
            let (model_id, file_name) = file_id
                .split_once("#")
                .ok_or_else(|| anyhow::anyhow!("Illegal file_id"))?;

            let index = self.model_indexs
                .get_index_by_id(model_id)
                .ok_or(anyhow::anyhow!("No model found"))?
                .clone();
            
            let remote_model = self.model_indexs.load_model_card(&index)?;
            
            let remote_file = remote_model
                .files
                .into_iter()
                .find(|f| f.name == file_name)
                .ok_or_else(|| anyhow::anyhow!("file not found"))?;

            let download_model = store::models::Model {
                id: Arc::new(remote_model.id),
                name: remote_model.name,
                summary: remote_model.summary,
                size: remote_model.size,
                requires: remote_model.requires,
                architecture: remote_model.architecture,
                released_at: remote_model.released_at,
                prompt_template: remote_model.prompt_template.clone(),
                reverse_prompt: remote_model.reverse_prompt.clone(),
                author: Arc::new(store::model_cards::Author {
                    name: remote_model.author.name,
                    url: remote_model.author.url,
                    description: remote_model.author.description,
                }),
                like_count: remote_model.like_count,
                download_count: remote_model.download_count,
            };

            let remote_file_clone = remote_file.clone();
            let download_file = store::download_files::DownloadedFile {
                id: Arc::new(file_id.clone()),
                model_id: model_id.to_string(),
                name: file_name.to_string(),
                size: remote_file_clone.size,
                quantization: remote_file_clone.quantization,
                prompt_template: remote_model.prompt_template,
                reverse_prompt: remote_model.reverse_prompt,
                context_size: remote_model.context_size,
                downloaded: false,
                file_size: 0,
                download_dir: self.models_dir.to_string_lossy().to_string(),
                downloaded_at: Utc::now(),
                tags: remote_file_clone.tags,
                featured: false,
                sha256: remote_file_clone.sha256.unwrap_or_default(),
            };

            (download_model, download_file, remote_file)
        };

        // Create a channel for progress updates that we'll handle with SSE later
        let (progress_tx, progress_rx) = tokio::sync::mpsc::channel(100);
        self.download_progress.write().await.insert(file_id.clone(), progress_rx);
        
        // Start the download
        self.download_tx.send((
            model,
            file.clone(),
            remote_file,
            progress_tx
        ))?;

        Ok(())
    }

    /// Returns the progress channel for a given file ID.
    pub async fn get_download_progress_channel(&mut self, file_id: String) 
    -> anyhow::Result<Receiver<anyhow::Result<FileDownloadResponse>>> {
        let rx = self.download_progress
            .write()
            .await
            .remove(&file_id) // TODO:(Julian) we should only remove the channel once the download is complete
            .ok_or_else(|| anyhow::anyhow!("No download in progress for this ID"))?;
        Ok(rx)
    }

    pub fn get_downloaded_files(&self) -> Result<Vec<DownloadedFile>, anyhow::Error> {
        store::get_all_download_file(&self.open_db_conn())
            .map_err(|e| anyhow::anyhow!("get download file error: {e}"))
    }

    pub fn delete_file(&self, file_id: String) -> Result<(), anyhow::Error> {
        store::download_files::DownloadedFile::remove(&file_id, &self.open_db_conn())?;
        store::remove_downloaded_file(
            self.models_dir.to_string_lossy().to_string(),
            file_id,
        )
    }

    pub fn get_current_downloads(&self) -> Result<Vec<PendingDownload>, anyhow::Error> {
        store::get_all_pending_downloads(&self.open_db_conn())
            .map_err(|e| anyhow::anyhow!("get pending download file error: {e}"))
    }

    pub fn pause_download(&self, file_id: String) -> Result<(), anyhow::Error> {
        let _ = self.control_tx.send(DownloadControlCommand::Stop(file_id));
        Ok(())
    }

    pub fn cancel_download(&self, file_id: String) -> Result<(), anyhow::Error> {
        self.control_tx.send(DownloadControlCommand::Stop(file_id.clone()))?;
        store::download_files::DownloadedFile::remove(&file_id, &self.open_db_conn())?;
        store::remove_downloaded_file(
            self.models_dir.to_string_lossy().to_string(),
            file_id,
        )
    }

    pub async fn load_model(&mut self, file_id: String, options: LoadModelOptions) -> Result<LoadModelResponse, anyhow::Error> {
        let download_file =
            store::download_files::DownloadedFile::get_by_id(&self.open_db_conn(), &file_id);

        match download_file {
            Ok(file) => {
                nn_preload_file(&file, self.model_indexs.embedding_model());
                let old_model = self.model.take();

                let result = Model::new_or_reload(
                    old_model,
                    file,
                    options,
                    self.model_indexs.embedding_model(),
                ).await;

                match result {
                    Ok((model, response)) => {
                        self.model = Some(model);
                        return Ok(response);
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Load model error: {e}"));
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Load model error: {e}"));
            }
        }
    }

    pub async fn eject_model(&mut self) {
        if let Some(model) = self.model.take() {
            model.stop().await;
        }
    }

    pub fn get_featured_models(&mut self) -> Result<Vec<moly_protocol::data::Model>, anyhow::Error> {
        let res = self.model_indexs.get_featured_model(100, 0);
        match res {
            Ok(indexs) => {
                let mut models = Vec::new();
                for index in indexs {
                    if let Ok(card) = self.model_indexs.load_model_card(&index) {
                        models.push(card);
                    }
                }

                let models = ModelCard::to_model(&models, &self.open_db_conn())
                    .map_err(|e| anyhow::anyhow!("get featured error: {e}"))?;

                Ok(models)
            }
            Err(e) => {
                return Err(anyhow::anyhow!("get featured models error: {e}"));
            }
        }
    }

    pub fn search_models(&mut self, search_text: String) -> Result<Vec<moly_protocol::data::Model>, anyhow::Error> {
        let res = self.model_indexs.search(&search_text, 100, 0);
        match res {
            Ok(indexs) => {
                log::debug!("search models: {}", indexs.len());

                let mut models = Vec::new();
                for index in indexs {
                    match self.model_indexs.load_model_card(&index) {
                        Ok(card) => {
                            models.push(card);
                        }
                        Err(e) => {
                            log::error!("load model card {} error: {e}", index.id);
                        }
                    }
                }

                let models = ModelCard::to_model(&models, &self.open_db_conn())
                    .map_err(|e| anyhow::anyhow!("search models error: {e}"))?;

                Ok(models)
            }
            Err(e) => {
                return Err(anyhow::anyhow!("search models error: {e}"));
            }
        }
    }

    // WIP. Keeping this for reference.
    // pub fn update_models_dir<M: AsRef<Path>>(&mut self, models_dir: M) {
    //     self.models_dir = models_dir.as_ref().to_path_buf();
    // }

    pub fn chat(&self, data: ChatRequestData, tx: tokio::sync::mpsc::Sender<anyhow::Result<ChatResponse>>) -> Result<(), anyhow::Error> {
        if let Some(model) = &self.model {
            model.chat(data, tx);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model not loaded"))
        }
    }

    fn open_db_conn(&self) -> rusqlite::Connection {
        open_sqlite_conn(&self.app_data_dir)
    }
}

pub fn nn_preload_file(
    file: &store::download_files::DownloadedFile,
    embedding: Option<(PathBuf, u64)>,
) {
    let file_path = Path::new(&file.download_dir)
        .join(&file.model_id)
        .join(&file.name);

    let preloads = wasmedge_sdk::plugin::NNPreload::new(
        "moly-chat",
        wasmedge_sdk::plugin::GraphEncoding::GGML,
        wasmedge_sdk::plugin::ExecutionTarget::AUTO,
        &file_path,
    );

    let mut preload_vec = vec![preloads];
    if let Some((embedding_path, _)) = embedding {
        let preloads = wasmedge_sdk::plugin::NNPreload::new(
            "moly-embedding",
            wasmedge_sdk::plugin::GraphEncoding::GGML,
            wasmedge_sdk::plugin::ExecutionTarget::AUTO,
            &embedding_path,
        );
        preload_vec.push(preloads);
    }

    wasmedge_sdk::plugin::PluginManager::nn_preload(preload_vec);
}

fn open_sqlite_conn(app_data_dir: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(Path::new(app_data_dir).join("data.sqlite")).unwrap()
}
