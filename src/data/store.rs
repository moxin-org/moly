use super::chats::chat::ChatID;
use super::chats::chat_entity::ChatEntityId;
use super::chats::model_loader::ModelLoaderStatusChanged;
use super::downloads::download::DownloadFileAction;
use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::search::SortCriteria;
use super::supported_providers;
use super::{chats::Chats, downloads::Downloads, search::Search};
use anyhow::Result;
use chrono::{DateTime, Utc};
use makepad_widgets::{Action, ActionDefaultRef, DefaultNone};

use makepad_widgets::*;
use serde::{Deserialize, Serialize};
use super::chats::{Provider, ProviderConnectionResult, ProviderTestResultAction, ServerConnectionStatus};
use moly_protocol::data::{Author, DownloadedFile, File, FileID, Model, ModelID, PendingDownload};

#[allow(dead_code)]
const DEFAULT_MOFA_ADDRESS: &str = "http://localhost:8000";

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

pub struct Store {
    /// This is the backend representation, including the sender and receiver ends of the channels to
    /// communicate with the backend thread.
    // pub backend: Rc<Backend>,
    pub moly_client: MolyClient,

    pub search: Search,
    pub downloads: Downloads,
    pub chats: Chats,
    pub preferences: Preferences,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let preferences = Preferences::load();

        let server_port = std::env::var("MOLY_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8765);

        let moly_client = MolyClient::new(format!("http://localhost:{}", server_port));

        let mut store = Self {
            moly_client: moly_client.clone(),
            search: Search::new(moly_client.clone()),
            downloads: Downloads::new(moly_client.clone()),
            chats: Chats::new(moly_client.clone()),
            preferences,
        };

        store.downloads.load_downloaded_files();
        store.downloads.load_pending_downloads();

        store.chats.load_chats();
        store.init_current_chat();

        store.search.load_featured_models();
        // store
        //     .chats
        //     .register_mofa_server(DEFAULT_MOFA_ADDRESS.to_string());

        store.load_preference_connections();

        store
    }

    pub fn load_model(&mut self, file: &File) {
        self.chats.load_model(file, None);
    }

    pub fn update_server_port(&mut self, server_port: u16) {
        if let Some(file) = &self.chats.loaded_model {
            if !self.chats.model_loader.is_loading() {
                self.chats.load_model(&file.clone(), Some(server_port));
            }
        }
    }

    fn update_load_model(&mut self) {
        if self.chats.model_loader.is_loaded() {
            self.chats.loaded_model = self
                .chats
                .model_loader
                .file_id()
                .map(|id| self.downloads.get_file(&id))
                .flatten()
                .cloned();
        }

        if let Some(file) = &self.chats.loaded_model {
            self.preferences.set_current_chat_model(file.id.clone());

            // If there is no chat, create an empty one
            if self.chats.get_current_chat().is_none() {
                self.chats.create_empty_chat();
            }
        }
    }

    pub fn send_message_to_current_entity(
        &mut self,
        prompt: String,
        regenerate_from: Option<usize>,
    ) {
        let entity_id = self
            .chats
            .get_current_chat()
            .and_then(|c| c.borrow().associated_entity.clone());

        if let Some(entity_id) = entity_id {
            self.send_entity_message(&entity_id, prompt, regenerate_from);
        }
    }

    pub fn send_entity_message(
        &mut self,
        entity_id: &ChatEntityId,
        prompt: String,
        regenerate_from: Option<usize>,
    ) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            if let Some(message_id) = regenerate_from {
                chat.remove_messages_from(message_id);
            }

            match entity_id {
                ChatEntityId::Agent(model_id) => {
                    let model = self.chats.remote_models.get(model_id);
                    if let Some(model) = model {
                        let client = self.chats.get_client_for_provider(&model.provider_url);
                        if let Some(client) = client {
                            chat.send_message_to_agent(model, prompt, client.as_ref());
                        } else {
                            eprintln!("client not found for provider: {:?}", model.provider_url);
                        }
                    } else {
                        eprintln!("model not found: {:?}", model_id);
                    }
                }
                ChatEntityId::ModelFile(file_id) => {
                    if let Some(file) = self.downloads.get_file(&file_id) {
                        chat.send_message_to_model(
                            prompt,
                            file,
                            self.chats.model_loader.clone(),
                            &self.moly_client,
                        );
                    }
                }
                ChatEntityId::RemoteModel(model_id) => {
                    let model = self.chats.remote_models.get(model_id);
                    if let Some(model) = model {
                        let client = self.chats.get_client_for_provider(&model.provider_url);
                        if let Some(client) = client {
                            chat.send_message_to_remote_model(prompt, model, client.as_ref());
                        } else {
                            eprintln!("client not found for provider: {:?}", model.provider_url);
                        }
                    } else {
                        eprintln!("model not found: {:?}", model_id);
                    }
                }
            }
        }
    }

    pub fn edit_chat_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            chat.edit_message(message_id, updated_message);
        }
    }

    pub fn _get_loading_file(&self) -> Option<&File> {
        self.chats
            .model_loader
            .get_loading_file_id()
            .map(|file_id| self.downloads.get_file(&file_id))
            .flatten()
    }

    pub fn _get_loaded_downloaded_file(&self) -> Option<DownloadedFile> {
        if let Some(file) = &self.chats.loaded_model {
            self.downloads
                .downloaded_files
                .iter()
                .find(|d| d.file.id == file.id)
                .cloned()
        } else {
            None
        }
    }

    pub fn get_chat_entity_name(&self, chat_id: ChatID) -> Option<String> {
        let Some(chat) = self.chats.get_chat_by_id(chat_id) else {
            return None;
        };

        match &chat.borrow().associated_entity {
            Some(ChatEntityId::ModelFile(ref file_id)) => self
                .downloads
                .downloaded_files
                .iter()
                .find(|df| df.file.id == *file_id)
                .map(|df| Some(df.file.name.clone()))?,
            Some(ChatEntityId::Agent(model_id)) => self
                .chats
                .remote_models
                .get(model_id)
                .map(|m| m.name.clone()),
            Some(ChatEntityId::RemoteModel(model_id)) => self
                .chats
                .remote_models
                .get(&model_id)
                .map(|m| m.name.clone()),
            None => {
                // Fallback to loaded model if exists
                self.chats.loaded_model.as_ref().map(|m| m.name.clone())
            }
        }
    }

    /// This function combines the search results information for a given model
    /// with the download information for the files of that model.
    pub fn add_download_info_to_model(&self, model: &Model) -> ModelWithDownloadInfo {
        let files = model
            .files
            .iter()
            .map(|file| {
                let download = self
                    .downloads
                    .pending_downloads
                    .iter()
                    .find(|d| d.file.id == file.id)
                    .cloned();

                FileWithDownloadInfo {
                    file: file.clone(),
                    download,
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
            files,
        }
    }

    pub fn get_model_and_file_download(&self, file_id: &str) -> (Model, File) {
        if let Some(result) = self
            .downloads
            .get_model_and_file_for_pending_download(file_id)
        {
            result
        } else {
            self.search
                .get_model_and_file_from_search_results(file_id)
                .unwrap()
        }
    }

    pub fn delete_file(&mut self, file_id: FileID) -> Result<()> {
        if self
            .chats
            .loaded_model
            .as_ref()
            .map_or(false, |file| file.id == file_id)
        {
            self.chats.eject_model().expect("Failed to eject model");
        }

        self.chats.remove_file_from_associated_entity(&file_id);
        self.downloads.delete_file(file_id.clone())?;
        self.search
            .update_downloaded_file_in_search_results(&file_id, false);

        Ok(())
    }

    pub fn handle_action(&mut self, action: &Action) {
        self.chats.handle_action(action);
        self.search.handle_action(action);
        self.downloads.handle_action(action);

        if let Some(_) = action.downcast_ref::<ModelLoaderStatusChanged>() {
            self.update_load_model();
        }

        if let Some(_) = action.downcast_ref::<DownloadFileAction>() {
            self.update_downloads();
        }
    }

    fn update_downloads(&mut self) {
        let completed_download_ids = self.downloads.refresh_downloads_data();

        // For search results let's trust on our local cache, but updating
        // the downloaded state of the files
        for file_id in completed_download_ids {
            self.search
                .update_downloaded_file_in_search_results(&file_id, true);
        }
    }

    fn init_current_chat(&mut self) {
        if let Some(chat_id) = self.chats.get_last_selected_chat_id() {
            self.chats.set_current_chat(Some(chat_id));
        } else {
            self.chats.create_empty_chat();
        }

        // If there is no load model, let's try to load the one from preferences
        if self.chats.loaded_model.is_none() {
            if let Some(ref file_id) = self.preferences.current_chat_model {
                if let Some(file) = self
                    .downloads
                    .downloaded_files
                    .iter()
                    .find(|d| d.file.id == *file_id)
                    .map(|d| d.file.clone())
                {
                    self.load_model(&file);
                }
            }
        }
    }

    pub fn delete_chat(&mut self, chat_id: ChatID) {
        self.chats.remove_chat(chat_id);

        // TODO Decide proper behavior when deleting the current chat
        // For now, we just create a new empty chat because we don't fully
        // support having no chat selected
        self.init_current_chat();
    }
    
    // TODO Rework this mess
    pub fn handle_provider_connection_action(&mut self, result: ProviderTestResultAction) {
        match result {
            ProviderTestResultAction::Success(address, models) => {
                self.chats.handle_provider_connection_result(ProviderConnectionResult::Connected(address, models), &mut self.preferences);
            }
            ProviderTestResultAction::Failure(address_opt) => {
                if let Some(addr) = address_opt {
                    eprintln!("Failed to connect to provider at {}", addr);
                    self.chats.handle_provider_connection_result(ProviderConnectionResult::Unavailable(addr), &mut self.preferences);
                }
            },
            _ => {},
        }
    }

    /// Loads the preference connections from the preferences and registers them in the chats.
    pub fn load_preference_connections(&mut self) {
        let supported = supported_providers::load_supported_providers();
        let mut final_list = Vec::new();

        for s in &supported {
            let maybe_prefs = self.preferences.providers_preferences
                .iter()
                .find(|pp| pp.url == s.url);

            if let Some(prefs) = maybe_prefs {
                final_list.push(Provider {
                    name: s.name.clone(),
                    url: prefs.url.clone(),
                    api_key: prefs.api_key.clone(),
                    provider_type: s.provider_type.clone(),
                    connection_status: if prefs.enabled {
                        ServerConnectionStatus::Connected
                    } else {
                        ServerConnectionStatus::Disconnected
                    },
                    enabled: prefs.enabled,
                    models: vec![],
                });
            } else {
                // Known from JSON but user has no preferences
                final_list.push(Provider {
                    name: s.name.clone(),
                    url: s.url.clone(),
                    api_key: None,
                    provider_type: s.provider_type.clone(),
                    connection_status: ServerConnectionStatus::Disconnected,
                    enabled: true,
                    models: vec![],
                });
            }
        }

        // Custom providers from preferences (not in the JSON)
        for pp in &self.preferences.providers_preferences {
            let is_custom = !supported.iter().any(|sp| sp.url == pp.url);
            if is_custom {
                final_list.push(Provider {
                    name: pp.url.clone(),
                    url: pp.url.clone(),
                    api_key: pp.api_key.clone(),
                    provider_type: pp.provider_type.clone(),
                    connection_status: ServerConnectionStatus::Disconnected,
                    enabled: pp.enabled,
                    models: vec![],
                });
            }
        }

        for provider in final_list {
            self.chats.providers.insert(provider.url.clone(), provider);
        }

        self.auto_fetch_for_enabled_providers();
    }

    fn auto_fetch_for_enabled_providers(&mut self) {
        // Automatically fetch providers that are enabled and have an API key or are MoFa servers
        let urls_to_fetch: Vec<String> = self.preferences.providers_preferences
            .iter()
            .filter(|pp| pp.enabled && (pp.api_key.is_some() || pp.provider_type == ProviderType::MoFa || pp.url.starts_with("http://localhost")))
            .map(|pp| pp.url.clone())
            .collect();

        for url in urls_to_fetch {
            if let Some(provider) = self.chats.providers.get(&url) {
                // TODO(Julian): split register and test
                // Register the provider client, it triggers test_provider_and_fetch_models internally
                self.chats.register_provider(provider.clone());
            }
        }
    }

    pub fn insert_or_update_provider(&mut self, provider: &Provider) {
        self.chats.insert_or_update_provider(provider);
        self.preferences.insert_or_update_provider(provider);
    }
}

#[derive(Live, LiveHook, PartialEq, Debug, LiveRead, Serialize, Deserialize, Clone)]
pub enum ProviderType {
    #[pick]
    OpenAI,
    MoFa,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::OpenAI
    }
}

impl ProviderType {
    pub fn from_usize(value: usize) -> Self {
        match value {
            0 => ProviderType::OpenAI,
            1 => ProviderType::MoFa,
            _ => panic!("Invalid provider type"),
        }
    }
}
