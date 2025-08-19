//! Utilities to deal with Moly Kit attachments and related persistance.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use moly_kit::protocol::*;

pub fn generate_persisted_key(attachment: &Attachment) -> String {
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

pub fn set_persistence_key_and_reader(attachment: &mut Attachment, key: String) {
    attachment.set_persistence_key(key);
    attachment.set_persistence_reader(persistence_reader());
}

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

pub async fn write_attachment_to_key(attachment: &Attachment, key: &str) -> std::io::Result<()> {
    let content = attachment.read().await?;
    let mut fs = super::filesystem::global();
    let path = PathBuf::from(key);
    fs.queue_write(path, content.to_vec())
        .await
        .map_err(|e| std::io::Error::other(e))?;
    Ok(())
}
