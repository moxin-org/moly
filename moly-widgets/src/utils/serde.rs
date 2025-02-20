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
