use std::sync::{Arc, Mutex};

use crate::app::app_runner;
use crate::data::providers::ProviderID;
use crate::shared::actions::ChatAction;

use super::chats::chat::ChatID;
use super::downloads::download::DownloadFileAction;
use super::mcp_servers::McpServersConfig;
use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::providers::{ProviderFetchModelsResult, ProviderType};
use super::search::SortCriteria;
use super::supported_providers;
use super::{chats::Chats, downloads::Downloads, search::Search};
use chrono::{DateTime, Utc};
use makepad_widgets::{Action, ActionDefaultRef, DefaultNone};
use moly_kit::controllers::chat::ChatController;
use moly_kit::utils::asynchronous::spawn;

use super::providers::{Provider, ProviderConnectionStatus};
use moly_kit::mcp::mcp_manager::McpManagerClient;
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
    pub chat_controller: Arc<Mutex<ChatController>>,
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
                chat_controller: ChatController::new_arc(),
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
            // Find MolyServer provider
            let provider = self
                .chats
                .providers
                .values()
                .find(|p| {
                    p.url == address && p.provider_type == ProviderType::MolyServer && p.enabled
                })
                .cloned();

            if let Some(provider) = provider {
                self.chats.test_provider_and_fetch_models(
                    &provider.id,
                    &mut self.provider_syncing_status,
                );
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
                .find(|pp| pp.id == s.id || (pp.id.is_empty() && pp.url == s.url));

            if let Some(prefs) = maybe_prefs {
                final_list.push(Provider {
                    id: if !prefs.id.is_empty() {
                        prefs.id.clone()
                    } else {
                        s.id.clone()
                    },
                    name: s.name.clone(),
                    url: prefs.url.clone(),
                    api_key: prefs.api_key.clone(),
                    provider_type: s.provider_type.clone(),
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: prefs.enabled,
                    models: vec![],
                    was_customly_added: prefs.was_customly_added,
                    system_prompt: prefs.system_prompt.clone(),
                    tools_enabled: prefs.tools_enabled,
                });
            } else {
                // Known from supported_providers.json but user has no preferences
                final_list.push(Provider {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    url: s.url.clone(),
                    api_key: None,
                    provider_type: s.provider_type.clone(),
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: false,
                    models: vec![],
                    was_customly_added: false,
                    system_prompt: None,
                    tools_enabled: true,
                });
            }
        }

        // Custom providers from preferences (not in the supported_providers.json)
        for pp in &self.preferences.providers_preferences {
            let is_custom = !supported
                .iter()
                .any(|sp| sp.id == pp.id || (pp.id.is_empty() && sp.url == pp.url));
            if is_custom {
                // Ensure provider has an ID
                let mut pp_clone = pp.clone();
                pp_clone.ensure_id();

                final_list.push(Provider {
                    id: pp_clone.id.clone(),
                    name: pp_clone.name.clone(),
                    url: pp_clone.url.clone(),
                    api_key: pp_clone.api_key.clone(),
                    provider_type: pp_clone.provider_type.clone(),
                    connection_status: ProviderConnectionStatus::Disconnected,
                    enabled: pp_clone.enabled,
                    models: vec![],
                    was_customly_added: pp_clone.was_customly_added,
                    system_prompt: pp_clone.system_prompt.clone(),
                    tools_enabled: pp_clone.tools_enabled,
                });
            }
        }

        for provider in final_list {
            self.chats.providers.insert(provider.id.clone(), provider);
        }

        self.auto_fetch_for_enabled_providers();
    }

    fn auto_fetch_for_enabled_providers(&mut self) {
        // Automatically fetch providers that are enabled and have an API key or are MoFa servers
        let ids_to_fetch: Vec<String> = self
            .preferences
            .providers_preferences
            .iter()
            // TODO: If the provider requires an API key, we should fetch only if the API key is set
            .filter(|pp| {
                pp.enabled
                    && (pp.api_key.is_some()
                        || pp.provider_type == ProviderType::MoFa
                        || pp.provider_type == ProviderType::DeepInquire
                        || pp.provider_type == ProviderType::OpenAIRealtime
                        || pp.url.starts_with("http://localhost"))
            })
            .map(|pp| {
                // Ensure we have an ID to use
                if !pp.id.is_empty() {
                    pp.id.clone()
                } else {
                    // Generate ID for backward compatibility
                    super::preferences::ProviderPreferences::generate_id_from_url_and_name(
                        &pp.url,
                        &pp.name,
                        &pp.provider_type,
                    )
                }
            })
            .collect();

        // Collect providers first to avoid borrow issues
        let providers_to_register: Vec<Provider> = ids_to_fetch
            .iter()
            .filter_map(|id| self.chats.providers.get(id).cloned())
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
        // Because MolyKit does not currently expose an API to update the clients, we'll reset the controller
        // TODO(MolyKit): Find a better way to do this
        self.chat_controller.lock().unwrap().set_client(None);
        self.chat_controller.lock().unwrap().set_tool_manager(None);
    }

    pub fn remove_provider(&mut self, provider_id: &ProviderID) {
        self.chats.remove_provider(provider_id);
        self.preferences.remove_provider(provider_id);
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

    pub fn get_mcp_servers_config(&self) -> &McpServersConfig {
        &self.preferences.mcp_servers_config
    }

    pub fn get_mcp_servers_config_json(&self) -> String {
        self.preferences.get_mcp_servers_config_json()
    }

    /// Creates a new MCP tool manager and loads servers asynchronously
    /// Returns the manager immediately, loading happens in the background
    pub fn create_and_load_mcp_tool_manager(&self) -> McpManagerClient {
        let tool_manager = McpManagerClient::new();

        // Check if MCP servers are globally enabled
        if !self.preferences.get_mcp_servers_enabled() {
            // Return empty tool manager if globally disabled
            return tool_manager;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mcp_config = self.get_mcp_servers_config().clone();
            tool_manager.set_dangerous_mode_enabled(mcp_config.dangerous_mode_enabled);
            let tool_manager_clone = tool_manager.clone();

            spawn(async move {
                // Load MCP servers from configuration
                for (server_id, server_config) in mcp_config.list_enabled_servers() {
                    if let Some(transport) = server_config.to_transport() {
                        match tool_manager_clone.add_server(server_id, transport).await {
                            Ok(()) => {
                                ::log::debug!("Successfully added MCP server: {}", server_id);
                            }
                            Err(e) => {
                                ::log::error!("Failed to add MCP server '{}': {}", server_id, e);
                            }
                        }
                    }
                }
            });
        }

        tool_manager
    }

    pub fn update_mcp_servers_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        self.preferences.update_mcp_servers_from_json(json)?;
        self.update_mcp_tool_manager();

        Ok(())
    }

    pub fn update_mcp_tool_manager(&mut self) {
        let new_tool_manager = self.create_and_load_mcp_tool_manager();
        self.chat_controller
            .lock()
            .unwrap()
            .set_tool_manager(Some(new_tool_manager));
    }

    pub fn set_mcp_servers_enabled(&mut self, enabled: bool) {
        self.preferences.set_mcp_servers_enabled(enabled);
        // Reset controller to apply the new MCP setting
        self.chat_controller.lock().unwrap().reset_connections();
    }

    pub fn set_mcp_servers_dangerous_mode_enabled(&mut self, enabled: bool) {
        self.preferences
            .set_mcp_servers_dangerous_mode_enabled(enabled);
        self.update_mcp_tool_manager();
    }
}
