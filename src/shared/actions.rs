use makepad_widgets::{ActionDefaultRef, DefaultNone};
use moly_protocol::data::FileID;

use crate::data::chats::{chat::ChatID, chat_entity::ChatEntityId};

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    // Start a new chat, no entity specified
    StartWithoutEntity,
    // Start a new chat with a given entity
    Start(ChatEntityId),
    // Select a chat from the chat history
    ChatSelected(ChatID),
    // Update the title of a chat
    TitleUpdated(ChatID),
    None,
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadAction {
    Play(FileID),
    Pause(FileID),
    Cancel(FileID),
    None,
}
