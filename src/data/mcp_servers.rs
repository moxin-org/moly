use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the transport method for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpServerTransport {
    #[serde(rename = "http")]
    Http { url: String },
    #[serde(rename = "sse")]
    Sse { url: String },
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        #[serde(default)]
        working_directory: Option<String>,
    },
}

/// Represents an MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    #[serde(flatten)]
    pub transport: McpServerTransport,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Represents the complete MCP servers configuration (follows MCP standard format)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServersConfig {
    pub servers: HashMap<String, McpServer>,
    #[serde(default)]
    pub inputs: HashMap<String, serde_json::Value>,
}

impl McpServersConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_server(&mut self, id: String, server: McpServer) {
        self.servers.insert(id, server);
    }

    pub fn remove_server(&mut self, id: &str) {
        self.servers.remove(id);
    }

    pub fn get_server(&self, id: &str) -> Option<&McpServer> {
        self.servers.get(id)
    }

    pub fn list_enabled_servers(&self) -> impl Iterator<Item = (&String, &McpServer)> {
        self.servers.iter().filter(|(_, server)| server.enabled)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn create_sample() -> Self {
        let mut config = Self::new();

        config.add_server(
            "filesystem".to_string(),
            McpServer {
                transport: McpServerTransport::Stdio {
                    command: "npx".to_string(),
                    args: vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                        "/Users/username/Desktop".to_string(),
                        "/Users/username/Downloads".to_string(),
                    ],
                    env: HashMap::new(),
                    working_directory: None,
                },
                enabled: false,
            },
        );

        // Local browser automation example (stdio)
        config.add_server(
            "my-node-server".to_string(),
            McpServer {
                transport: McpServerTransport::Stdio {
                    command: "node".to_string(),
                    args: vec!["server.js".to_string()],
                    env: HashMap::new(),
                    working_directory: Some("/path/to/mcp/server".to_string()),
                },
                enabled: false,
            },
        );

        // HTTP server example
        config.add_server(
            "my-mcp-server-4b11bf70".to_string(),
            McpServer {
                transport: McpServerTransport::Http {
                    url: "http://localhost:8931".to_string(),
                },
                enabled: true,
            },
        );

        // Local SSE server example
        config.add_server(
            "browser".to_string(),
            McpServer {
                transport: McpServerTransport::Sse {
                    url: "http://localhost:8931/sse".to_string(),
                },
                enabled: false,
            },
        );

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let config = McpServersConfig::create_sample();
        let json = config.to_json().unwrap();
        let deserialized = McpServersConfig::from_json(&json).unwrap();

        assert_eq!(config.servers.len(), deserialized.servers.len());
    }

    #[test]
    fn test_mcp_standard_format() {
        let config = McpServersConfig::create_sample();
        let json = config.to_json().unwrap();

        // Verify it follows the MCP standard structure
        assert!(json.contains("\"servers\""));
        assert!(json.contains("\"inputs\""));
        assert!(json.contains("\"type\": \"http\""));
        assert!(json.contains("\"type\": \"sse\""));
        assert!(json.contains("\"type\": \"stdio\""));
    }
}
