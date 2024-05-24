use makepad_widgets::DefaultNone;
use moxin_protocol::data::FileID;

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    Start(FileID),
    Resume(FileID),
    None,
}
