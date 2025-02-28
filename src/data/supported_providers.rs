use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SupportedProvidersFile {
    pub providers: Vec<SupportedProvider>,
}

#[derive(Debug, Deserialize)]
pub struct SupportedProvider {
    pub name: String,
    pub url: String,
    // TODO(Julian): this should be an enum
    pub provider_type: String
}

// Utility to load from the JSON file
pub fn load_supported_providers() -> Vec<SupportedProvider> {

    let data = include_str!("./supported_providers.json");
    let parsed: SupportedProvidersFile = serde_json::from_str(data)
        .expect("Failed to parse supported_providers.json");
    parsed.providers
}