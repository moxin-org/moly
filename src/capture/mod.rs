//! Implementation of contextual capture for Moly.
//!
//! This module contains the API and platform implementations of Moly's
//! contextual capture functionality, enabling convenient transfer of content
//! from external sources into Moly.
//!
//! Currently, capture is supported with:
//!
//! - (macOS) System service accessible via keyboard shortcuts, context menus,
//!   and similar. Implemented with the macOS/Cocoa pasteboard and service
//!   provider APIs.
//!
//! In the future, the plan is to build upon this functionality with more
//! extensive, contextual interactions across the system.
//!
//! ## Usage
//!
//! The primary entrypoint for the API is [`CaptureHandler`], registered with
//! [`register_handler`], and by which [`Event`]s are consumed.
//!
//! ### Example
//!
//! ```no_run
//! # fn main() {
//! use moly::capture::{self, CaptureHandler};
//!
//! struct MyCaptureHandler;
//!
//! impl CaptureHandler for MyCaptureHandler {
//!     fn capture(&self, event: capture::Event) {
//!         dbg!(event);
//!     }
//!
//!     fn error(&self, error: capture::Error) {
//!         dbg!(error);
//!     }
//! }
//!
//! // Initialize capture with the above handler.
//! moly::capture::register_handler(MyCaptureHandler);
//! # }
//! ```

use std::fmt;
use std::sync::{Arc, Mutex};

mod platform;

/// Handle capture events and errors for an application.
///
/// **See also:** [module docs](self), [`register_handler`]
pub trait CaptureHandler: 'static + Send + Sync {
    fn capture(&self, event: Event);

    fn error(&self, error: Error);
}

/// Initialize platform capture integration with a given [`CaptureHandler`].
///
/// This will register the following to call into the provided handler:
///
/// - (macOS) Capture service provider.
///
/// **See also:** [module docs](self), [`CaptureHandler`]
pub fn register_handler<T>(handler: T) -> Result<(), Error>
where
    T: CaptureHandler,
{
    let handler = Arc::new(Mutex::new(handler));
    platform::register_handler(handler)?;
    Ok(())
}

/// An individual capture event.
///
/// **See also:** [module docs](self)
#[derive(Debug, Clone)]
pub struct Event {
    contents: String,
    source: Source,
}

impl Event {
    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn source(&self) -> &Source {
        &self.source
    }
}

/// The origin of a capture event, i.e., what triggered it.
///
/// **See also:** [module docs](self), [`Event`]
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Source {
    /// System/platform service (e.g. via context menu, keyboard shortcut).
    System,
}

/// Error type related to capture.
///
/// **See also:** [`CaptureHandler`]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "capture error")
    }
}

impl std::error::Error for Error {}
