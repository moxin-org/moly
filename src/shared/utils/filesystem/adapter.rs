//! Filesystem adapter trait and platform-specific utilities.
//!
//! This module defines the core `Adapter` trait that provides a unified interface
//! for filesystem operations across different platforms (native, web, Android).

use anyhow::Result;
use std::future::Future;
use std::path::Path;

// TODO: Consider using single-threaded futures across all platforms, even in
// Moly Kit `spawn` function to avoid separating code by `Send`.
cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub trait PlatformSpecifics {}
        impl<F, O> PlatformSpecifics for F
        where
            F: Future<Output = O>,
        {
        }
    } else {
        pub trait PlatformSpecifics: Send {}
        impl<F, O> PlatformSpecifics for F
        where
            F: Future<Output = O> + Send,
            O: Send,
        {
        }
    }
}

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
    ) -> impl Future<Output = Result<()>> + PlatformSpecifics;
    
    /// Read a file from the filesystem, returning its content as a byte vector.
    fn read(&mut self, path: &Path) -> impl Future<Output = Result<Vec<u8>>> + PlatformSpecifics;
    
    /// Check if a file exists, failing if it cannot be determined.
    fn exists(&mut self, path: &Path) -> impl Future<Output = Result<bool>> + PlatformSpecifics;
    
    /// Remove a file from the filesystem.
    fn remove(&mut self, path: &Path) -> impl Future<Output = Result<()>> + PlatformSpecifics;
    
    /// Get a list of the entry names in the given directory.
    fn list(
        &mut self,
        path: &Path,
    ) -> impl Future<Output = Result<Vec<String>>> + PlatformSpecifics;
}
