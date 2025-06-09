//! Mobile filesystem adapter for Makepad applications (iOS and Android).
//!
//! This adapter uses the data directory provided by Makepad's `cx.get_data_dir()` to store
//! application files on mobile devices. It organizes files into a logical directory structure
//! within the app's data directory.
//!
//! ## Directory Structure
//!
//! ```text
//! {data_dir}/                    # From cx.get_data_dir()
//! ├── preferences/               # User preferences and configuration
//! │   └── preferences.json
//! ├── chats/                     # Chat history and data
//! └── model_downloads/           # Downloaded AI models
//! ```
//!
//! ## Initialization
//!
//! The adapter must be initialized with the data directory path before use:
//!
//! ```rust
//! // During app startup when Cx is available
//! if let Some(data_dir) = cx.get_data_dir() {
//!     filesystem::init_cx_data_dir(PathBuf::from(data_dir));
//! }
//! ```

use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use super::super::adapter::Adapter;
use anyhow::Result;
use futures::StreamExt;

/// Global storage for mobile data directory path
static MOBILE_DATA_DIR: LazyLock<Mutex<Option<PathBuf>>> = LazyLock::new(|| Mutex::new(None));

/// Set the mobile data directory path from Makepad's Cx.get_data_dir()
///
/// This should be called early in the app lifecycle when Cx is available.
/// The path will be used as the base directory for all file operations.
pub fn set_mobile_data_dir(path: PathBuf) {
    log::info!("Mobile data directory set to: {}", path.display());
    let mut data_dir = MOBILE_DATA_DIR.lock().unwrap();
    *data_dir = Some(path);
}

/// Get the mobile data directory, panicking if not set
///
/// # Panics
///
/// Panics if the mobile data directory has not been initialized via `set_mobile_data_dir()`.
fn get_mobile_data_dir() -> PathBuf {
    MOBILE_DATA_DIR.lock().unwrap().clone().expect(
        "Mobile data directory not set. Call filesystem::init_cx_data_dir() during app startup.",
    )
}

/// Resolve a relative path to an absolute path within the mobile data directory.
///
/// - Paths starting with "preferences/" are placed in a preferences subdirectory
/// - All other paths are placed directly in the data directory
fn validate_and_resolve(path: &Path) -> PathBuf {
    let base_dir = get_mobile_data_dir();

    match path.strip_prefix("preferences/") {
        Ok(rest) => {
            // For preferences, create a preferences subdirectory in the data dir
            base_dir.join("preferences").join(rest)
        }
        Err(_) => {
            // For other files (like chats), put them directly in the data dir
            base_dir.join(path)
        }
    }
}

/// An [Adapter] for `FileSystem` that interacts with mobile device filesystems (iOS and Android)
/// using the data directory provided by Makepad's Cx.get_data_dir().
///
/// This adapter automatically organizes files into appropriate subdirectories
/// within the mobile app's data directory for better organization and compatibility
/// with mobile platform file system conventions.
#[derive(Default)]
pub struct MobileAdapter;

impl Adapter for MobileAdapter {
    async fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let path = validate_and_resolve(path);
        let content = async_fs::read(path).await?;
        Ok(content)
    }

    async fn exists(&mut self, path: &Path) -> Result<bool> {
        let path = validate_and_resolve(path);
        let exists = async_fs::metadata(path).await.map(|_| true).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e)
            }
        })?;
        Ok(exists)
    }

    async fn remove(&mut self, path: &Path) -> Result<()> {
        let path = validate_and_resolve(path);
        async_fs::remove_file(path).await?;
        Ok(())
    }

    async fn list(&mut self, path: &Path) -> Result<Vec<String>> {
        let path = validate_and_resolve(path);
        let mut entries = async_fs::read_dir(path).await?;
        let mut result = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                result.push(name.to_string());
            }
        }
        Ok(result)
    }

    async fn write(&mut self, path: &Path, content: &[u8]) -> Result<()> {
        let path = validate_and_resolve(path);
        async_fs::create_dir_all(path.parent().unwrap()).await?;
        async_fs::write(path, content).await?;
        Ok(())
    }
}
