use crate::utils::asynchronous::{PlatformSendFuture, PlatformSendStream, ThreadToken};

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Private `rfd::FileHandle` wrapper with a runtime generated ID for partial equality.
#[derive(Clone)]
struct WebFileHandle {
    id: u64,
    rfd_handle: rfd::FileHandle,
}

impl From<rfd::FileHandle> for WebFileHandle {
    fn from(handle: rfd::FileHandle) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        WebFileHandle {
            id,
            rfd_handle: handle,
        }
    }
}

impl PartialEq for WebFileHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Private type that points to wherever the attachment content is stored.
///
/// Comparision is done by pointer, file path, file handle, etc. Not by content.
#[derive(Debug, Clone)]
enum AttachmentContentHandle {
    InMemory(Arc<[u8]>),
    NativeFile(std::path::PathBuf),
    WebFile(ThreadToken<WebFileHandle>),
}

impl PartialEq for AttachmentContentHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AttachmentContentHandle::InMemory(a), AttachmentContentHandle::InMemory(b)) => {
                Arc::ptr_eq(a, b)
            }
            (AttachmentContentHandle::NativeFile(a), AttachmentContentHandle::NativeFile(b)) => {
                a == b
            }
            (AttachmentContentHandle::WebFile(a), AttachmentContentHandle::WebFile(b)) => {
                let a_id = a.peek(|handle| handle.id);
                let b_id = b.peek(|handle| handle.id);
                a_id == b_id
            }
            _ => false,
        }
    }
}

impl<T> From<T> for AttachmentContentHandle
where
    T: AsRef<[u8]>,
{
    fn from(bytes: T) -> Self {
        AttachmentContentHandle::InMemory(Arc::from(bytes.as_ref()))
    }
}

impl AttachmentContentHandle {
    async fn read(&self) -> std::io::Result<Arc<[u8]>> {
        match self {
            AttachmentContentHandle::InMemory(content) => Ok(content.clone()),
            AttachmentContentHandle::NativeFile(path) => {
                // TODO: Do not compile tokio on web!
                let content = tokio::fs::read(path).await?;
                Ok(Arc::from(content))
            }
            AttachmentContentHandle::WebFile(handle) => {
                let handle = handle.clone_inner();
                let content = handle.rfd_handle.read().await;
                Ok(Arc::from(content))
            }
        }
    }
}

/// Represents a file/image/document sent or received as part of a message.
///
/// When comparing, two [`Attachment`]s are considered equal if they have the same
/// metadata (name, content type, etc), and they **point** to the same data.
///
/// This means:
/// - For in-memory attachments, the content is compared by reference (pointer equality).
/// - TODO: For attachments picked from a native file system, the path is compared.
/// - TODO: For attachments picked on the web, the (wrapped) file handle must be the same.
/// - TODO: For persisted attachments, the storage key and adapter are compared.
///
/// The content itself is never compared, because not all attachments can be read
/// synchronously, and it would be expensive to do so.
///
/// Unless persistence is configured, when serializing this type, the "pointer" to
/// data is skipped and the attachment will become "unavailable" when deserialized back.
///
/// Two unavailable attachments are considered equal if they have the same metadata.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Attachment {
    /// Normally the original filename.
    pub name: String,
    /// Mime type of the content, if known.
    pub content_type: Option<String>,
    // TODO: Read on demand instead of holding the content in memory.
    #[serde(skip)]
    content: Option<AttachmentContentHandle>,
}

impl Attachment {
    /// Crate private utility to pick files from the file system.
    ///
    /// - On web, async API is required to pick files.
    /// - On macos, sync API is required and must be called from the main UI thread.
    ///   - This is the reason why it takes a closure instead of returning a Future.
    ///     Because on native `spawn` may run in a separate thread. So we can't generalize.
    /// - We follow macos requirements on all native platforms just in case.
    pub(crate) fn pick_multiple(cb: impl FnOnce(Result<Vec<Attachment>, ()>) + 'static) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                crate::utils::asynchronous::spawn(async move {
                    let Some(handles) = rfd::AsyncFileDialog::new()
                        .pick_files()
                        .await
                    else {
                        cb(Err(()));
                        return;
                    };

                    let mut attachments = Vec::with_capacity(handles.len());
                    for handle in handles {
                        // Notice that rfd doesn't return a Result.
                        let content = handle.read().await;
                        let name = handle.file_name();
                        let content_type = mime_guess::from_path(&name)
                            .first()
                            .map(|m| m.to_string());
                        attachments.push(Attachment {
                            name,
                            content_type,
                            content: Some(content.into()),
                        });
                    }

                    cb(Ok(attachments));
                });
            } else if #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))] {
                let Some(paths) = rfd::FileDialog::new()
                    .pick_files()
                else {
                    cb(Err(()));
                    return;
                };

                let mut attachments = Vec::with_capacity(paths.len());
                for path in paths {
                    let content = match std::fs::read(&path) {
                        Ok(content) => content,
                        Err(_) => {
                            cb(Err(()));
                            return;
                        }
                    };
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let content_type = mime_guess::from_path(&name)
                        .first()
                        .map(|m| m.to_string());

                    attachments.push(Attachment {
                        name,
                        content_type,
                        content: Some(content.into()),
                    });
                }
                cb(Ok(attachments));
            } else {
                makepad_widgets::warning!("Attachment picking is not supported on this platform");
                cb(Err(()));
            }
        }
    }

    /// Creates a new in-memory attachment from the given bytes.
    pub fn from_bytes(name: String, content_type: Option<String>, content: &[u8]) -> Self {
        Attachment {
            name,
            content_type,
            content: Some(content.into()),
        }
    }

    /// Creates a new in-memory attachment from a base64 encoded string.
    pub fn from_base64(
        name: String,
        content_type: Option<String>,
        base64_content: &str,
    ) -> std::io::Result<Self> {
        use base64::Engine;
        let content = base64::engine::general_purpose::STANDARD
            .decode(base64_content)
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid base64 content")
            })?;

        Ok(Attachment::from_bytes(name, content_type, &content))
    }

    pub fn is_available(&self) -> bool {
        self.content.is_some()
    }

    pub fn is_image(&self) -> bool {
        if let Some(content_type) = &self.content_type {
            content_type.starts_with("image/")
        } else {
            false
        }
    }

    pub async fn read(&self) -> std::io::Result<Arc<[u8]>> {
        if let Some(content) = &self.content {
            content.read().await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Attachment content not available",
            ))
        }
    }

    pub async fn read_base64(&self) -> std::io::Result<String> {
        use base64::Engine;
        let content = self.read().await?;
        Ok(base64::engine::general_purpose::STANDARD.encode(content))
    }

    /// Crate private utility to save/download the attachment to the file system.
    pub(crate) fn save(&self) {
        makepad_widgets::log!("Downloading attachment: {}", self.name);

        if self.content.is_none() {
            makepad_widgets::warning!("Attachment content not available for saving: {}", self.name);
            return;
        }

        self.save_impl();
    }

    #[cfg(target_arch = "wasm32")]
    fn save_impl(&self) {
        let self_clone = self.clone();
        crate::utils::asynchronous::spawn(async move {
            let Ok(content) = self_clone.content.as_ref().unwrap().read().await else {
                makepad_widgets::warning!(
                    "Failed to read attachment content for saving: {}",
                    self_clone.name
                );
                return;
            };

            use crate::utils::platform::{create_scoped_blob_url, trigger_download};
            create_scoped_blob_url(content, self_clone.content_type.as_deref(), |url| {
                trigger_download(url, &self_clone.name);
            });
        });
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn save_impl(&self) {
        let content_handle = self.content.as_ref().unwrap();

        let content = match futures::executor::block_on(content_handle.read()) {
            Ok(content) => content,
            Err(_) => {
                makepad_widgets::warning!(
                    "Failed to read attachment content for saving: {}",
                    self.name
                );
                return;
            }
        };

        crate::utils::platform::trigger_save_as(&content, Some(self.name.as_str()));
    }

    #[cfg(not(any(
        target_arch = "wasm32",
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    fn save_impl(&self) {
        makepad_widgets::warning!("Attachment saving is not supported on this platform");
    }

    /// Get the content type or "application/octet-stream" if not set.
    pub fn content_type_or_octet_stream(&self) -> &str {
        self.content_type
            .as_deref()
            .unwrap_or("application/octet-stream")
    }
}
