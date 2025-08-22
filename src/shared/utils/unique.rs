//! Module to work with unique identifiers.

/// Generates a UUID v7 which is chronologically sortable and highly unique.
///
/// Uses the standard hexadecimal and hyphenated format.
pub fn generate_uuid_v7_string() -> String {
    uuid::Uuid::now_v7().hyphenated().to_string()
}
