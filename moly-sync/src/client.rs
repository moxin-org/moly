use anyhow::Result;

/// Fetch JSON from a sync server
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

    let content = response.text().await?;
    Ok(content)
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
