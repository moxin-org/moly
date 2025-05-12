use std::sync::mpsc::channel;
use std::sync::Arc;

use crate::shared::actions::ChatAction;

use super::capture::register_capture_manager;
use super::chats::chat::ChatID;
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
use moly_protocol::data::{Author, File, FileID, Model, ModelID, PendingDownload};

use makepad_widgets::*;
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

#[derive(Clone, Debug, PartialEq)]
pub enum ProviderSyncingStatus {
    NotSyncing,
    Syncing(ProviderSyncing),
    Synced,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderSyncing {
    pub current: u32,
    pub total: u32,
}

pub struct Store {
    pub search: Search,
    pub downloads: Downloads,
    pub chats: Chats,
    pub preferences: Preferences,
    pub bot_repo: Option<BotRepo>,
    moly_client: Arc<MolyClient>,
    pub provider_syncing_status: ProviderSyncingStatus,
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

        let moly_client = Arc::new(MolyClient::new(format!("http://localhost:{}", server_port)));

        register_capture_manager();

        let mut store = Self {
            search: Search::new(Arc::clone(&moly_client)),
            downloads: Downloads::new(Arc::clone(&moly_client)),
            chats: Chats::new(Arc::clone(&moly_client)),
            moly_client,
            preferences,
            bot_repo: None,
            provider_syncing_status: ProviderSyncingStatus::NotSyncing,
        };

        store.chats.load_chats();
        store.init_current_chat();

        store.sync_with_moly_server();
        store.load_preference_connections();

        store
    }

    /// Check if the main moly server provider is enabled in settings.
    pub fn is_moly_server_enabled(&self) -> bool {
        self.preferences.providers_preferences.iter().any(|p| {
            p.provider_type == ProviderType::MolyServer
                && p.enabled
                && p.url.starts_with(self.moly_client.address())
        })
    }

    /// Check if the connection to moly server was successful.
    pub fn is_moly_server_connected(&self) -> bool {
        self.moly_client.is_connected() && self.is_moly_server_enabled()
    }

    /// Pull the latest data from moly server.
    pub fn sync_with_moly_server(&mut self) {
        if !self.is_moly_server_enabled() {
            return;
        }

        let (tx, rx) = channel();
        self.moly_client.test_connection(tx);
        if let Ok(response) = rx.recv() {
            match response {
                Ok(()) => {
                    self.downloads.load_downloaded_files();
                    self.downloads.load_pending_downloads();
                    self.search.load_featured_models();
                }
                Err(_err) => {}
            }
        };
    }

    pub fn get_chat_associated_bot(&self, chat_id: ChatID) -> Option<BotId> {
        self.chats
            .get_chat_by_id(chat_id)
            .and_then(|chat| chat.borrow().associated_bot.clone())
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
        self.chats.eject_model().expect("Failed to eject model");

        self.downloads.delete_file(file_id.clone())?;
        self.search
            .update_downloaded_file_in_search_results(&file_id, false);

        Ok(())
    }

    pub fn handle_action(&mut self, action: &Action) {
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
            Cx::post_action(ChatAction::ChatSelected(chat_id));
        } else {
            self.chats.create_empty_chat(None);
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
        if let ProviderFetchModelsResult::None = result {
            return;
        }
        let fetched_from_moly_server = self.chats.handle_provider_connection_result(
            result,
            &mut self.preferences,
            &mut self.provider_syncing_status,
        );
        if fetched_from_moly_server && !self.moly_client.is_connected() {
            self.sync_with_moly_server();
        }
    }

    /// Set the provider syncing status to indicate a single provider is being synced
    pub fn set_syncing_single_provider(&mut self) {
        // TODO: this is called in multiple places usually besides test_provider_and_fetch_models
        // we should refactor this to avoid code duplication. Ideally we'd call this function
        // from test_provider_and_fetch_models and have it increase the syncing count instead of resetting to 1
        // (if we have more than one provider to sync).
        self.provider_syncing_status = ProviderSyncingStatus::Syncing(ProviderSyncing {
            current: 0,
            total: 1,
        });
    }

    /// Loads the preference connections from the preferences and registers them in the chats.
    pub fn load_preference_connections(&mut self) {
        let supported = supported_providers::load_supported_providers();
        let mut final_list = Vec::new();

        for s in &supported {
            let maybe_prefs = self
                .preferences
                .providers_preferences
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
        let urls_to_fetch: Vec<String> = self
            .preferences
            .providers_preferences
            .iter()
            // TODO: If the provider requires an API key, we should fetch only if the API key is set
            .filter(|pp| {
                pp.enabled
                    && (pp.api_key.is_some()
                        || pp.provider_type == ProviderType::MoFa
                        || pp.provider_type == ProviderType::DeepInquire
                        || pp.url.starts_with("http://localhost"))
            })
            .map(|pp| pp.url.clone())
            .collect();

        self.provider_syncing_status = ProviderSyncingStatus::Syncing(ProviderSyncing {
            current: 0,
            total: urls_to_fetch.len() as u32,
        });

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
            // Because MolyKit does not currently expose an API to update the clients, we'll remove and recreate the entire bot repo
            // TODO(MolyKit): Find a better way to do this
            self.bot_repo = None;
        }
    }

    pub fn remove_provider(&mut self, url: &str) {
        self.chats.remove_provider(url);
        self.preferences.remove_provider(url);
    }
}
