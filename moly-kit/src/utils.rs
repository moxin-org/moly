//! Internally used to hold utility modules but exposes some very helpful ones.

pub mod asynchronous;
pub(crate) mod events;
pub(crate) mod portal_list;
pub(crate) mod scraping;
pub(crate) mod sse;
pub(crate) mod ui_runner;

#[cfg(feature = "json")]
pub(crate) mod serde;
