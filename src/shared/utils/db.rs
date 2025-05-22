use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

mod adapter;
use adapter::Adapter;

/// "Mimic" a simple synchronous document database over a simple key-value storage
/// adapter.
///
/// `/` is disallowed as collection names and document keys as its reserved.
///
/// Can store anything that can be serialized with `serde_json`.
///
/// The database must be explicitly locked during use to prevent simultaneous access
/// on multi-threaded environments.
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

    /// Fetch the value of a record.
    pub fn value<V: RecordValue>(&mut self, collection: &str, key: &str) -> Result<Option<V>> {
        validate_identifier(collection)?;
        validate_identifier(key)?;
        let value = self.adapter_lock.get(&value_key(collection, key))?;
        if let Some(value) = value {
            let value: V = serde_json::from_str(&value)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Update an existing (or create a new) record with the given value.
    pub fn upsert<V: RecordValue>(&mut self, collection: &str, key: &str, value: V) -> Result<()> {
        validate_identifier(collection)?;
        validate_identifier(key)?;
        let value = serde_json::to_string(&value)?;
        self.adapter_lock.set(&value_key(collection, key), &value)?;
        Ok(())
    }

    /// Delete a record.
    pub fn remove(&mut self, collection: &str, key: &str) -> Result<()> {
        validate_identifier(collection)?;
        validate_identifier(key)?;
        self.adapter_lock.remove(&value_key(collection, key))?;
        Ok(())
    }

    /// Get all the record keys of a collection.
    pub fn keys(&mut self, collection: &str) -> Result<Vec<String>> {
        validate_identifier(collection)?;
        let results = self
            .adapter_lock
            .keys()?
            .into_iter()
            .filter_map(|key| {
                let (c, k) = deconstruct_key(&key);
                if c == collection {
                    Some(k.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(results)
    }
}

/// The underlying key in the storage adapter for the value of a record.
fn value_key(collection: &str, key: &str) -> String {
    format!("{}/{}", collection, key)
}

/// Deconstruct a key into its possible components.
fn deconstruct_key(key: &str) -> (&str, &str) {
    key.split_once('/').expect("malformed storage key")
}

fn validate_identifier(key: &str) -> Result<()> {
    if key.contains('/') {
        Err(anyhow!("identifier cannot contain '/'"))
    } else {
        Ok(())
    }
}

/// Record values require serialization and deserialization capabilities.
pub trait RecordValue: Serialize + DeserializeOwned {}

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod native;

/// Get access to the global database instance.
pub fn global() -> Db<impl Adapter> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use web::WebAdapter;
            static DB: LazyLock<Db<WebAdapter>> = LazyLock::new(|| Db::new(WebAdapter::default()));
        } else {
            use native::NativeAdapter;
            static DB: LazyLock<Db<NativeAdapter>> = LazyLock::new(|| Db::new(NativeAdapter::default()));
        }
    }

    DB.clone()
}
