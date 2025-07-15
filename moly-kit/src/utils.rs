//! Internally used to hold utility modules but exposes some very helpful ones.

pub mod asynchronous;
pub(crate) mod errors;
pub(crate) mod makepad;
pub(crate) mod platform;
pub(crate) mod scraping;
pub(crate) mod string;

#[cfg(feature = "json")]
pub(crate) mod serde;
pub(crate) mod sse;
pub(crate) mod ui_runner;
