pub mod chat;

use chat::{Chat, ChatID};
use futures::StreamExt;
use moly_kit::BotId;
use moly_protocol::data::*;
use std::collections::HashMap;
use std::{cell::RefCell, path::PathBuf};

use crate::shared::utils::filesystem;

use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::providers::{
    Provider, ProviderBot, ProviderConnectionStatus, ProviderFetchModelsResult, ProviderID,
    ProviderType, fetch_models_for_provider,
};
use super::store::{ProviderSyncing, ProviderSyncingStatus};

pub struct Chats {
    pub moly_client: MolyClient,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub available_bots: HashMap<BotId, ProviderBot>,

    // Legacy field - no longer used but kept for backward compatibility
    // pub provider_clients: HashMap<String, Box<dyn ProviderClient>>,
    /// Map from provider ID to provider
    pub providers: HashMap<ProviderID, Provider>,

    /// Set it thru `set_current_chat` method to trigger side effects.
    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,

    /// Placeholder remote model used when a remote model is not available
    /// This is used to avoid recreating it on each call and make borrowing simpler.
    unknown_bot: ProviderBot,
}

impl Chats {
    fn new(moly_client: MolyClient) -> Self {
        Self {
            moly_client,
            saved_chats: Vec::new(),
            current_chat_id: None,
            chats_dir: PathBuf::from("chats"),
            available_bots: HashMap::new(),
            providers: HashMap::new(),
            unknown_bot: ProviderBot::unknown(),
        }
    }

    pub async fn load(moly_client: MolyClient) -> Self {
        let mut chats = Chats::new(moly_client);

        let fs = filesystem::global();
        let paths = fs
            .list(&chats.chats_dir)
            .await
            .unwrap_or_else(|_| {
                log::info!(
                    "No chats directory found {:?}, it will be created upon first chat creation.",
                    chats.chats_dir
                );
                vec![]
            })
            .into_iter()
            .filter_map(|file_name| {
                if file_name.ends_with(".json") {
                    Some(chats.chats_dir.join(file_name))
                } else {
                    None
                }
            });

        chats.saved_chats = futures::stream::iter(paths)
            .filter_map(|path| async move {
                match Chat::load(&path).await {
                    Ok(chat) => Some(RefCell::new(chat)),
                    Err(e) => {
                        log::error!("Failed to load chat from path {:?}: {}", path, e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;

        chats
    }

    pub fn get_last_selected_chat_id(&self) -> Option<ChatID> {
        self.saved_chats
            .iter()
            .max_by_key(|c| c.borrow().accessed_at)
            .map(|c| c.borrow().id)
    }

    pub fn get_current_chat_id(&self) -> Option<ChatID> {
        self.current_chat_id
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

    pub fn get_chat_by_id(&self, chat_id: ChatID) -> Option<&RefCell<Chat>> {
        self.saved_chats.iter().find(|c| c.borrow().id == chat_id)
    }

    pub fn set_current_chat(&mut self, chat_id: Option<ChatID>) {
        self.current_chat_id = chat_id;

        if let Some(chat) = self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            chat.update_accessed_at();
            chat.save_and_forget();
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
            chat.borrow().save_and_forget();
        }
    }

    pub fn create_empty_chat(&mut self, bot_id: Option<BotId>) -> ChatID {
        let mut new_chat = Chat::new(self.chats_dir.clone());
        let id = new_chat.id;

        if let Some(bot_id) = bot_id {
            new_chat.associated_bot = Some(bot_id);
        } else {
            // Default to the most recently used bot
            if let Some(last_chat_id) = self.get_last_selected_chat_id() {
                if let Some(last_chat) = self.get_chat_by_id(last_chat_id) {
                    new_chat.associated_bot = last_chat.borrow().associated_bot.clone();
                }
            }
        }

        new_chat.save_and_forget();
        self.saved_chats.push(RefCell::new(new_chat));
        self.set_current_chat(Some(id));
        id
    }

    pub fn remove_chat(&mut self, chat_id: ChatID) {
        if self.current_chat_id == Some(chat_id) {
            self.set_current_chat(self.get_last_selected_chat_id());
        }

        let pos = self
            .saved_chats
            .iter()
            .position(|c| c.borrow().id == chat_id)
            .expect("non-existing chat");

        let chat = self.saved_chats.remove(pos);
        chat.borrow().remove_saved_file_and_forget();
    }

    /// Registers a provider to listen to and the provider info.
    ///
    /// When calling this function, the provider will be tested for connectivity and
    /// the models will be fetched.
    pub fn register_provider(
        &mut self,
        provider: Provider,
        provider_syncing_status: &mut ProviderSyncingStatus,
    ) {
        let provider_id = provider.id.clone();
        self.providers.insert(provider.id.clone(), provider.clone());
        self.test_provider_and_fetch_models(&provider_id, provider_syncing_status);
    }

    pub fn test_provider_and_fetch_models(
        &mut self,
        provider_id: &str,
        provider_syncing_status: &mut ProviderSyncingStatus,
    ) {
        // Update syncing status
        if let ProviderSyncingStatus::Syncing(syncing) = provider_syncing_status {
            // If already syncing, increment the total count
            syncing.total += 1;
        } else {
            // Otherwise, start new syncing status with 1 provider
            *provider_syncing_status = ProviderSyncingStatus::Syncing(ProviderSyncing {
                current: 0,
                total: 1,
            });
        }

        // Use the new model fetcher to fetch models directly
        let provider = self.providers.get(provider_id).unwrap();
        fetch_models_for_provider(provider);
    }

    /// Handle the result of a provider fetching models operation.
    ///
    /// Returns true if the provider is MolyServer and the fetching was successful.
    pub fn handle_provider_connection_result(
        &mut self,
        result: ProviderFetchModelsResult,
        preferences: &mut Preferences,
        provider_syncing_status: &mut ProviderSyncingStatus,
    ) -> bool {
        let mut fetched_from_moly_server = false;
        match result {
            ProviderFetchModelsResult::Success(provider_id, mut fetched_models) => {
                // If the provider is part of the predefined list of supported providers,
                // filter the fetched models to only include those that are in the supported models list.
                // Include supported models even if they are not in the fetched models.
                if let Some(supported_provider) =
                    super::supported_providers::load_supported_providers()
                        .iter()
                        .find(|sp| sp.id == provider_id)
                {
                    if let Some(supported_models) = &supported_provider.supported_models {
                        let fetched_names: std::collections::HashSet<String> =
                            fetched_models.iter().map(|m| m.name.clone()).collect();

                        // Filter fetched models to only include supported ones
                        fetched_models.retain(|model| supported_models.contains(&model.name));

                        let address = self.providers.get(&provider_id).unwrap().url.clone();

                        // Add missing supported models
                        for model_name in supported_models {
                            if !fetched_names.contains(model_name) {
                                fetched_models.push(super::providers::ProviderBot {
                                    id: moly_kit::BotId::new(model_name, &address),
                                    name: model_name.clone(),
                                    description: model_name.clone(),
                                    provider_id: provider_id.clone(),
                                    enabled: true,
                                });
                            }
                        }
                    }
                }

                // Update user's preferences for the provider (adding new models if needed)
                if let Some(pref_entry) = preferences
                    .providers_preferences
                    .iter_mut()
                    .find(|pp| pp.id == provider_id)
                {
                    for rm in &fetched_models {
                        let maybe_model = pref_entry
                            .models
                            .iter_mut()
                            .find(|(mname, _)| *mname == rm.name);

                        if maybe_model.is_none() {
                            // Insert with default enabled: true
                            pref_entry.models.push((rm.name.clone(), true));
                        }
                    }
                    // Remove stale model names from preferences if needed
                    pref_entry
                        .models
                        .retain(|(mname, _)| fetched_models.iter().any(|rm| rm.name == *mname));

                    preferences.save();
                }

                // Insert the fetched models in memory, respecting preference "enabled" if it exists
                for mut provider_bot in fetched_models {
                    if let Some(pref_entry) = preferences
                        .providers_preferences
                        .iter()
                        .find(|pp| pp.id == provider_id)
                    {
                        // if there's a matching "(model_name, enabled)" in preferences, apply it
                        if let Some((_m, enabled_val)) = pref_entry
                            .models
                            .iter()
                            .find(|(m, _)| *m == provider_bot.name)
                        {
                            provider_bot.enabled = *enabled_val;
                        }
                    }

                    // Add it to the provider record, only if it's not already in there
                    if !self
                        .providers
                        .get(&provider_id)
                        .unwrap()
                        .models
                        .contains(&provider_bot.id)
                    {
                        self.providers
                            .get_mut(&provider_id)
                            .unwrap()
                            .models
                            .push(provider_bot.id.clone());
                    }

                    // Add to the global available_bots only if it's not already in there
                    if !self.available_bots.contains_key(&provider_bot.id) {
                        self.available_bots
                            .insert(provider_bot.id.clone(), provider_bot);
                    }
                }

                if let Some(provider) = self.providers.get_mut(&provider_id) {
                    provider.connection_status = ProviderConnectionStatus::Connected;
                    // If the fetching was successful and the provider is MolyServer, sync status
                    if provider.provider_type == ProviderType::MolyServer {
                        fetched_from_moly_server = true;
                    }
                }
            }
            ProviderFetchModelsResult::Failure(address, error) => {
                let provider_id = address.clone();
                if let Some(provider) = self.providers.get_mut(&provider_id) {
                    provider.connection_status =
                        ProviderConnectionStatus::from_client_error(&error);
                }
            }
            _ => {}
        }

        match provider_syncing_status {
            // Increase the current count of providers being synced, regardless of the result
            // We just care to know that we've already got a response for each provider
            ProviderSyncingStatus::Syncing(syncing) => {
                let new_current = syncing.current + 1;
                if new_current < syncing.total {
                    syncing.current = new_current;
                } else {
                    *provider_syncing_status = ProviderSyncingStatus::Synced;
                }
            }
            _ => {}
        }

        fetched_from_moly_server
    }

    /// Inserts or updates a provider in the list of providers.
    ///
    /// If the provider is already in the list, it updates the provider info and the client.
    /// If the provider is not in the list, it registers the provider and creates a new client.
    ///
    /// Automatically tests the provider and fetches models, both on new providers and on API key changes.
    pub fn insert_or_update_provider(
        &mut self,
        provider: &Provider,
        provider_syncing_status: &mut ProviderSyncingStatus,
    ) {
        // Ensure provider has an ID
        let mut provider = provider.clone();
        if provider.id.is_empty() {
            // Generate ID for backward compatibility
            provider.id = super::preferences::ProviderPreferences::generate_id_from_url_and_name(
                &provider.url,
                &provider.name,
                &provider.provider_type,
            );
        }

        // If the provider is already in the list update it
        if let Some(existing_provider) = self.providers.get_mut(&provider.id) {
            existing_provider.url = provider.url.clone();
            existing_provider.api_key = provider.api_key.clone();
            existing_provider.provider_type = provider.provider_type.clone();
            existing_provider.enabled = provider.enabled;
            existing_provider.models = provider.models.clone();
            existing_provider.connection_status = provider.connection_status.clone();

            if provider.enabled {
                self.test_provider_and_fetch_models(&provider.id, provider_syncing_status);
            }
        } else {
            self.register_provider(provider.clone(), provider_syncing_status);
        }
    }

    pub fn remove_provider(&mut self, provider_id: &ProviderID) {
        self.available_bots
            .retain(|_, model| model.provider_id != *provider_id);
        self.providers.remove(provider_id);
    }

    /// Returns a list of remote models for a given server address.
    pub fn get_provider_models(&self, provider_id: &ProviderID) -> Vec<ProviderBot> {
        let provider = self.providers.get(provider_id);

        if let Some(provider) = provider {
            provider
                .models
                .iter()
                .map(|id| self.available_bots.get(id).unwrap().clone())
                .collect()
        } else {
            vec![]
        }
    }

    /// Returns a reference to a remote model by ID, falling back to an unknown remote model placeholder
    /// if the remote model is not found in the available remote models list.
    ///
    /// This is useful when dealing with historical chat references to remote models that may
    /// no longer be available (e.g., server disconnected or remote model deleted).
    ///
    /// In the future, we'll want a more sophisticated solution, by potentially storing
    /// remote model information locally and updating it when a server is connected.
    pub fn get_bot_or_placeholder(&self, bot_id: &BotId) -> &ProviderBot {
        self.available_bots.get(bot_id).unwrap_or(&self.unknown_bot)
    }

    pub fn get_bot(&self, bot_id: &BotId) -> Option<&ProviderBot> {
        self.available_bots.get(bot_id)
    }

    pub fn get_bot_provider(&self, bot_id: &BotId) -> Option<&Provider> {
        if let Some(bot) = self.available_bots.get(bot_id) {
            self.providers.get(&bot.provider_id)
        } else {
            None
        }
    }

    /// Returns a list of all available agents.
    ///
    /// If [enabled_only] is set to true, then only enabled agents from enabled providers are returned.
    pub fn get_mofa_agents_list(&self, enabled_only: bool) -> Vec<ProviderBot> {
        self.available_bots
            .values()
            .filter(|m| {
                self.is_agent(&m.id)
                    && (!enabled_only
                        || (m.enabled
                            && self
                                .providers
                                .get(&m.provider_id)
                                .map_or(false, |p| p.enabled)))
            })
            .cloned()
            .collect()
    }

    /// Returns a list of all available bots
    ///
    /// If [enabled_only] is set to true, then only enabled bots from enabled providers are returned.
    pub fn get_all_bots(&self, enabled_only: bool) -> Vec<ProviderBot> {
        self.available_bots
            .values()
            .filter(|pb| {
                !enabled_only
                    || (pb.enabled
                        && self
                            .providers
                            .get(&pb.provider_id)
                            .map_or(false, |p| p.enabled))
            })
            .cloned()
            .collect()
    }

    /// Returns a list of all available non-MoFa/Agent bots
    ///
    /// If [enabled_only] is set to true, then only enabled bots from enabled providers are returned.
    pub fn get_non_mofa_models_list(&self, enabled_only: bool) -> Vec<ProviderBot> {
        self.available_bots
            .values()
            .filter(|m| {
                !self.is_agent(&m.id)
                    && (!enabled_only
                        || (m.enabled
                            && self
                                .providers
                                .get(&m.provider_id)
                                .map_or(false, |p| p.enabled)))
            })
            .cloned()
            .collect()
    }

    pub fn is_agent(&self, bot_id: &BotId) -> bool {
        if let Some(provider_bot) = self.available_bots.get(bot_id) {
            if let Some(provider) = self.providers.get(&provider_bot.provider_id) {
                provider.provider_type == ProviderType::MoFa
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn is_local_model(&self, bot_id: &BotId) -> bool {
        if let Some(provider_bot) = self.available_bots.get(bot_id) {
            if let Some(provider) = self.providers.get(&provider_bot.provider_id) {
                provider.provider_type == ProviderType::MolyServer
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_bot_id_by_file_id(&self, file_id: &FileID) -> Option<BotId> {
        self.available_bots
            .values()
            .find(|m| m.name == file_id.as_str())
            .map(|m| m.id.clone())
    }
}
