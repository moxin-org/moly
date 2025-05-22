use super::Adapter;
use anyhow::{anyhow, Result};

fn local_storage() -> web_sys::Storage {
    web_sys::window().unwrap().local_storage().unwrap().unwrap()
}

#[derive(Clone, Default)]
pub(super) struct WebAdapter;

impl Adapter for WebAdapter {
    fn get(&mut self, key: &str) -> Result<Option<String>> {
        local_storage()
            .get_item(key)
            .map_err(|e| anyhow!("Failed to get item '{}' from local storage: {:?}", key, e))
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        local_storage()
            .set_item(key, value)
            .map_err(|e| anyhow!("Failed to set item '{}' in local storage: {:?}", key, e))
    }

    fn has(&mut self, key: &str) -> Result<bool> {
        local_storage()
            .get_item(key)
            .map(|opt_val| opt_val.is_some())
            .map_err(|e| anyhow!("Failed to check item '{}' in local storage: {:?}", key, e))
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        local_storage().remove_item(key).map_err(|e| {
            anyhow!(
                "Failed to remove item '{}' from local storage: {:?}",
                key,
                e
            )
        })
    }

    fn keys(&mut self) -> Result<Vec<String>> {
        let storage = local_storage();
        let length = storage
            .length()
            .map_err(|e| anyhow!("Failed to get local storage length: {:?}", e))?;

        let mut keys_vec = Vec::with_capacity(length as usize);
        for i in 0..length {
            match storage.key(i) {
                Ok(Some(key_name)) => keys_vec.push(key_name),
                Ok(None) => {
                    makepad_widgets::warning!(
                        "Rare case of null local storage key at valid index {}. Skipping.",
                        i
                    );
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Failed to get key at index {} from local storage: {:?}",
                        i,
                        e
                    ))
                }
            }
        }
        Ok(keys_vec)
    }
}
