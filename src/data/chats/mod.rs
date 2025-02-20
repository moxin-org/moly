pub mod chat;
pub mod chat_entity;
pub mod model_loader;

use anyhow::{Context, Result};
use chat::{Chat, ChatEntityAction, ChatID};
use chat_entity::ChatEntityId;
use makepad_widgets::{error, ActionDefaultRef, ActionTrait, Cx, DefaultNone};
use model_loader::ModelLoader;
use moly_mofa::{AgentId, MofaAgent, MofaClient, MofaServerId, MofaServerResponse};
use moly_protocol::data::*;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::{self, channel};
use std::{cell::RefCell, path::PathBuf};

use super::filesystem::setup_chats_folder;
use super::moly_client::MolyClient;
use super::remote_servers::{OpenAIClient, OpenAIServerResponse, RemoteModel, RemoteModelId};

#[derive(Clone, Debug)]
pub struct MofaServer {
    pub client: MofaClient,
    pub connection_status: ServerConnectionStatus,
}

impl MofaServer {
    pub fn is_local(&self) -> bool {
        self.client.address.starts_with("http://localhost")
    }
}

/// The connection status of the server
#[derive(Debug, Clone, PartialEq)]
pub enum ServerConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, DefaultNone, Clone)]
pub enum MoFaTestServerAction {
    Success(String, Vec<MofaAgent>),
    Failure(Option<String>),
    None,
}

#[derive(Debug, DefaultNone, Clone)]
pub enum OpenAiTestServerAction {
    Success(String, Vec<RemoteModel>),
    Failure(Option<String>),
    None,
}

pub struct Chats {
    pub moly_client: MolyClient,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub loaded_model: Option<File>,
    pub model_loader: ModelLoader,

    pub mofa_servers: HashMap<MofaServerId, MofaServer>,
    pub available_agents: HashMap<AgentId, MofaAgent>,

    pub remote_models: HashMap<ModelID, RemoteModel>,
    pub openai_servers: HashMap<String, OpenAIClient>,

    /// Set it thru `set_current_chat` method to trigger side effects.
    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,

    override_port: Option<u16>,

    /// Placeholder agent used when an agent is not available
    /// This is used to avoid recreating it on each call and make borrowing simpler.
    unknown_agent: MofaAgent,

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
            mofa_servers: HashMap::new(),
            available_agents: HashMap::new(),
            remote_models: HashMap::new(),
            openai_servers: HashMap::new(),
            unknown_agent: MofaAgent::unknown(),
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
                    if let Some(mofa_client) = self.get_client_for_agent(&agent_id) {
                        chat.cancel_agent_interaction(&mofa_client);
                    } else {
                        error!("No mofa client found for agent: {}", agent_id.0);
                    }
                }
                Some(ChatEntityId::RemoteModel(model_id)) => {
                    if let Some(openai_client) = self.get_client_for_remote_model(&model_id.0) {
                        chat.cancel_remote_model_interaction(&openai_client);
                    } else {
                        error!("No openai client found for model: {}", model_id.0);
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

    pub fn create_empty_chat_with_agent(&mut self, agent_id: &AgentId) {
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

    pub fn register_server(&mut self, server_type: ServerType) {
        match server_type {
            ServerType::Mofa(address) => self.register_mofa_server(address),
            ServerType::OpenAI { address, api_key } => self.register_openai_server(address, api_key),
        }
    }

    pub fn register_openai_server(&mut self, address: String, api_key: String) {
        let client = OpenAIClient::with_api_key(address.clone(), api_key);
        self.openai_servers.insert(address.clone(), client);
        self.test_openai_server_and_fetch_models(&address);
    }

    pub fn test_openai_server_and_fetch_models(&mut self, address: &str) {
        if let Some(client) = self.openai_servers.get(address) {
            let (tx, rx) = mpsc::channel();
            client.fetch_agents(tx);

            std::thread::spawn(move || match rx.recv() {
                Ok(OpenAIServerResponse::Connected(server_address, remote_models)) => {
                    Cx::post_action(OpenAiTestServerAction::Success(
                        server_address,
                        remote_models
                    ));
                }
                Ok(OpenAIServerResponse::Unavailable(server_address)) => {
                    Cx::post_action(OpenAiTestServerAction::Failure(Some(server_address)));
                }
                Err(_) => {
                    Cx::post_action(OpenAiTestServerAction::Failure(None));
                }
            });
        }
    }

    pub fn handle_openai_server_connection_result(&mut self, result: OpenAIServerResponse) {
        match result {
            OpenAIServerResponse::Connected(address, models) => {
                if let Some(server) = self.openai_servers.get_mut(&address) {
                    server.connection_status = ServerConnectionStatus::Connected;

                    for remote_model in models {
                        self.remote_models.insert(remote_model.id.0.clone(), remote_model);
                    }
                }
            }
            OpenAIServerResponse::Unavailable(address) => {
                error!("Failed to connect to OpenAI-compatible server at {}", address);
            }
        }
    }

    // Agents

    /// Registers a new MoFa server by creating a new client, automatically testing the connection
    /// and fetching the available agents.
    pub fn register_mofa_server(&mut self, address: String) {
        let server_id = MofaServerId(address.clone());
        let client = MofaClient::new(address.clone());
        
        self.mofa_servers.insert(server_id.clone(), MofaServer {
            client,
            connection_status: ServerConnectionStatus::Connecting,
        });

        self.test_mofa_server_and_fetch_agents(&address);
    }

    /// Removes a MoFa server from the list of available servers.
    pub fn remove_mofa_server(&mut self, address: &str) {
        self.mofa_servers.remove(&MofaServerId(address.to_string()));
        self.available_agents.retain(|_, agent| agent.server_id.0 != address);
    }

    /// Retrieves the corresponding MofaClient for an agent
    pub fn get_client_for_agent(&self, agent_id: &AgentId) -> Option<&MofaClient> {
        self.available_agents.get(agent_id)
            .and_then(|agent| self.mofa_servers.get(&agent.server_id))
            .map(|server| &server.client)
    }

    /// Retrieves the corresponding OpenAIClient for a remote model
    pub fn get_client_for_remote_model(&self, model_id: &ModelID) -> Option<&OpenAIClient> {
        self.remote_models.get(model_id)
            .and_then(|model| self.openai_servers.get(&model.server_id.0))
    }

    /// Helper method for components that need a sorted vector of agents
    pub fn get_agents_list(&self) -> Vec<MofaAgent> {
        let mut agents: Vec<_> = self.available_agents.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        agents
    }

    /// Helper method for components that need a sorted vector of remote models
    pub fn get_remote_models_list(&self) -> Vec<RemoteModel> {
        let mut models: Vec<_> = self.remote_models.values().cloned().collect();
        models.sort_by(|a, b| a.name.cmp(&b.name));
        models
    }
    /// Tests the connection to a MoFa server by requesting /v1/models.
    ///
    /// The connection status is updated at the App level based on the actions dispatched.
    pub fn test_mofa_server_and_fetch_agents(&mut self, address: &String) {
        self.mofa_servers.get_mut(&MofaServerId(address.to_string())).unwrap().connection_status = ServerConnectionStatus::Connecting;
        let (tx, rx) = mpsc::channel();
        if let Some(server) = self.mofa_servers.get(&MofaServerId(address.to_string())) {
            server.client.fetch_agents(tx.clone());
        }

        std::thread::spawn(move || match rx.recv() {
            Ok(MofaServerResponse::Connected(server_address, agents)) => {
                Cx::post_action(MoFaTestServerAction::Success(server_address, agents));
            }
            Ok(MofaServerResponse::Unavailable(server_address)) => {
                Cx::post_action(MoFaTestServerAction::Failure(Some(server_address)));
            }
            Err(e) => {
                error!("Error receiving response from MoFa backend: {:?}", e);
                Cx::post_action(MoFaTestServerAction::Failure(None));
            }
        });
    }

    pub fn handle_mofa_server_connection_result(&mut self, result: MofaServerResponse) {
        match result {
            MofaServerResponse::Connected(address, agents) => {
                if let Some(server) = self.mofa_servers.get_mut(&MofaServerId(address.clone())) {
                    server.connection_status = ServerConnectionStatus::Connected;

                    for agent in agents {
                        self.available_agents.insert(agent.id.clone(), agent);
                    }
                }
            }
            MofaServerResponse::Unavailable(address) => {
                if let Some(server) = self.mofa_servers.get_mut(&MofaServerId(address)) {
                    server.connection_status = ServerConnectionStatus::Disconnected;
                }
            }
        }
    }

    pub fn agents_availability(&self) -> AgentsAvailability {
        if self.mofa_servers.is_empty() {
            AgentsAvailability::NoServers
        } else if self.available_agents.is_empty() {
            // Check the reason for the lack of agents, is it disconnected servers or servers with no agents?
            if self.mofa_servers.iter().all(|(_id, s)| s.connection_status == ServerConnectionStatus::Connected) {
                AgentsAvailability::NoAgents
            } else {
                AgentsAvailability::ServersNotConnected
            }
        } else {
            AgentsAvailability::Available
        }
    }

    /// Returns a reference to an agent by ID, falling back to an unknown agent placeholder
    /// if the agent is not found in the available agents list.
    /// 
    /// This is useful when dealing with historical chat references to agents that may
    /// no longer be available (e.g., server disconnected or agent deleted).
    /// 
    /// In the future, we'll want a more sophisticated solution, by potentially storing 
    /// agent information locally and updating it when a server is connected.
    pub fn get_agent_or_placeholder(&self, agent_id: &AgentId) -> &MofaAgent {
        self.available_agents.get(agent_id).unwrap_or(&self.unknown_agent)
    }

    pub fn get_remote_model_or_placeholder(&self, model_id: &RemoteModelId) -> &RemoteModel {
        self.remote_models.get(&model_id.0).unwrap_or(&self.unknown_remote_model)
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

pub enum ServerType {
    Mofa(String),
    OpenAI {
        address: String,
        api_key: String,
    },
}
