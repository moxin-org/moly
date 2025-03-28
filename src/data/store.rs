use super::capture::register_capture_manager;
use super::chats::chat::ChatID;
use super::chats::chat_entity::ChatEntityId;
use super::downloads::download::DownloadFileAction;
use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::providers::{ProviderFetchModelsResult, ProviderType};
use super::search::SortCriteria;
use super::supported_providers;
use super::{chats::Chats, downloads::Downloads, search::Search};
use anyhow::Result;
use chrono::{DateTime, Utc};
use makepad_widgets::{Action, ActionDefaultRef, DefaultNone};

use super::providers::{Provider, ProviderConnectionStatus};
use moly_protocol::data::{Author, DownloadedFile, File, FileID, Model, ModelID, PendingDownload};

use moly_kit::*;

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
    pub search: Search,
    pub downloads: Downloads,
    pub chats: Chats,
    pub preferences: Preferences,
    pub bot_repo: Option<BotRepo>,
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

        register_capture_manager();

        let mut store = Self {
            search: Search::new(moly_client.clone()),
            downloads: Downloads::new(moly_client.clone()),
            chats: Chats::new(moly_client),
            preferences,
            bot_repo: None,
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

    pub fn edit_chat_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            chat.edit_message(message_id, updated_message);
        }
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

    }

    pub fn delete_chat(&mut self, chat_id: ChatID) {
        self.chats.remove_chat(chat_id);

        // TODO Decide proper behavior when deleting the current chat
        // For now, we just create a new empty chat because we don't fully
        // support having no chat selected
        self.init_current_chat();
    }

    pub fn handle_provider_connection_action(&mut self, result: ProviderFetchModelsResult) {
        self.chats.handle_provider_connection_result(result, &mut self.preferences);
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
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: prefs.enabled,
                    models: vec![],
                    was_customly_added: prefs.was_customly_added,
                });
            } else {
                // Known from supported_providers.json but user has no preferences
                final_list.push(Provider {
                    name: s.name.clone(),
                    url: s.url.clone(),
                    api_key: None,
                    provider_type: s.provider_type.clone(),
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: false,
                    models: vec![],
                    was_customly_added: false,
                });
            }
        }

        // Custom providers from preferences (not in the supported_providers.json)
        for pp in &self.preferences.providers_preferences {
            let is_custom = !supported.iter().any(|sp| sp.url == pp.url);
            if is_custom {
                final_list.push(Provider {
                    name: pp.name.clone(),
                    url: pp.url.clone(),
                    api_key: pp.api_key.clone(),
                    provider_type: pp.provider_type.clone(),
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: pp.enabled,
                    models: vec![],
                    was_customly_added: pp.was_customly_added,
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
            // TODO: If the provider requires an API key, we should fetch only if the API key is set
            .filter(|pp| pp.enabled && (pp.api_key.is_some() || pp.provider_type == ProviderType::MoFa || pp.provider_type == ProviderType::DeepInquire || pp.url.starts_with("http://localhost")))
            .map(|pp| pp.url.clone())
            .collect();

        for url in urls_to_fetch {
            if let Some(provider) = self.chats.providers.get(&url) {
                // Register the provider client, it triggers test_provider_and_fetch_models internally
                self.chats.register_provider(provider.clone());
            }
        }
    }

    pub fn insert_or_update_provider(&mut self, provider: &Provider) {
        // Update in memory
        self.chats.insert_or_update_provider(provider);
        // Update in preferences (persist in disk)
        self.preferences.insert_or_update_provider(provider);
        // Update in MolyKit (to update the API key used by the client, if needed)
        if let Some(_bot_repo) = &self.bot_repo {
            // Because MolyKit does not currently expose an API to update the clients, 
            // we'll remove and recreate the entire bot repo
            // TODO(MolyKit): I think BotRepo should be an actual repository-like interface and not a client interface, it might still hold a main
            // client/multi_client, but the crate user should be able to update the clients (add new ones, update existing ones, remove, etc.)
            // it would also be helpful if BotRepo can expose the bots without re-fetching every time. (or at least rename BotRepo, it does not follow
            // a repository pattern)
            self.bot_repo = None;
        }
    }

    pub fn remove_provider(&mut self, url: &str) {
        self.chats.remove_provider(url);
        self.preferences.remove_provider(url);
    }
}
