use makepad_widgets::{ActionDefaultRef, DefaultNone};
use moly_protocol::data::FileID;

use crate::data::chats::chat_entity::ChatEntityId;

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    Start(ChatEntityId),
    None,
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadAction {
    Play(FileID),
    Pause(FileID),
    Cancel(FileID),
    None,
}
