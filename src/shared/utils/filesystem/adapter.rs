//! Filesystem adapter trait and platform-specific utilities.
//!
//! This module defines the core `Adapter` trait that provides a unified interface
//! for filesystem operations across different platforms (native, web, Android).

use anyhow::Result;
use moly_kit::utils::asynchronous::PlatformSendFuture;
use std::path::Path;

/// An adapter exposes the **bare minimum** functionality needed to interact with
/// a specific filesystem.
///
/// This trait is the foundation for the `FileSystem` abstraction, which exposes
/// higher-level operations with better ergonomics. Different platforms implement
/// this trait to provide filesystem access:
///
/// - **Native**: Uses standard filesystem operations via `async_fs`
/// - **Web**: Uses browser storage APIs via `web_fs`
/// - **Android**: Uses Makepad's data directory with organized subdirectories
///
/// All operations are async to support restrictive environments like the web.
pub trait Adapter: Send + Sync + 'static {
    /// Write some binary content to a given path, creating any necessary directories.
    fn write(
        &mut self,
        path: &Path,
        content: &[u8],
    ) -> impl PlatformSendFuture<Output = Result<()>>;

    /// Read a file from the filesystem, returning its content as a byte vector.
    fn read(&mut self, path: &Path) -> impl PlatformSendFuture<Output = Result<Vec<u8>>>;

    /// Check if a file exists, failing if it cannot be determined.
    fn exists(&mut self, path: &Path) -> impl PlatformSendFuture<Output = Result<bool>>;

    /// Remove a file from the filesystem.
    fn remove(&mut self, path: &Path) -> impl PlatformSendFuture<Output = Result<()>>;

    /// Get a list of the entry names in the given directory.
    fn list(&mut self, path: &Path) -> impl PlatformSendFuture<Output = Result<Vec<String>>>;
}
