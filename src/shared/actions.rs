use makepad_widgets::{DefaultNone, ActionDefaultRef};
use moly_mofa::MofaAgent;
use moly_protocol::data::FileID;

#[derive(Clone, Debug)]
pub enum ChatHandler {
    Model(FileID),
    Agent(MofaAgent),
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    Start(ChatHandler),
    None,
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadAction {
    Play(FileID),
    Pause(FileID),
    Cancel(FileID),
    None,
}
