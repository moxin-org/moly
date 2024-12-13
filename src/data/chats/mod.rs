pub mod chat;
pub mod chat_entity;
pub mod model_loader;

use anyhow::{Context, Result};
use chat::{Chat, ChatEntityAction, ChatID};
use chat_entity::ChatEntityId;
use makepad_widgets::{error, ActionDefaultRef, ActionTrait, Cx, DefaultNone};
use model_loader::ModelLoader;
use moly_backend::Backend;
use moly_mofa::{AgentId, MofaAgent, MofaClient, MofaServerId, TestServerResponse};
use moly_protocol::data::*;
use moly_protocol::protocol::Command;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::{self, channel};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use super::filesystem::setup_chats_folder;

#[derive(Clone, Debug)]
pub struct MofaServer {
    pub address: String,
    pub client: Rc<MofaClient>,
    pub connection_status: MofaServerConnectionStatus,
}

impl MofaServer {
    pub fn is_local(&self) -> bool {
        self.address.starts_with("http://localhost")
    }
}

/// The connection status of the server
#[derive(Debug, Clone, PartialEq)]
pub enum MofaServerConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, DefaultNone, Clone)]
pub enum MoFaTestServerAction {
    Success(String),
    Failure(Option<String>),
    None,
}

pub struct Chats {
    pub backend: Rc<Backend>,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub loaded_model: Option<File>,
    pub model_loader: ModelLoader,

    pub mofa_servers: HashMap<MofaServerId, MofaServer>,
    pub available_agents: HashMap<AgentId, MofaAgent>,

    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,

    override_port: Option<u16>,
}

impl Chats {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            saved_chats: Vec::new(),
            current_chat_id: None,
            loaded_model: None,
            model_loader: ModelLoader::new(),
            chats_dir: setup_chats_folder(),
            override_port: None,
            mofa_servers: HashMap::new(),
            available_agents: HashMap::new(),
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
            self.backend.command_sender.clone(),
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

    pub fn set_current_chat(&mut self, chat_id: ChatID) {
        self.cancel_chat_streaming();
        self.current_chat_id = Some(chat_id);

        let mut chat = self.get_current_chat().unwrap().borrow_mut();
        chat.update_accessed_at();
        chat.save();
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            match &chat.associated_entity {
                Some(ChatEntityId::ModelFile(_)) => {
                    chat.cancel_streaming(self.backend.as_ref());
                }
                Some(ChatEntityId::Agent(agent_id)) => {
                    if let Some(mofa_client) = self.get_client_for_agent(&agent_id) {
                        chat.cancel_agent_interaction(&mofa_client);
                    } else {
                        error!("No mofa client found for agent: {}", agent_id.0);
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
        self.backend
            .as_ref()
            .command_sender
            .send(Command::EjectModel(tx))
            .context("Failed to send eject model command")?;

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
        new_chat.associated_entity = self
            .loaded_model
            .as_ref()
            .map(|m| ChatEntityId::ModelFile(m.id.clone()));

        new_chat.save();
        self.current_chat_id = Some(new_chat.id);
        self.saved_chats.push(RefCell::new(new_chat));
    }

    pub fn create_empty_chat_with_agent(&mut self, agent: &MofaAgent) {
        self.create_empty_chat();
        if let Some(mut chat) = self.get_current_chat().map(|c| c.borrow_mut()) {
            chat.associated_entity = Some(ChatEntityId::Agent(agent.id.clone()));
            chat.save();
        }
    }

    pub fn create_empty_chat_and_load_file(&mut self, file: &File) {
        let mut new_chat = Chat::new(self.chats_dir.clone());
        new_chat.associated_entity = Some(ChatEntityId::ModelFile(file.id.clone()));
        new_chat.save();

        self.current_chat_id = Some(new_chat.id);
        self.saved_chats.push(RefCell::new(new_chat));

        self.load_model(file, None);
    }

    pub fn remove_chat(&mut self, chat_id: ChatID) {
        if let Some(chat) = self.saved_chats.iter().find(|c| c.borrow().id == chat_id) {
            chat.borrow().remove_saved_file();
        };
        self.saved_chats.retain(|c| c.borrow().id != chat_id);

        if let Some(current_chat_id) = self.current_chat_id {
            if current_chat_id == chat_id {
                self.current_chat_id = self.get_last_selected_chat_id();
            }
        }
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

    // Agents

    /// Registers a new MoFa server by creating a new client, automatically testing the connection
    /// and fetching the available agents.
    pub fn register_mofa_server(&mut self, address: String) -> MofaServerId {
        let server_id = MofaServerId(address.clone());
        let client = Rc::new(MofaClient::new(address.clone()));
        
        self.mofa_servers.insert(server_id.clone(), MofaServer {
            address: address.clone(),
            client,
            connection_status: MofaServerConnectionStatus::Disconnected,
        });

        // TODO: once MoFa servers properly support fetching agents, we can rework these 
        // two steps into one.
        self.test_mofa_server_connection(&address);
        self.fetch_agents_from_server(&server_id);
        
        server_id
    }

    pub fn fetch_agents_from_server(&mut self, server_id: &MofaServerId) {
        let server = self.mofa_servers.get(server_id).unwrap();
        let agents = server.client.get_available_agents();
        for agent in agents {
            self.available_agents.insert(agent.id.clone(), agent);
        }
    }

    /// Removes a MoFa server from the list of available servers.
    pub fn remove_mofa_server(&mut self, address: &str) {
        self.mofa_servers.remove(&MofaServerId(address.to_string()));
        self.available_agents.retain(|_, agent| agent.server_id.0 != address);
    }

    /// Retrieves the corresponding MofaClient for an agent
    pub fn get_client_for_agent(&self, agent_id: &AgentId) -> Option<Rc<MofaClient>> {
        self.available_agents.get(agent_id)
            .and_then(|agent| self.mofa_servers.get(&agent.server_id))
            .map(|server| server.client.clone())
    }

    /// Helper method for components that need a sorted vector of agents
    pub fn get_agents_list(&self) -> Vec<MofaAgent> {
        let mut agents: Vec<_> = self.available_agents.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        agents
    }

    /// Tests the connection to a MoFa server by requesting /v1/models
    /// The connection status is updated at the App level based on the actions dispatched
    pub fn test_mofa_server_connection(&mut self, address: &String) {
        self.mofa_servers.get_mut(&MofaServerId(address.to_string())).unwrap().connection_status = MofaServerConnectionStatus::Connecting;
        let (tx, rx) = mpsc::channel();
        if let Some(server) = self.mofa_servers.get(&MofaServerId(address.to_string())) {
            server.client.test_connection(tx.clone());
        }

        std::thread::spawn(move || match rx.recv() {
            Ok(TestServerResponse::Success(server_address)) => {
                Cx::post_action(MoFaTestServerAction::Success(server_address));
            }
            Ok(TestServerResponse::Failure(server_address)) => {
                Cx::post_action(MoFaTestServerAction::Failure(Some(server_address)));
            }
            Err(e) => {
                error!("Error receiving response from MoFa backend: {:?}", e);
                Cx::post_action(MoFaTestServerAction::Failure(None));
            }
        });
    }
}
