#[cfg(target_os = "android")]
pub mod mobile;
#[cfg(target_os = "android")]
pub use mobile::*;

#[cfg(target_os = "ios")]
pub mod mobile;
#[cfg(target_os = "ios")]
pub use mobile::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod web;
