use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sheet {
    #[serde(default)]
    pub code: String,
    pub rounds: Vec<Round>,
}

impl Sheet {
    pub fn current_round_index(&self) -> Option<usize> {
        self.rounds.iter().position(|r| r.weight.is_none())
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Round {
    pub chats: Vec<Chat>,
    pub weight: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub body: String,
    pub sender: Sender,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Sender {
    Agent,
    User,
}
