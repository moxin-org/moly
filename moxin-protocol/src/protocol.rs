use crate::data::Model;

#[derive(Clone, Debug)]
pub enum Command {
    GetFeaturedModels,
    SearchModels(String),
}

#[derive(Clone, Debug)]
pub enum Response {
    FeaturedModels(Vec<Model>)
}