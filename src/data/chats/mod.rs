pub mod chat;

use anyhow::{Context, Result};
use chat::{Chat, ChatID};
use moly_kit::BotId;
use moly_protocol::data::*;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::channel;
use std::{cell::RefCell, path::PathBuf};

use super::filesystem::setup_chats_folder;
use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::providers::{create_client_for_provider, Provider, ProviderClient, ProviderFetchModelsResult, ProviderType, ProviderBot, ProviderConnectionStatus};

pub struct Chats {
    pub moly_client: MolyClient,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub available_bots: HashMap<BotId, ProviderBot>,

    pub provider_clients: HashMap<String, Box<dyn ProviderClient>>,

    pub providers: HashMap<String, Provider>,

    /// Set it thru `set_current_chat` method to trigger side effects.
    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,

    /// Placeholder remote model used when a remote model is not available
    /// This is used to avoid recreating it on each call and make borrowing simpler.
    unknown_bot: ProviderBot,
}

impl Chats {
    pub fn new(moly_client: MolyClient) -> Self {
        Self {
            moly_client,
            saved_chats: Vec::new(),
            current_chat_id: None,
            chats_dir: setup_chats_folder(),
            available_bots: HashMap::new(),
            provider_clients: HashMap::new(),
            providers: HashMap::new(),
            unknown_bot: ProviderBot::unknown(),
        }
    }

    pub fn load_chats(&mut self) {
        let paths = fs::read_dir(&self.chats_dir).unwrap();

        for path in paths.map(|p| p.unwrap().path()) {
            let loaded_chat_result = Chat::load(path, self.chats_dir.clone());
            match loaded_chat_result {
                Err(e) => {
                    eprintln!("{}", &e.to_string());
                }
                Ok(loaded_chat) => self.saved_chats.push(RefCell::new(loaded_chat)),
            }
        }
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
            chat.save();
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
            chat.borrow().save();
        }
    }

    pub fn eject_model(&mut self) -> Result<()> {
        let (tx, rx) = channel();
        self.moly_client.eject_model(tx);

        let _ = rx
            .recv()
            .context("Failed to receive eject model response")?
            .context("Eject model operation failed");

        Ok(())
    }

    pub fn create_empty_chat(&mut self, bot_id: Option<BotId>) {
        let mut new_chat = Chat::new(self.chats_dir.clone());
        let id = new_chat.id;

        // TODO: A better default bot id, for now we just use the first available one
        new_chat.associated_bot = if bot_id.is_some() {
            bot_id
        } else {
            self.available_bots.keys().next().map(|id| id.clone())
        };

        new_chat.save();
        self.saved_chats.push(RefCell::new(new_chat));
        self.set_current_chat(Some(id));
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
        chat.borrow().remove_saved_file();
    }

    pub fn register_provider(&mut self, provider: Provider) {
        self.providers.insert(provider.url.clone(), provider.clone());
        self.test_provider_and_fetch_models(&provider.url);
    }

    pub fn test_provider_and_fetch_models(&mut self, address: &str) {
        // Use the existing client if found, otherwise create a new one
        let client = if let Some(existing_client) = self.provider_clients.get(address) {
            existing_client
        } else {
            let provider = self.providers.get(address).unwrap();
            self.provider_clients.insert(address.to_string(), create_client_for_provider(provider));
            self.provider_clients.get(address).unwrap()
        };

        client.fetch_models();
    }

    pub fn handle_provider_connection_result(
        &mut self,
        result: ProviderFetchModelsResult,
        preferences: &mut Preferences
    ) {
        match result {
            ProviderFetchModelsResult::Success(address, fetched_models) => {
                // Update user's preferences for the provider (adding new models if needed)
                if let Some(pref_entry) = preferences.providers_preferences
                    .iter_mut()
                    .find(|pp| pp.url == address)
                {
                    for rm in &fetched_models {
                        let maybe_model = pref_entry.models.iter_mut()
                            .find(|(mname, _)| *mname == rm.name);

                        if maybe_model.is_none() {
                            // Insert with default enabled: true
                            pref_entry.models.push((rm.name.clone(), true));
                        }
                    }
                    // Remove stale model names from preferences if needed
                    pref_entry.models.retain(|(mname, _)| {
                        fetched_models.iter().any(|rm| rm.name == *mname)
                    });

                    preferences.save();
                }

                // Insert the fetched models in memory, respecting preference "enabled" if it exists
                for mut provider_bot in fetched_models {
                    if let Some(pref_entry) = preferences.providers_preferences
                        .iter()
                        .find(|pp| pp.url == address)
                    {
                        // if there's a matching "(model_name, enabled)" in preferences, apply it
                        if let Some((_m, enabled_val)) = pref_entry.models
                            .iter()
                            .find(|(m, _)| *m == provider_bot.name)
                        {
                            provider_bot.enabled = *enabled_val;
                        }
                    }

                    // Add it to the provider record, only if it's not already in there
                    if !self.providers.get(&address).unwrap().models.contains(&provider_bot.id) {
                        self.providers.get_mut(&address)
                            .unwrap()
                            .models.push(provider_bot.id.clone());
                    }

                    // Add to the global available_bots only if it's not already in there
                    if !self.available_bots.contains_key(&provider_bot.id) {
                        self.available_bots.insert(provider_bot.id.clone(), provider_bot);
                    }
                }

                if let Some(provider) = self.providers.get_mut(&address) {
                    provider.connection_status = ProviderConnectionStatus::Connected;
                }
            }
            ProviderFetchModelsResult::Failure(address, error) => {
                if let Some(provider) = self.providers.get_mut(&address) {
                    provider.connection_status = ProviderConnectionStatus::Error(error);
                }
            },
            _ => {}
        }
    }

    pub fn insert_or_update_provider(&mut self, provider: &Provider) {
        // If the provider is already in the list update it, and create a new client if there's a new API key
        if let Some(existing_provider) = self.providers.get_mut(&provider.url) {
            existing_provider.api_key = provider.api_key.clone();
            existing_provider.provider_type = provider.provider_type.clone();
            existing_provider.enabled = provider.enabled;
            existing_provider.models = provider.models.clone();
            existing_provider.connection_status = provider.connection_status.clone();
            // Update the client if the API key has changed
            if let Some(_client) = self.provider_clients.get_mut(&provider.url) {
                // TODO: we should instead have a way to update the client api key without recreating it
                // skipping that for now as the client will be replaced by MolyKit
                self.provider_clients.remove(&provider.url);
                self.provider_clients.insert(provider.url.clone(), create_client_for_provider(provider));
            }
        } else {
            self.providers.insert(provider.url.clone(), provider.clone());
            self.provider_clients.insert(provider.url.clone(), create_client_for_provider(provider));
        }
    }

    pub fn remove_provider(&mut self, address: &str) {
        self.provider_clients.remove(address);
        self.available_bots.retain(|_, model| model.provider_url != address);
        self.providers.remove(address);
    }

    // Agents

    /// Removes a MoFa server from the list of available servers.
    pub fn remove_mofa_server(&mut self, address: &str) {
        // self.mofa_servers.remove(&MofaServerId(address.to_string()));
        self.provider_clients.remove(address);
        self.available_bots.retain(|_, model| model.provider_url != address);
        self.providers.remove(address);
    }

    /// Returns a list of remote models for a given server address.
    pub fn get_provider_models(&self, server_url: &str) -> Vec<ProviderBot> {
        if let Some(provider) = self.providers.get(server_url) {
            provider.models.iter().map(|id| self.available_bots.get(id).unwrap().clone()).collect()
        } else {
            vec![]
        }
    }

    pub fn agents_availability(&self) -> AgentsAvailability {
        let mut has_mofa_provider = false;
        let mut has_available_agent = false;
        
        for (_, p) in &self.providers {
            if p.provider_type == ProviderType::MoFa {
                has_mofa_provider = true;
                if !p.models.is_empty() {
                    has_available_agent = true;
                    break; // No need to continue once we've found an available agent
                }
            }
        }
        
        if has_available_agent {
            AgentsAvailability::Available
        } else if !has_mofa_provider {
            AgentsAvailability::NoServers
        } else {
            AgentsAvailability::ServersNotConnected
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

    pub fn get_mofa_agents_list(&self, enabled_only: bool) -> Vec<ProviderBot> {
        self.available_bots.values().filter(|m| self.is_agent(&m.id) && (!enabled_only || m.enabled)).cloned().collect()
    }

    pub fn get_non_mofa_models_list(&self, enabled_only: bool) -> Vec<ProviderBot> {
        self.available_bots.values().filter(|m| !self.is_agent(&m.id) && (!enabled_only || m.enabled)).cloned().collect()
    }

    pub fn is_agent(&self, bot_id: &BotId) -> bool {
       if let Some(provider_bot) = self.available_bots.get(bot_id) {
            if let Some(provider) = self.providers.get(&provider_bot.provider_url) {
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
            if let Some(provider) = self.providers.get(&provider_bot.provider_url) {
                provider.provider_type == ProviderType::MolyServer
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_bot_id_by_file_id(&self, file_id: &FileID) -> Option<BotId> {
        self.available_bots.values().find(|m| m.name == file_id.as_str()).map(|m| m.id.clone())
    }
}

pub enum AgentsAvailability {
    Available,
    NoServers,
    ServersNotConnected,
}

impl AgentsAvailability {
    pub fn to_human_readable(&self) -> &'static str {
        match self {
            AgentsAvailability::Available => "Agents available",
            AgentsAvailability::NoServers => "Not connected to any MoFa servers.",
            AgentsAvailability::ServersNotConnected => "Could not connect to some servers. Check your MoFa settings.",
        }
    }
}
