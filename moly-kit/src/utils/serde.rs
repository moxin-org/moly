//! Utilities to deal with [serde].

use serde::{Deserialize, Deserializer};

/// The value gets deserialized to its default if set to `null` explicitly.
///
/// You still need `#[serde(default)]` in case the field is missing.
///
/// See https://github.com/serde-rs/serde/issues/1098 for more information.
pub(crate) fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Deserializes to the default value on any deserialization error.
///
/// This function will catch all deserialization errors and return the default
/// value for the type instead.
pub(crate) fn deserialize_default_on_error<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    // Let's first deserialize into any possible JSON value to ensure the JSON
    // stream moves past this.
    let value = serde_json::Value::deserialize(deserializer)?;

    T::deserialize(value).or_else(|_| Ok(T::default()))
}
