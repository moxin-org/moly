use super::chats::chat::ChatID;
use super::chats::chat_entity::ChatEntityId;
use super::chats::model_loader::ModelLoaderStatusChanged;
use super::downloads::download::DownloadFileAction;
use super::filesystem::project_dirs;
use super::preferences::Preferences;
use super::search::SortCriteria;
use super::{chats::Chats, downloads::Downloads, search::Search};
use anyhow::Result;
use chrono::{DateTime, Utc};
use makepad_widgets::{error, Action, ActionDefaultRef, Cx, DefaultNone};
use moly_backend::Backend;
use moly_mofa::{AgentId, MofaServerId};
use moly_mofa::{
    MofaAgent,
    MofaClient, TestServerResponse,
};
use moly_protocol::data::{Author, DownloadedFile, File, FileID, Model, ModelID, PendingDownload};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

pub const DEFAULT_MAX_DOWNLOAD_THREADS: usize = 3;
const DEFAULT_MOFA_ADDRESS: &str = "http://localhost:8000";

#[derive(Clone, DefaultNone, Debug)]
pub enum StoreAction {
    Search(String),
    ResetSearch,
    Sort(SortCriteria),
    None,
}

#[derive(Debug, DefaultNone, Clone)]
pub enum MoFaTestServerAction {
    Success(String),
    Failure(Option<String>),
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
    pub backend: Rc<Backend>,


    pub search: Search,
    pub downloads: Downloads,
    pub chats: Chats,
    pub preferences: Preferences,
    pub mofa_servers: HashMap<MofaServerId, ServerInfo>,
    pub available_agents: HashMap<AgentId, MofaAgent>,
}

#[derive(Clone, Debug)]
pub struct ServerInfo {
    pub address: String,
    pub client: Rc<MofaClient>,
    pub connection_status: MofaServerConnectionStatus,
    // TODO(Julian): remove this
    pub available_agent_ids: Vec<AgentId>,
}

/// The connection status of the server
#[derive(Debug, Clone, PartialEq)]
pub enum MofaServerConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    // Disconnecting,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let preferences = Preferences::load();
        let app_data_dir = project_dirs().data_dir();

        let backend = Rc::new(Backend::new(
            app_data_dir,
            preferences.downloaded_files_dir.clone(),
            DEFAULT_MAX_DOWNLOAD_THREADS,
        ));

        // let mut mofa_clients = HashMap::new();
        // mofa_clients.insert(DEFAULT_MOFA_ADDRESS.to_string(), Rc::new(MofaClient::new(DEFAULT_MOFA_ADDRESS.to_string())));

        let mut store = Self {
            backend: backend.clone(),

            search: Search::new(backend.clone()),
            downloads: Downloads::new(backend.clone()),
            chats: Chats::new(backend),
            preferences,
            mofa_servers: HashMap::new(),
            available_agents: HashMap::new(),
        };

        store.downloads.load_downloaded_files();
        store.downloads.load_pending_downloads();

        store.chats.load_chats();
        store.init_current_chat();

        store.search.load_featured_models();

        if moly_mofa::should_be_real() && moly_mofa::should_be_visible() {
            store.register_mofa_server(DEFAULT_MOFA_ADDRESS.to_string());
        }
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
                ChatEntityId::Agent(agent_id) => {
                    if let (Some(client), Some(agent)) = (
                        self.get_client_for_agent(agent_id),
                        self.available_agents.get(agent_id)
                    ) {
                        chat.send_message_to_agent(agent, prompt, &client);
                    } else {
                        println!("client or agent not found: {:?}", agent_id);
                    }
                }
                ChatEntityId::ModelFile(file_id) => {
                    if let Some(file) = self.downloads.get_file(&file_id) {
                        chat.send_message_to_model(
                            prompt,
                            file,
                            self.chats.model_loader.clone(),
                            &self.backend,
                        );
                    }
                }
            }
        }
    }

    pub fn agents_list(&mut self) -> Vec<MofaAgent> {
        // Here we should ask each client for its available agents
        // and return the union of all agents
        // TODO(Julian): remove cloning and server_id should be server_address, set from within the client
        if self.available_agents.is_empty() {
            println!("no agents fetched, fetching from servers");
            for server in self.mofa_servers.values() {
                let server_agents = server.client.get_available_agents();
                println!("extending agents list with server {} agents: {}", server.address, server_agents.len());
                // agents.extend(server_agents);
                for mut agent in server_agents {
                    let unique_agent_id = unique_agent_id(&agent.id, &server.address);
                    agent.server_id = moly_mofa::MofaServerId(server.address.clone());
                    agent.id = unique_agent_id.clone();
                    self.available_agents.insert(unique_agent_id, agent);
                }
            }
        }
        // TODO(Julian): remove unnecessary cloning, rework this
        self.available_agents.values().cloned().collect()
    }

    /// Tests the connection to a MoFa server by requesting /v1/models
    /// The connection status is updated at the App level based on the actions dispatched
    pub fn test_mofa_server_connection(&mut self, address: String) {
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

    pub fn edit_chat_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            chat.edit_message(message_id, updated_message);
        }
    }

    pub fn get_loading_file(&self) -> Option<&File> {
        self.chats
            .model_loader
            .get_loading_file_id()
            .map(|file_id| self.downloads.get_file(&file_id))
            .flatten()
    }

    pub fn get_loaded_downloaded_file(&self) -> Option<DownloadedFile> {
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
            Some(ChatEntityId::Agent(agent)) => {
                self.available_agents
                    .get(&agent)
                    .map(|a| a.name.clone())
            }
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
            self.chats.set_current_chat(chat_id);
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

    pub fn remove_mofa_server(&mut self, address: &str) {
        self.mofa_servers.remove(&MofaServerId(address.to_string()));
        self.available_agents.retain(|_, agent| agent.server_id.0 != address);
    }

    pub fn register_mofa_server(&mut self, address: String) -> MofaServerId {
        let server_id = MofaServerId(address.clone());
        let client = Rc::new(MofaClient::new(address.clone()));
        
        self.mofa_servers.insert(server_id.clone(), ServerInfo {
            address: address.clone(),
            client,
            available_agent_ids: Vec::new(),
            connection_status: MofaServerConnectionStatus::Disconnected,
        });

        // TODO(Julian): we might want to these in one step
        self.test_mofa_server_connection(address);
        self.fetch_agents_from_server(&server_id);
        
        server_id
    }

    pub fn fetch_agents_from_server(&mut self, server_id: &MofaServerId) {
        // TODO(Julian): remove cloning and server_id should be server_address, set from within the client
        let server = self.mofa_servers.get(server_id).unwrap();
        let agents = server.client.get_available_agents();
        for mut agent in agents {
            let unique_agent_id = unique_agent_id(&agent.id, &server.address);
            agent.server_id = moly_mofa::MofaServerId(server.address.clone());
            agent.id = unique_agent_id.clone();
            self.available_agents.insert(unique_agent_id, agent);
        }
    }

    pub fn get_client_for_agent(&self, agent_id: &AgentId) -> Option<Rc<MofaClient>> {
        self.available_agents.get(agent_id)
            .and_then(|agent| self.mofa_servers.get(&agent.server_id))
            .map(|server| server.client.clone())
    }
}

fn unique_agent_id(agent_id: &AgentId, server_address: &str) -> AgentId {
    AgentId(format!("{}-{}", agent_id.0, server_address))
}
