//! Utilities to deal with Moly Kit attachments and related persistance.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use moly_kit::protocol::*;

pub fn generate_persistence_key(attachment: &Attachment) -> String {
    // If `filename.a.b` this is `.b`.
    // If `filename` this is empty.
    let suffix = attachment
        .name
        .rsplit_once('.')
        .map(|(_, suffix)| suffix)
        .map(|s| format!(".{}", s))
        .unwrap_or_else(|| String::from(""));

    let uuid = super::unique::generate_uuid_v7_string();
    // Key includes the full path relative to the data directory in case it's moved.
    format!("attachments/{}{}", uuid, suffix)
}

/// Get the reader to inject into Moly Kit attachments upon setting a persistence key.
///
/// Also re-injected on every attachment on app start.
pub fn persistence_reader()
-> impl Fn(&str) -> BoxPlatformSendFuture<'static, std::io::Result<Arc<[u8]>>> {
    |key| {
        let path = std::path::PathBuf::from(key);
        Box::pin(async move {
            let fs = super::filesystem::global();
            // TODO: Do not use "other" error kind.
            let content = fs.read(&path).await.map_err(|e| std::io::Error::other(e))?;
            Ok(content.into())
        })
    }
}

/// Convinience to set the persistence key together with the Moly's reader.
pub fn set_persistence_key_and_reader(attachment: &mut Attachment, key: String) {
    attachment.set_persistence_key(key);
    attachment.set_persistence_reader(persistence_reader());
}

/// Deletes the persisted file of the given attachment.
pub async fn delete_attachment(attachment: &Attachment) -> std::io::Result<()> {
    let key = attachment
        .get_persistence_key()
        .expect("tried to delete non-persisted attachment");

    let path = Path::new(&key);

    let fs = super::filesystem::global();
    fs.remove(path)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    Ok(())
}

/// Write the content of the given attachment to the given key (actual filesystem location).
pub async fn write_attachment_to_key(attachment: &Attachment, key: &str) -> std::io::Result<()> {
    let content = attachment.read().await?;
    let mut fs = super::filesystem::global();
    let path = PathBuf::from(key);
    fs.queue_write(path, content.to_vec())
        .await
        .map_err(|e| std::io::Error::other(e))?;
    Ok(())
}
