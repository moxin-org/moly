use makepad_widgets::DefaultNone;
use moly_protocol::data::FileID;

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    Start(FileID),
    None,
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadAction {
    Play(FileID),
    Pause(FileID),
    Cancel(FileID),
    None,
}