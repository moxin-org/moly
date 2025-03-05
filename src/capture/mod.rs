//! Implementation of platform *contextual capture*.
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
//! [`register_handler`], and by which [`CaptureEvent`]s are consumed.
//!
//! ### Example
//!
//! ```no_run
//! # fn main() {
//! use moly::capture::CaptureHandler;
//!
//! struct MyCaptureHandler;
//!
//! impl CaptureHandler for MyCaptureHandler {
//!     fn capture(&self, event: CaptureEvent) {
//!         dbg!(event);
//!     }
//!
//!     fn error(&self, error: CaptureError) {
//!         dbg!(error);
//!     }
//! }
//!
//! // Initialize capture with the above handler.
//! moly::capture::register_handler(MyCaptureHandler);
//! # }
//! ```
//!
//! ## Notes
//!
//! Typically When a capture is initiated, the application is opened and/or
//! focused as appropriate, although this behavior is not strictly guaranteed.

use std::fmt;

mod platform;

/// Handle capture events and errors for an application.
///
/// See also: [module docs](self), [`register_handler`]
pub trait CaptureHandler: 'static + Send + Sync {
    fn capture(&self, event: CaptureEvent);

    fn error(&self, error: CaptureError);
}

/// Initialize platform capture integration with a given [`CaptureHandler`].
///
/// This will register the following to call into the provided handler:
///
/// - (macOS) Capture service provider.
///
/// See also: [module docs](self), [`CaptureHandler`].
pub fn register_handler<T>(handler: T) -> Result<(), InitError>
where
    T: CaptureHandler,
{
    platform::register_handler(handler)
}

/// An individual capture event.
///
/// See also: [module docs](self)
#[derive(Debug, Clone)]
pub struct CaptureEvent {
    contents: String,
    source: CaptureSource,
}

impl CaptureEvent {
    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn source(&self) -> &CaptureSource {
        &self.source
    }
}

/// The origin of a [`CaptureEvent`], i.e., what triggered it.
///
/// See also: [module docs](self)
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum CaptureSource {
    #[default]
    Unknown,
    /// System/platform service (e.g. context menu, keyboard shortcut).
    System,
}

/// An error during initialization.
///
/// See also: [`register_handler`]
#[derive(Debug)]
pub enum InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "capture initialization error")
    }
}

impl std::error::Error for InitError {}

/// An error during capture.
///
/// See also: [`CaptureHandler`]
#[derive(Debug)]
pub enum CaptureError {}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "capture error")
    }
}

impl std::error::Error for CaptureError {}
