#[cfg(target_os = "macos")]
mod apple;
#[cfg(target_os = "macos")]
pub(super) use apple::*;

#[cfg(not(target_os = "macos"))]
mod unsupported;
#[cfg(not(target_os = "macos"))]
pub(super) use unsupported::*;
