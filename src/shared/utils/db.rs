use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

mod adapter;
mod native;
mod web;

use adapter::Adapter;
use native::NativeAdapter;
use web::WebAdapter;

/// "Mimic" a simple synchronous document database by using the filesystem on native
/// and storage APIs on web.
///
/// On native, collections are just directories, and documents are files with JSON
/// content.
///
/// On web, key-value storage is used, where colection names and document keys are
/// joined with `/` to form the final key and JSON is used as the string value.
///
/// `/` is disallowed as collection names and document keys.
///
/// Can store anything that can be serialized with `serde_json`.
///
/// The database must be explicitly locked during use to prevent concurrent access
/// to the file system on multi-threaded native environments.
#[derive(Clone)]
struct Db<A: Adapter> {
    adapter: Arc<Mutex<A>>,
}

impl<A: Adapter> Db<A> {
    fn new(adapter: A) -> Self {
        Self {
            adapter: Arc::new(Mutex::new(adapter)),
        }
    }

    /// Get exclusive access to the database.
    pub fn lock(&self) -> DbLock<A> {
        DbLock::new(self.adapter.lock().expect("db lock is poisoned"))
    }
}

/// Main interface to interact with the database. With exclusive access.
pub struct DbLock<'a, A: Adapter> {
    adapter_lock: MutexGuard<'a, A>,
}

impl<'a, A: Adapter> DbLock<'a, A> {
    fn new(adapter_lock: MutexGuard<'a, A>) -> Self {
        Self { adapter_lock }
    }

    /// Get the metadata of a record.
    pub fn metadata(&mut self, collection: &str, key: &str) -> Result<RecordMetadata> {
        unimplemented!()
    }

    /// Fetch the value of a record.
    pub fn value<V: RecordValue>(
        &mut self,
        collection: &str,
        key: &str,
    ) -> Result<Option<V>> {
        let value = self.adapter_lock.get(collection, key)?;
        if let Some(value) = value {
            let record: V = serde_json::from_str(&value)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Store a record in the database.
    pub fn upsert<V: RecordValue>(
        &mut self,
        collection: &str,
        key: &str,
        value: V,
    ) -> Result<()> {
        let existing 
    }
}

/// The underlying key in the storage adapter for the value of a record.
fn value_key(collection: &str, key: &str) -> String {
    format!("{}/{}", collection, key)
}

/// The underlying key in the storage adapter for the metadata of a record.
fn metadata_key(collection: &str, key: &str) -> String {
    format!("{}/{}/metadata", collection, key)
}

/// Record values require serialization and deserialization capabilities.
pub trait RecordValue: Serialize + DeserializeOwned {}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Metadata related to the record.
pub struct RecordMetadata {
    /// Timestamp set only when the record is created without previous existence.
    pub created_at: DateTime<Utc>,
    /// Timestamp updated everytime the record is written to. Also set on creation.
    pub updated_at: DateTime<Utc>,
}

/// A full record stored in the database, with timestamps and other metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound(deserialize = "V: DeserializeOwned"))]
pub struct Record<V: RecordValue> {
    /// The key that was used during fetch time to get this record.
    pub key: String,
    /// The data stored in this record.
    pub value: V,
    /// Metadata related to the record.
    pub metadata: RecordMetadata,
}

/// Get access to the global database instance.
pub fn global() -> Db<impl Adapter> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            static DB: LazyLock<Db<WebAdapter>> = LazyLock::new(|| Db::new(WebAdapter::default()));
        } else {
            static DB: LazyLock<Db<NativeAdapter>> = LazyLock::new(|| Db::new(NativeAdapter::default()));
        }
    }

    DB.clone()
}
