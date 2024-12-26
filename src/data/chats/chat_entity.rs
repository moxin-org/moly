//! Hopefully, provisional solution to unify model files and agents in the chat system.

use moly_mofa::MofaAgent;
use moly_protocol::data::{File, FileID};
use serde::{Deserialize, Serialize};

/// Identifies either a model file or an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatEntityId {
    ModelFile(FileID),
    /// Since agents are currently fixed enum values, the agent itself is the identifier.
    Agent(MofaAgent),
}

/// Reference to either a model file or an agent.
///
/// Can be used to chain iterators of both types or simply to take either as a parameter.
#[derive(Debug, Clone, Serialize, Copy)]
pub enum ChatEntityRef<'a> {
    Agent(&'a MofaAgent),
    ModelFile(&'a File),
}

impl<'a> ChatEntityRef<'a> {
    pub fn id(&self) -> ChatEntityId {
        match self {
            ChatEntityRef::ModelFile(file) => ChatEntityId::ModelFile(file.id.clone()),
            ChatEntityRef::Agent(agent) => ChatEntityId::Agent(**agent),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ChatEntityRef::ModelFile(file) => &file.name,
            ChatEntityRef::Agent(agent) => agent.name(),
        }
    }
}

impl<'a> From<&'a MofaAgent> for ChatEntityRef<'a> {
    fn from(agent: &'a MofaAgent) -> Self {
        ChatEntityRef::Agent(agent)
    }
}

impl<'a> From<&'a File> for ChatEntityRef<'a> {
    fn from(file: &'a File) -> Self {
        ChatEntityRef::ModelFile(file)
    }
}
