//! This module contains the `Attachment` abstraction to exchange files back and forth
//! between users and AI.
//!
//! See [`Attachment`] for more details.

use crate::utils::asynchronous::BoxPlatformSendFuture;
#[cfg(target_arch = "wasm32")]
use crate::utils::asynchronous::ThreadToken;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU64, Ordering};

use std::sync::Arc;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Private `rfd::FileHandle` wrapper with a runtime generated ID for partial equality.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct WebFileHandle {
    id: u64,
    rfd_handle: rfd::FileHandle,
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
impl PartialEq for WebFileHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone)]
struct PersistedAttachmentHandle {
    reader: Arc<
        dyn Fn(&str) -> BoxPlatformSendFuture<'static, std::io::Result<Arc<[u8]>>> + Send + Sync,
    >,
    key: String,
}

impl std::fmt::Debug for PersistedAttachmentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistedAttachmentHandle")
            .field("key", &self.key)
            .field("reader", &format_args!("{:p}", Arc::as_ptr(&self.reader)))
            .finish()
    }
}

/// Private type that points to wherever the attachment content is stored.
///
/// Comparision is done by pointer, file path, file handle, etc. Not by content.
#[derive(Debug, Clone)]
enum AttachmentContentHandle {
    InMemory(Arc<[u8]>),
    #[cfg(not(target_arch = "wasm32"))]
    FilePick(std::path::PathBuf),
    #[cfg(target_arch = "wasm32")]
    FilePick(ThreadToken<WebFileHandle>),
    ErasedPersisted(String),
    Persisted(PersistedAttachmentHandle),
}

#[cfg(feature = "json")]
impl serde::Serialize for AttachmentContentHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AttachmentContentHandle::ErasedPersisted(key) => serializer.serialize_str(key),
            AttachmentContentHandle::Persisted(persisted) => {
                serializer.serialize_str(&persisted.key)
            }
            _ => serializer.serialize_none(),
        }
    }
}

#[cfg(feature = "json")]
impl<'de> serde::Deserialize<'de> for AttachmentContentHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt_key: Option<String> = Option::deserialize(deserializer)?;
        match opt_key {
            Some(key) => Ok(AttachmentContentHandle::ErasedPersisted(key)),
            None => Err(serde::de::Error::custom(
                "AttachmentContentHandle cannot be deserialized from null",
            )),
        }
    }
}

impl PartialEq for AttachmentContentHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AttachmentContentHandle::InMemory(a), AttachmentContentHandle::InMemory(b)) => {
                Arc::ptr_eq(a, b)
            }
            #[cfg(not(target_arch = "wasm32"))]
            (AttachmentContentHandle::FilePick(a), AttachmentContentHandle::FilePick(b)) => a == b,
            #[cfg(target_arch = "wasm32")]
            (AttachmentContentHandle::FilePick(a), AttachmentContentHandle::FilePick(b)) => {
                let a_id = a.peek(|handle| handle.id);
                let b_id = b.peek(|handle| handle.id);
                a_id == b_id
            }
            (
                AttachmentContentHandle::ErasedPersisted(a),
                AttachmentContentHandle::ErasedPersisted(b),
            ) => a == b,
            (AttachmentContentHandle::Persisted(a), AttachmentContentHandle::Persisted(b)) => {
                a.key == b.key
            }

            _ => false,
        }
    }
}

impl Eq for AttachmentContentHandle {}

impl std::hash::Hash for AttachmentContentHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            AttachmentContentHandle::InMemory(content) => {
                Arc::as_ptr(content).hash(state);
            }
            #[cfg(not(target_arch = "wasm32"))]
            AttachmentContentHandle::FilePick(path) => path.hash(state),
            #[cfg(target_arch = "wasm32")]
            AttachmentContentHandle::FilePick(handle) => handle.peek(|h| h.id).hash(state),
            AttachmentContentHandle::ErasedPersisted(key) => key.hash(state),
            AttachmentContentHandle::Persisted(persisted) => persisted.key.hash(state),
        }
    }
}

impl From<&[u8]> for AttachmentContentHandle {
    fn from(bytes: &[u8]) -> Self {
        AttachmentContentHandle::InMemory(Arc::from(bytes))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::path::PathBuf> for AttachmentContentHandle {
    fn from(path: std::path::PathBuf) -> Self {
        AttachmentContentHandle::FilePick(path)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<rfd::FileHandle> for AttachmentContentHandle {
    fn from(handle: rfd::FileHandle) -> Self {
        AttachmentContentHandle::FilePick(ThreadToken::new(WebFileHandle::from(handle)))
    }
}

impl AttachmentContentHandle {
    async fn read(&self) -> std::io::Result<Arc<[u8]>> {
        match self {
            AttachmentContentHandle::InMemory(content) => Ok(content.clone()),
            #[cfg(not(target_arch = "wasm32"))]
            AttachmentContentHandle::FilePick(path) => {
                let content = tokio::fs::read(path).await?;
                Ok(Arc::from(content))
            }
            #[cfg(target_arch = "wasm32")]
            AttachmentContentHandle::FilePick(handle) => {
                let handle = handle.clone_inner();
                let content = handle.rfd_handle.read().await;
                Ok(Arc::from(content))
            }
            AttachmentContentHandle::ErasedPersisted(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot read erased persisted attachment. Please restore the reader with `set_persisted_reader` first.",
            )),
            AttachmentContentHandle::Persisted(persisted) => {
                (persisted.reader)(persisted.key.as_str()).await
            }
        }
    }
}

/// Represents a file/image/document sent or received as part of a message.
///
/// ## Examples
///
/// - An attachment sent by the user.
/// - An image generated by the AI.
///
/// ## Equality
///
/// When comparing, two [`Attachment`]s are considered equal if they have the same
/// metadata (name, content type, etc), and they **point** to the same data.
///
/// This means:
/// - For in-memory attachments, the content is compared by reference (pointer equality).
/// - For attachments picked from a native file system, the path is compared.
/// - For attachments picked on the web, the (wrapped) file handle must be the same.
/// - For persisted attachments, the storage key is compared (independent of the reader).
///
/// The content itself is never compared, because not all attachments can be read
/// synchronously, and it would be expensive to do so.
///
/// ## Serialization
///
/// Unless a persistence key is configured, when serializing this type, the "pointer" to
/// data is skipped and the attachment will become "unavailable" when deserialized back.
///
/// Two unavailable attachments are considered equal if they have the same metadata.
///
/// If a persistence key is configured, the attachment will be serialized with the key.
/// That key will be restored when deserializing, however, you will need to manually
/// restore its reader implementation.
///
/// ## Abstraction details
///
/// Different than other abstraction in [`crate::protocol`], this one not only
/// acts as "data model", but also as system and I/O interface. This coupling was
/// originally intended to give "pragmatic" access to methods like `read()` and `pick_multiple()`,
/// but as everything mixing concerns, this now causes some issues like making the persistence
/// feature uglier to integrate. So this abstraction is likely to change in the future.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Attachment {
    /// Normally the original filename.
    pub name: String,
    /// Mime type of the content, if known.
    pub content_type: Option<String>,
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
                        let name = handle.file_name();
                        let content_type = mime_guess::from_path(&name)
                            .first()
                            .map(|m| m.to_string());
                        attachments.push(Attachment {
                            name,
                            content_type,
                            content: Some(handle.into()),
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
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let content_type = mime_guess::from_path(&name)
                        .first()
                        .map(|m| m.to_string());

                    attachments.push(Attachment {
                        name,
                        content_type,
                        content: Some(path.into()),
                    });
                }
                cb(Ok(attachments));
            } else {
                ::log::warn!("Attachment picking is not supported on this platform");
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
        ::log::info!("Downloading attachment: {}", self.name);

        if self.content.is_none() {
            ::log::warn!("Attachment content not available for saving: {}", self.name);
            return;
        }

        self.save_impl();
    }

    #[cfg(target_arch = "wasm32")]
    fn save_impl(&self) {
        let self_clone = self.clone();
        crate::utils::asynchronous::spawn(async move {
            let Ok(content) = self_clone.content.as_ref().unwrap().read().await else {
                ::log::warn!(
                    "Failed to read attachment content for saving: {}",
                    self_clone.name
                );
                return;
            };

            use crate::utils::platform::{create_scoped_blob_url, trigger_download};
            create_scoped_blob_url(&content, self_clone.content_type.as_deref(), |url| {
                trigger_download(url, &self_clone.name);
            });
        });
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn save_impl(&self) {
        let content_handle = self.content.as_ref().unwrap();

        // Although we could read this content asynchronously, we would still need
        // to open the save dialog synchronously from the main thread, which would
        // complicate things.
        let content = match futures::executor::block_on(content_handle.read()) {
            Ok(content) => content,
            Err(err) => {
                ::log::warn!(
                    "Failed to read attachment content for saving {}: {}",
                    self.name,
                    err
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
        ::log::warn!("Attachment saving is not supported on this platform");
    }

    /// Get the content type or "application/octet-stream" if not set.
    pub fn content_type_or_octet_stream(&self) -> &str {
        self.content_type
            .as_deref()
            .unwrap_or("application/octet-stream")
    }

    /// Get the persistence key if set.
    pub fn get_persistence_key(&self) -> Option<&str> {
        match &self.content {
            Some(AttachmentContentHandle::Persisted(persisted)) => Some(&persisted.key),
            Some(AttachmentContentHandle::ErasedPersisted(key)) => Some(key),
            _ => None,
        }
    }

    /// Check if the attachment has a persistence key set, indicating it was persisted.
    pub fn has_persistence_key(&self) -> bool {
        self.get_persistence_key().is_some()
    }

    /// Check if this attachment has a reader implementation set.
    pub fn has_persistence_reader(&self) -> bool {
        matches!(&self.content, Some(AttachmentContentHandle::Persisted(_)))
    }

    /// Give this attachment a custom persistence key.
    ///
    /// This means you persisted this attachment somewhere and you will take care
    /// of how it's read.
    ///
    /// You should set this key only after you really persisted (wrote) the attachment.
    ///
    /// If you call this, you should also call [`Self::set_persistence_reader`]
    /// to configure how this attachment will be read using this key.
    pub fn set_persistence_key(&mut self, key: String) {
        match &self.content {
            Some(AttachmentContentHandle::Persisted(persisted)) => {
                self.content = Some(AttachmentContentHandle::Persisted(
                    PersistedAttachmentHandle {
                        reader: persisted.reader.clone(),
                        key,
                    },
                ));
            }
            Some(AttachmentContentHandle::ErasedPersisted(_)) => {
                self.content = Some(AttachmentContentHandle::ErasedPersisted(key));
            }
            _ => {
                self.content = Some(AttachmentContentHandle::ErasedPersisted(key));
            }
        }
    }

    /// Gives this attachment a custom implementation to read the persisted content.
    ///
    /// Can only be used after setting a persistence key with [`Self::set_persistence_key`].
    pub fn set_persistence_reader(
        &mut self,
        reader: impl Fn(&str) -> BoxPlatformSendFuture<'static, std::io::Result<Arc<[u8]>>>
        + Send
        + Sync
        + 'static,
    ) {
        match &self.content {
            Some(AttachmentContentHandle::Persisted(persisted)) => {
                self.content = Some(AttachmentContentHandle::Persisted(
                    PersistedAttachmentHandle {
                        reader: Arc::new(reader),
                        key: persisted.key.clone(),
                    },
                ));
            }
            Some(AttachmentContentHandle::ErasedPersisted(key)) => {
                self.content = Some(AttachmentContentHandle::Persisted(
                    PersistedAttachmentHandle {
                        reader: Arc::new(reader),
                        key: key.clone(),
                    },
                ));
            }
            _ => {
                ::log::warn!("Cannot set persistence reader on a non-persisted attachment");
            }
        }
    }
}
