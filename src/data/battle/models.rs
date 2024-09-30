use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sheet {
    #[serde(default)]
    pub code: String,
    pub rounds: Vec<Round>,
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
