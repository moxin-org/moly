use crossbeam::channel::Sender;

use crate::data::{ChatBody, LoadModel, Model, Token, TokenError};

#[derive(Clone, Debug)]
pub enum Command {
    GetFeaturedModels(Sender<Vec<Model>>),

    // The argument is a string with the keywords to search for.
    SearchModels(String, Sender<Vec<Model>>),

    // The argument is the File name.
    DownloadFile(String, Sender<()>),

    LoadModel(LoadModel, Sender<()>),
    Chat(ChatBody, Sender<Result<Token, TokenError>>),
}
