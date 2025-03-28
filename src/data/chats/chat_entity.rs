//! Hopefully, provisional solution to unify model files and agents in the chat system.

use moly_kit::BotId;
use moly_protocol::data::{File, FileID};
use serde::{Deserialize, Serialize};

use crate::data::providers::{bot_id_as_str, RemoteModel, RemoteModelId};

/// Identifies either a local model file, a MoFa agent, or a remote model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatEntityId {
    ModelFile(FileID),
    Agent(RemoteModelId),
    RemoteModel(RemoteModelId),
}

impl ChatEntityId {
    pub fn as_bot_id(&self) -> BotId {
        match self {
            ChatEntityId::Agent(agent) => BotId::from(agent.0.as_str()),
            ChatEntityId::RemoteModel(model) => BotId::from(model.0.as_str()),
            ChatEntityId::ModelFile(file) => {
                // TODO(Julian): Rework this so that this mapping is not hardcoded or necessary at all
                let id = bot_id_as_str(file.as_str(), "http://localhost:8765/api/v1");
                BotId::from(id.as_str())
            },
        }
    }
}

/// Reference to either a model file, an agent, or a remote model.
///
/// Can be used to chain iterators of both types or simply to take either as a parameter.
#[derive(Debug, Clone, Serialize, Copy)]
pub enum ChatEntityRef<'a> {
    Agent(&'a RemoteModel),
    ModelFile(&'a File),
    RemoteModel(&'a RemoteModel), 
}

impl<'a> ChatEntityRef<'a> {
    pub fn id(&self) -> ChatEntityId {
        match self {
            ChatEntityRef::ModelFile(file) => ChatEntityId::ModelFile(file.id.clone()),
            ChatEntityRef::Agent(agent) => ChatEntityId::Agent(agent.id.clone()),
            ChatEntityRef::RemoteModel(model) => ChatEntityId::RemoteModel(model.id.clone()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ChatEntityRef::ModelFile(file) => &file.name,
            ChatEntityRef::Agent(agent) => &agent.name,
            ChatEntityRef::RemoteModel(model) => &model.name,
        }
    }
}

// impl<'a> From<&'a MofaAgent> for ChatEntityRef<'a> {
//     fn from(agent: &'a MofaAgent) -> Self {
//         ChatEntityRef::Agent(agent)
//     }
// }

impl<'a> From<&'a File> for ChatEntityRef<'a> {
    fn from(file: &'a File) -> Self {
        ChatEntityRef::ModelFile(file)
    }
}

impl<'a> From<&'a RemoteModel> for ChatEntityRef<'a> {
    fn from(model: &'a RemoteModel) -> Self {
        ChatEntityRef::RemoteModel(model)
    }
}
