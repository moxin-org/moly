use anyhow::Result;

use crate::crypto::decrypt_json;

/// Fetch and decrypt JSON from a sync server
pub async fn fetch_json(server_addr: &str, pin: &str) -> Result<String> {
    let url = if server_addr.starts_with("http") {
        format!("{}/preferences.json?token={}", server_addr, pin)
    } else {
        format!("http://{}/preferences.json?token={}", server_addr, pin)
    };

    let response = reqwest::get(&url).await?;
    if !response.status().is_success() {
        ::log::error!(
            "Failed to fetch preferences from server: {}",
            response.status()
        );
        anyhow::bail!(
            "Failed to fetch preferences from server: {}",
            response.status()
        );
    }

    let encrypted_content = response.text().await?;

    // Decrypt the content using the PIN
    let decrypted_content = decrypt_json(&encrypted_content, pin)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt preferences data: {}. This could be due to an incorrect PIN or corrupted data.", e))?;

    Ok(decrypted_content)
}

/// Test if server is reachable
pub async fn test_connection(server_addr: &str) -> Result<()> {
    let url = if server_addr.starts_with("http") {
        format!("{}/health", server_addr)
    } else {
        format!("http://{}/health", server_addr)
    };

    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        anyhow::bail!("Health check failed: {}", response.status())
    }
}
