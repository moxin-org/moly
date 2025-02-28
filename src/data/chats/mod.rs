pub mod chat;
pub mod chat_entity;
pub mod model_loader;

use anyhow::{Context, Result};
use chat::{Chat, ChatEntityAction, ChatID};
use chat_entity::ChatEntityId;
use makepad_widgets::{error, ActionDefaultRef, ActionTrait, Cx, DefaultNone};
use model_loader::ModelLoader;
use moly_protocol::data::*;
use moly_protocol::open_ai::ChatResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::{self, channel, Sender};
use std::{cell::RefCell, path::PathBuf};

use super::filesystem::setup_chats_folder;
use super::mofa::MofaClient;
use super::moly_client::MolyClient;
use super::preferences::Preferences;
use super::remote_servers::{OpenAIClient, RemoteModel, RemoteModelId};
use super::store::ProviderType;

/// The connection status of the server
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum ServerConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, DefaultNone, Clone)]
pub enum ProviderTestResultAction {
    Success(String, Vec<RemoteModel>),
    Failure(Option<String>),
    None,
}

pub enum ProviderConnectionResult {
    Connected(String, Vec<RemoteModel>),
    Unavailable(String),
}


#[derive(Clone, Debug)]
pub enum ChatResponse {
    // https://platform.openai.com/docs/api-reference/chat/object
    ChatFinalResponseData(ChatResponseData),
}


pub trait ProviderClient: Send + Sync {
    fn cancel_task(&self);
    fn fetch_models(&self, tx: Sender<ProviderConnectionResult>);
    fn send_message(&self, model: &RemoteModel, prompt: &String, tx: Sender<ChatResponse>);
}

pub struct Chats {
    pub moly_client: MolyClient,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub loaded_model: Option<File>,
    pub model_loader: ModelLoader,

    pub remote_models: HashMap<RemoteModelId, RemoteModel>,

    pub provider_clients: HashMap<String, Box<dyn ProviderClient>>,

    pub providers: HashMap<String, Provider>,

    /// Templates for providers, used to create new providers when registering.
    pub providers_templates: HashMap<String, Provider>,

    /// Set it thru `set_current_chat` method to trigger side effects.
    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,

    override_port: Option<u16>,

    /// Placeholder remote model used when a remote model is not available
    /// This is used to avoid recreating it on each call and make borrowing simpler.
    unknown_remote_model: RemoteModel,
}

impl Chats {
    pub fn new(moly_client: MolyClient) -> Self {
        Self {
            moly_client,
            saved_chats: Vec::new(),
            current_chat_id: None,
            loaded_model: None,
            model_loader: ModelLoader::new(),
            chats_dir: setup_chats_folder(),
            override_port: None,
            // mofa_servers: HashMap::new(),
            // available_agents: HashMap::new(),
            remote_models: HashMap::new(),
            // openai_servers: HashMap::new(),
            provider_clients: HashMap::new(),
            providers: HashMap::new(),
            providers_templates: HashMap::new(),
            // unknown_agent: MofaAgent::unknown(),
            unknown_remote_model: RemoteModel::unknown(),
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

    pub fn load_model(&mut self, file: &File, override_port: Option<u16>) {
        self.cancel_chat_streaming();

        if self.model_loader.is_loading() {
            return;
        }

        self.override_port = override_port;
        self.model_loader.load_async(
            file.id.clone(),
            self.moly_client.clone(),
            override_port,
        );
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
        self.cancel_chat_streaming();
        self.current_chat_id = chat_id;

        if let Some(chat) = self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            chat.update_accessed_at();
            chat.save();
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            match &chat.associated_entity {
                Some(ChatEntityId::ModelFile(_)) => {
                    chat.cancel_streaming();
                }
                Some(ChatEntityId::Agent(agent_id)) => {
                    if let Some(provider_client) = self.get_client_for_provider(&agent_id.0) {
                        chat.cancel_interaction(provider_client.as_ref());
                    } else {
                        error!("No provider client found for agent: {}", agent_id.0);
                    }
                }
                Some(ChatEntityId::RemoteModel(model_id)) => {
                    if let Some(provider_client) = self.get_client_for_provider(&model_id.0) {
                        chat.cancel_interaction(provider_client.as_ref());
                    } else {
                        error!("No provider client found for model: {}", model_id.0);
                    }
                }
                _ => {}
            }
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

        self.loaded_model = None;
        Ok(())
    }

    pub fn remove_file_from_associated_entity(&mut self, file_id: &FileID) {
        for chat in &self.saved_chats {
            let mut chat = chat.borrow_mut();
            if let Some(ChatEntityId::ModelFile(chat_file_id)) = &chat.associated_entity {
                if chat_file_id == file_id {
                    chat.associated_entity = None;
                    chat.save();
                }
            }
        }
    }

    /// Get the file id to use with this chat, or the loaded file id as a fallback.
    /// The fallback is used if the chat does not have a file id set, or, if it has
    /// one but references a no longer existing (deleted) file.
    #[allow(dead_code)]
    pub fn get_chat_file_id(&self, chat: &mut Chat) -> Option<FileID> {
        match &chat.associated_entity {
            Some(ChatEntityId::ModelFile(file_id)) => Some(file_id.clone()),
            _ => {
                let file_id = self.loaded_model.as_ref().map(|m| m.id.clone())?;
                Some(file_id)
            }
        }
    }

    pub fn create_empty_chat(&mut self) {
        let mut new_chat = Chat::new(self.chats_dir.clone());
        let id = new_chat.id;
        new_chat.associated_entity = self
            .loaded_model
            .as_ref()
            .map(|m| ChatEntityId::ModelFile(m.id.clone()));

        new_chat.save();
        self.saved_chats.push(RefCell::new(new_chat));
        self.set_current_chat(Some(id));
    }

    pub fn create_empty_chat_with_agent(&mut self, agent_id: &RemoteModelId) {
        self.create_empty_chat();
        if let Some(mut chat) = self.get_current_chat().map(|c| c.borrow_mut()) {
            chat.associated_entity = Some(ChatEntityId::Agent(agent_id.clone()));
            chat.save();
        }
    }

    pub fn create_empty_chat_with_remote_model(&mut self, model_id: &RemoteModelId) {
        self.create_empty_chat();
        if let Some(mut chat) = self.get_current_chat().map(|c| c.borrow_mut()) {
            chat.associated_entity = Some(ChatEntityId::RemoteModel(model_id.clone()));
            chat.save();
        }
    }

    pub fn create_empty_chat_and_load_file(&mut self, file: &File) {
        let mut new_chat = Chat::new(self.chats_dir.clone());
        let id = new_chat.id;
        new_chat.associated_entity = Some(ChatEntityId::ModelFile(file.id.clone()));
        new_chat.save();

        self.saved_chats.push(RefCell::new(new_chat));
        self.set_current_chat(Some(id));

        self.load_model(file, None);
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

    pub fn handle_action(&mut self, action: &Box<dyn ActionTrait>) {
        if let Some(action) = action.downcast_ref::<ChatEntityAction>() {
            if let Some(chat) = self.get_chat_by_id(action.chat_id) {
                if chat.borrow().id == action.chat_id {
                    chat.borrow_mut().handle_action(action);
                }
            }
        }
    }

    pub fn register_provider(&mut self, provider: Provider) {
        let client: Box<dyn ProviderClient> = match &provider.provider_type {
            ProviderType::OpenAIAPI => Box::new(OpenAIClient::new(provider.url.clone(), provider.api_key.clone())),
            ProviderType::MoFa => Box::new(MofaClient::new(provider.url.clone())),
        };
        self.provider_clients.insert(provider.url.clone(), client);
        self.providers.insert(provider.url.clone(), provider.clone());
        self.test_provider_and_fetch_models(&provider.url);
    }

    pub fn test_provider_and_fetch_models(&mut self, address: &str) {
        if let Some(client) = self.provider_clients.get(address) {
            let (tx, rx) = mpsc::channel();
            client.fetch_models(tx);

            std::thread::spawn(move || match rx.recv() {
                Ok(ProviderConnectionResult::Connected(server_address, remote_models)) => {
                    Cx::post_action(ProviderTestResultAction::Success(
                        server_address, 
                        remote_models
                    ));
                }
                Ok(ProviderConnectionResult::Unavailable(server_address)) => {
                    Cx::post_action(ProviderTestResultAction::Failure(Some(server_address)));
                }
                Err(_) => {
                    Cx::post_action(ProviderTestResultAction::Failure(None));
                }
            });
        }
    }

    pub fn handle_provider_connection_result(
        &mut self,
        result: ProviderConnectionResult,
        preferences: &mut Preferences
    ) {
        match result {
            ProviderConnectionResult::Connected(address, fetched_models) => {
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
                for mut remote_model in fetched_models {
                    if let Some(pref_entry) = preferences.providers_preferences
                        .iter()
                        .find(|pp| pp.url == address)
                    {
                        // if there's a matching "(model_name, enabled)" in preferences, apply it
                        if let Some((_m, enabled_val)) = pref_entry.models
                            .iter()
                            .find(|(m, _)| *m == remote_model.name)
                        {
                            remote_model.enabled = *enabled_val;
                        }
                    }

                    // Add it to the provider record
                    self.providers.get_mut(&address)
                        .unwrap()
                        .models.push(remote_model.id.clone());

                    // Add to the global remote_models
                    self.remote_models.insert(remote_model.id.clone(), remote_model);
                }

                if let Some(provider) = self.providers.get_mut(&address) {
                    provider.connection_status = ServerConnectionStatus::Connected;
                }
            }
            ProviderConnectionResult::Unavailable(address) => {
                if let Some(provider) = self.providers.get_mut(&address) {
                    provider.connection_status = ServerConnectionStatus::Disconnected;
                }
            }
        }
    }

    // Agents

    /// Removes a MoFa server from the list of available servers.
    pub fn remove_mofa_server(&mut self, address: &str) {
        // self.mofa_servers.remove(&MofaServerId(address.to_string()));
        self.provider_clients.remove(address);
        self.remote_models.retain(|_, model| model.provider_url != address);
        self.providers.remove(address);
    }

    /// Removes a OpenAI server from the list of available servers.
    pub fn remove_openai_server(&mut self, address: &str) {
        // self.openai_servers.remove(address);
        self.provider_clients.remove(address);
        self.remote_models.retain(|_, model| model.provider_url != address);
        self.providers.remove(address);
    }

    pub fn get_client_for_provider(&self, provider_url: &str) -> Option<&Box<dyn ProviderClient>> {
        self.provider_clients.get(provider_url)
    }

    /// Returns a list of remote models for a given server address.
    pub fn get_provider_models(&self, server_url: &str) -> Vec<RemoteModel> {
        if let Some(provider) = self.providers.get(server_url) {
            provider.models.iter().map(|id| self.remote_models.get(id).unwrap().clone()).collect()
        } else {
            vec![]
        }
    }

    // TODO(Julian): Clean this up
    pub fn agents_availability(&self) -> AgentsAvailability {
        let no_mofa_providers = self.providers.iter().filter(|(_, p)| p.provider_type == ProviderType::MoFa).count() == 0;
        let providers_but_no_agents = self.providers.iter().filter(|(_, p)| p.connection_status == ServerConnectionStatus::Connected).all(|(_, p)| p.models.is_empty());

        if no_mofa_providers {
            AgentsAvailability::NoServers
        } else if providers_but_no_agents {
            // Check the reason for the lack of agents, is it disconnected servers or servers with no agents?
            if self.providers.iter().all(|(_id, p)| p.connection_status == ServerConnectionStatus::Connected) {
                AgentsAvailability::NoAgents
            } else {
                AgentsAvailability::ServersNotConnected
            }
        } else {
            AgentsAvailability::Available
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
    pub fn get_remote_model_or_placeholder(&self, model_id: &RemoteModelId) -> &RemoteModel {
        self.remote_models.get(model_id).unwrap_or(&self.unknown_remote_model)
    }

    pub fn get_mofa_agents_list(&self) -> Vec<RemoteModel> {
        self.remote_models.values().filter(|m| self.is_agent(m)).cloned().collect()
    }

    pub fn is_agent(&self, model: &RemoteModel) -> bool {
        self.providers.get(&model.provider_url).map_or(false, |p| p.provider_type == ProviderType::MoFa)
    }
}

pub enum AgentsAvailability {
    Available,
    NoAgents,
    NoServers,
    ServersNotConnected,
}

impl AgentsAvailability {
    pub fn to_human_readable(&self) -> &'static str {
        match self {
            AgentsAvailability::Available => "Agents available",
            AgentsAvailability::NoAgents => "No agents found in the connected servers.",
            AgentsAvailability::NoServers => "Not connected to any MoFa servers.",
            AgentsAvailability::ServersNotConnected => "Could not connect to some servers. Check your MoFa settings.",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub provider_type: ProviderType,
    pub connection_status: ServerConnectionStatus,
    pub models: Vec<RemoteModelId>,
}
