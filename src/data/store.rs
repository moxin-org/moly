use crate::app::app_runner;
use crate::shared::actions::ChatAction;

use super::chats::chat::ChatID;
use super::downloads::download::DownloadFileAction;
use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::providers::{ProviderFetchModelsResult, ProviderType};
use super::search::SortCriteria;
use super::supported_providers;
use super::{chats::Chats, downloads::Downloads, search::Search};
use chrono::{DateTime, Utc};
use makepad_widgets::{Action, ActionDefaultRef, DefaultNone};
use moly_kit::utils::asynchronous::spawn;

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
    pub bot_context: Option<BotContext>,
    moly_client: MolyClient,
    pub provider_syncing_status: ProviderSyncingStatus,

    pub provider_icons: Vec<LiveDependency>,
}

const MOLY_SERVER_VERSION_EXTENSION: &str = "/api/v1";

impl Store {
    pub fn load_into_app() {
        spawn(async move {
            let preferences = Preferences::load().await;

            let server_port = std::env::var("MOLY_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8765);

            let moly_client = MolyClient::new(format!("http://localhost:{}", server_port));

            let chats = Chats::load(moly_client.clone()).await;

            let mut store = Self {
                search: Search::new(moly_client.clone()),
                downloads: Downloads::new(moly_client.clone()),
                chats,
                moly_client,
                preferences,
                bot_context: None,
                provider_syncing_status: ProviderSyncingStatus::NotSyncing,
                provider_icons: vec![],
            };

            store.init_current_chat();
            store.sync_with_moly_server();
            store.load_preference_connections();

            app_runner().defer(move |app, cx, _| {
                app.store = Some(store);
                app.ui.view(id!(body)).set_visible(cx, true);
                cx.redraw_all(); // app.ui.redraw(cx) doesn't work as expected on web.
            });
        })
    }

    /// Check if the main moly server provider is enabled in settings.
    pub fn is_moly_server_enabled(&self) -> bool {
        self.preferences.providers_preferences.iter().any(|p| {
            p.provider_type == ProviderType::MolyServer
                && p.enabled
                && p.url.starts_with(&self.moly_client.address())
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

        let moly_client = self.moly_client.clone();
        spawn(async move {
            let Ok(()) = moly_client.test_connection().await else {
                return;
            };

            app_runner().defer(|app, _, _| {
                let store = app.store.as_mut().unwrap();
                store.downloads.load_downloaded_files();
                store.downloads.load_pending_downloads();
                store.search.load_featured_models();
            });
        });
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

    pub fn delete_file(&mut self, file_id: FileID) {
        let moly_client = self.moly_client.clone();
        spawn(async move {
            let Ok(()) = moly_client.eject_model().await else {
                eprintln!("Eject model operation failed");
                return;
            };

            let Ok(()) = moly_client.delete_file(file_id.clone()).await else {
                eprintln!("Delete file operation failed");
                return;
            };

            app_runner().defer(move |app, _, _| {
                let store = app.store.as_mut().unwrap();
                store.downloads.load_downloaded_files();
                store.downloads.load_pending_downloads();
                store
                    .search
                    .update_downloaded_file_in_search_results(&file_id, false);
            });
        });
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

        let mut address = self.moly_client.address().clone();
        address.push_str(MOLY_SERVER_VERSION_EXTENSION);

        if !completed_download_ids.is_empty() {
            if let Some(provider) = self.chats.providers.get(&address).cloned() {
                if provider.provider_type == ProviderType::MolyServer && provider.enabled {
                    self.chats.test_provider_and_fetch_models(
                        &provider.url,
                        &mut self.provider_syncing_status,
                    );
                }
            }
        }

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
            if let Some(chat_id) = self.chats.get_last_selected_chat_id() {
                Cx::post_action(ChatAction::ChatSelected(chat_id));
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

        // Collect providers first to avoid borrow issues
        let providers_to_register: Vec<Provider> = urls_to_fetch
            .iter()
            .filter_map(|url| self.chats.providers.get(url).cloned())
            .collect();

        for provider in providers_to_register {
            self.chats
                .register_provider(provider, &mut self.provider_syncing_status);
        }
    }

    pub fn insert_or_update_provider(&mut self, provider: &Provider) {
        // Update in memory
        self.chats
            .insert_or_update_provider(provider, &mut self.provider_syncing_status);
        // Update in preferences (persist in disk)
        self.preferences.insert_or_update_provider(provider);
        // Update in MolyKit (to update the API key used by the client, if needed)
        if let Some(_bot_context) = &self.bot_context {
            // Because MolyKit does not currently expose an API to update the clients, we'll remove and recreate the entire bot context
            // TODO(MolyKit): Find a better way to do this
            self.bot_context = None;
        }
    }

    pub fn remove_provider(&mut self, url: &str) {
        self.chats.remove_provider(url);
        self.preferences.remove_provider(url);
    }

    pub fn get_provider_icon(&self, provider_name: &str) -> Option<LiveDependency> {
        // TODO: a more robust, less horrible way to programatically swap icons that are loaded as live dependencies
        // Find a path that contains the provider name
        self.provider_icons
            .iter()
            .find(|icon| {
                icon.as_str()
                    .to_lowercase()
                    .contains(&provider_name.to_lowercase())
            })
            .cloned()
    }
}
