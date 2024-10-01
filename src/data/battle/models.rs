use serde::{Deserialize, Serialize};

/// A sheet contains all the Q&A rounds of a battle. To be filled.
///
/// Called "sheet" like a "spreadsheet".
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sheet {
    #[serde(default)]
    pub code: String,
    pub rounds: Vec<Round>,
}

impl Sheet {
    pub fn current_round_index(&self) -> Option<usize> {
        self.rounds.iter().position(|r| r.vote.is_none())
    }

    pub fn current_round(&self) -> Option<&Round> {
        self.current_round_index().map(|i| &self.rounds[i])
    }

    pub fn current_round_mut(&mut self) -> Option<&mut Round> {
        self.current_round_index().map(move |i| &mut self.rounds[i])
    }

    pub fn is_completed(&self) -> bool {
        self.current_round_index().is_none()
    }
}

/// A round to play/answer. Contains the pair of chats to display, and a
/// weight/vote to be filled by the user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Round {
    pub chats: Vec<Chat>,
    /// The user's vote. `None` if not voted yet.
    /// Values should go from -2 to 2, where 0 is neutral, negative is for the
    /// first chat, and positive for the second.
    pub vote: Option<i8>,
}

/// Minimalistic representation of a chat. Contains a list of messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub messages: Vec<Message>,
}

/// Minimalistic representation of a message. Contains the body and the sender.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub body: String,
    pub sender: Sender,
}

/// Who sent the message.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Sender {
    Agent,
    User,
}
