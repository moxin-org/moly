#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(super) use macos::*;

#[cfg(not(target_os = "macos"))]
mod unsupported;
#[cfg(not(target_os = "macos"))]
pub(super) use unsupported::*;
