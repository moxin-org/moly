use anyhow::Result;
use moly_kit::utils::asynchronous::PlatformSend;
use std::future::Future;
use std::path::Path;

/// An adapter exposes the **bare minimum** functionality needed to interact with
/// a specific filesystem.
///
/// Is the foundation for the `FileSystem` abstraction, which exposes higher-level
/// operations with better ergonomics.
pub trait Adapter: Send + Sync + 'static {
    /// Write some binary content to a given path, creating any necessary directories.
    fn write(
        &mut self,
        path: &Path,
        content: &[u8],
    ) -> impl Future<Output = Result<()>> + PlatformSend;
    /// Read a file from the filesystem, returning its content as a byte vector.
    fn read(&mut self, path: &Path) -> impl Future<Output = Result<Vec<u8>>> + PlatformSend;
    /// Check if a file exists, failing if it cannot be determined.
    fn exists(&mut self, path: &Path) -> impl Future<Output = Result<bool>> + PlatformSend;
    /// Remove a file from the filesystem.
    fn remove(&mut self, path: &Path) -> impl Future<Output = Result<()>> + PlatformSend;
    /// Get a list of the entry names in the given directory.
    fn list(&mut self, path: &Path) -> impl Future<Output = Result<Vec<String>>> + PlatformSend;
}
