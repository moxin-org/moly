use serde::Deserialize;

use super::providers::ProviderType;

#[derive(Debug, Deserialize)]
pub struct SupportedProvidersFile {
    pub providers: Vec<SupportedProvider>,
}

/// Represents a supported provider, used as a template.
#[derive(Debug, Deserialize)]
pub struct SupportedProvider {
    pub name: String,
    pub url: String,
    pub provider_type: ProviderType,
    pub supported_models: Option<Vec<String>>,
}

/// Utility to load from the JSON file
pub fn load_supported_providers() -> Vec<SupportedProvider> {

    let data = include_str!("./supported_providers.json");
    let parsed: SupportedProvidersFile = serde_json::from_str(data)
        .expect("Failed to parse supported_providers.json");
    parsed.providers
}
