use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const PBKDF2_ITERATIONS: u32 = 100_000;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;

/// Encrypted data format that includes salt, nonce, and ciphertext
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded salt used for key derivation
    pub salt: String,
    /// Base64-encoded nonce used for encryption
    pub nonce: String,
    /// Base64-encoded encrypted data
    pub data: String,
}

/// Derive an AES-256 key from a PIN and salt using PBKDF2
fn derive_key(pin: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(pin.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Encrypt JSON data using AES-256-GCM with a PIN-derived key
///
/// Returns base64-encoded JSON containing salt, nonce, and encrypted data
pub fn encrypt_json(json_data: &str, pin: &str) -> Result<String> {
    // Generate random salt and nonce
    let mut salt = [0u8; SALT_SIZE];
    let mut nonce_bytes = [0u8; NONCE_SIZE];

    getrandom::getrandom(&mut salt)
        .map_err(|e| anyhow::anyhow!("Failed to generate random salt: {}", e))?;
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to generate random nonce: {}", e))?;

    // Derive key from PIN and salt
    let key_bytes = derive_key(pin, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Create nonce
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the JSON data
    let ciphertext = cipher
        .encrypt(nonce, json_data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Create encrypted data structure
    let encrypted_data = EncryptedData {
        salt: BASE64.encode(&salt),
        nonce: BASE64.encode(&nonce_bytes),
        data: BASE64.encode(&ciphertext),
    };

    // Return as JSON string
    serde_json::to_string(&encrypted_data).context("Failed to serialize encrypted data")
}

/// Decrypt JSON data using AES-256-GCM with a PIN-derived key
///
/// Takes base64-encoded JSON containing salt, nonce, and encrypted data
/// Returns the original JSON string
pub fn decrypt_json(encrypted_json: &str, pin: &str) -> Result<String> {
    // Parse the encrypted data structure
    let encrypted_data: EncryptedData =
        serde_json::from_str(encrypted_json).context("Failed to parse encrypted data JSON")?;

    // Decode base64 components
    let salt = BASE64
        .decode(&encrypted_data.salt)
        .context("Failed to decode salt from base64")?;
    let nonce_bytes = BASE64
        .decode(&encrypted_data.nonce)
        .context("Failed to decode nonce from base64")?;
    let ciphertext = BASE64
        .decode(&encrypted_data.data)
        .context("Failed to decode ciphertext from base64")?;

    // Validate sizes
    if salt.len() != SALT_SIZE {
        anyhow::bail!(
            "Invalid salt size: expected {}, got {}",
            SALT_SIZE,
            salt.len()
        );
    }
    if nonce_bytes.len() != NONCE_SIZE {
        anyhow::bail!(
            "Invalid nonce size: expected {}, got {}",
            NONCE_SIZE,
            nonce_bytes.len()
        );
    }

    // Derive key from PIN and salt
    let key_bytes = derive_key(pin, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Create nonce
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    // Convert back to string
    String::from_utf8(plaintext).context("Decrypted data is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original_data = r#"{"test": "data", "number": 42}"#;
        let pin = "1234";

        let encrypted = encrypt_json(original_data, pin).unwrap();
        let decrypted = decrypt_json(&encrypted, pin).unwrap();

        assert_eq!(original_data, decrypted);
    }

    #[test]
    fn test_wrong_pin_fails() {
        let original_data = r#"{"test": "data"}"#;
        let pin = "1234";
        let wrong_pin = "5678";

        let encrypted = encrypt_json(original_data, pin).unwrap();
        let result = decrypt_json(&encrypted, wrong_pin);

        assert!(result.is_err());
    }

    #[test]
    fn test_different_encryptions_produce_different_results() {
        let data = r#"{"test": "data"}"#;
        let pin = "1234";

        let encrypted1 = encrypt_json(data, pin).unwrap();
        let encrypted2 = encrypt_json(data, pin).unwrap();

        // Should be different due to random salt and nonce
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same original data
        assert_eq!(decrypt_json(&encrypted1, pin).unwrap(), data);
        assert_eq!(decrypt_json(&encrypted2, pin).unwrap(), data);
    }
}
