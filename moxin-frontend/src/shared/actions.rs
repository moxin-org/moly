use makepad_widgets::DefaultNone;
use moxin_protocol::data::FileID;

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadedFileAction {
    StartChat(FileID),
    ResumeChat(FileID),
    None,
}
