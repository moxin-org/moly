use makepad_widgets::{ActionDefaultRef, DefaultNone};
use moly_kit::BotId;
use moly_protocol::data::FileID;

use crate::data::chats::chat::ChatID;

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    // Start a new chat, no entity specified
    StartWithoutEntity,
    // Start a new chat with a given entity
    Start(BotId),
    // Select a chat from the chat history
    ChatSelected(ChatID),
    None,
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadAction {
    Play(FileID),
    Pause(FileID),
    Cancel(FileID),
    None,
}
