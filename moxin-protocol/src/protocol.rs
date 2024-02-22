use crate::data::Model;

#[derive(Clone, Debug)]
pub enum Command {
    GetFeaturedModels,

    // The argument is a string with the keywords to search for.
    SearchModels(String),

    // The argument is the File name.
    DownloadFile(String),
}

#[derive(Clone, Debug)]
pub enum Response {
    // Response to the GetFeaturedModels command
    FeaturedModels(Vec<Model>),

    // Response to the SearchModels command
    ModelsSearchResults(Vec<Model>),
}