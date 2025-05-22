use anyhow::Result;

/// A key-value storage adapter that at least knows how to store strings.
pub trait Adapter {
    /// Given a key, get the string value.
    fn get(&mut self, key: &str) -> Result<Option<String>>;

    /// Set the string value of a key.
    fn set(&mut self, key: &str, value: &str) -> Result<()>;

    /// Check if a key exists.
    fn has(&mut self, key: &str) -> Result<bool>;

    /// Delete a key and its value.
    fn remove(&mut self, key: &str) -> Result<()>;

    /// An iterator over stored keys.
    fn keys(&mut self) -> Result<impl Iterator<Item = String>>;
}
