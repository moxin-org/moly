pub mod app;
pub mod capture;
#[cfg(not(target_arch = "wasm32"))]
pub mod runtime;

mod chat;
mod data;
mod landing;
mod mcp;
mod my_models;
mod settings;
mod shared;
