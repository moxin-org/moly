use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an input configuration for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub description: String,
    #[serde(default)]
    pub password: bool,
}

/// Represents an MCP server configuration following the standard format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    // Stdio transport fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    
    // HTTP/SSE transport fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transport_type: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    
    // Optional extras
    #[serde(default = "default_enabled", skip_serializing_if = "is_default_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

fn default_enabled() -> bool {
    true
}

fn is_default_enabled(enabled: &bool) -> bool {
    *enabled
}

impl McpServer {
    /// Create a new stdio-based MCP server
    pub fn stdio(command: String, args: Vec<String>) -> Self {
        Self {
            command: Some(command),
            args,
            env: HashMap::new(),
            url: None,
            transport_type: None,
            headers: HashMap::new(),
            enabled: true,
            working_directory: None,
        }
    }
    
    /// Create a new HTTP-based MCP server
    pub fn http(url: String) -> Self {
        Self {
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            url: Some(url),
            transport_type: Some("http".to_string()),
            headers: HashMap::new(),
            enabled: true,
            working_directory: None,
        }
    }
    
    /// Create a new SSE-based MCP server
    pub fn sse(url: String) -> Self {
        Self {
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            url: Some(url),
            transport_type: Some("sse".to_string()),
            headers: HashMap::new(),
            enabled: true,
            working_directory: None,
        }
    }
    
    /// Determine the transport type based on configuration
    pub fn get_transport_type(&self) -> Option<&str> {
        if self.command.is_some() {
            Some("stdio")
        } else if self.url.is_some() {
            self.transport_type.as_deref().or(Some("http"))
        } else {
            None
        }
    }
    
    /// Check if this is a stdio transport
    pub fn is_stdio(&self) -> bool {
        self.command.is_some()
    }
    
    /// Check if this is an HTTP/SSE transport
    pub fn is_network(&self) -> bool {
        self.url.is_some()
    }
    
    /// Set environment variables for stdio transport
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }
    
    /// Set working directory for stdio transport
    pub fn with_working_directory(mut self, working_directory: String) -> Self {
        self.working_directory = Some(working_directory);
        self
    }
    
    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set headers for HTTP/SSE transport
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
}

impl McpServer {
    /// Convert this server configuration to a transport for the MCP manager
    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_transport(&self) -> Option<moly_kit::mcp::mcp_manager::McpTransport> {
        if let Some(command_str) = &self.command {
            // Stdio transport
            let mut command = tokio::process::Command::new(command_str);
            command.args(&self.args);
            
            // Add environment variables
            for (key, value) in &self.env {
                command.env(key, value);
            }
            
            // Set working directory if specified
            if let Some(working_dir) = &self.working_directory {
                command.current_dir(working_dir);
            }
            
            Some(moly_kit::mcp::mcp_manager::McpTransport::Stdio(command))
        } else if let Some(url) = &self.url {
            // Network transport - determine if HTTP or SSE
            match self.transport_type.as_deref() {
                Some("sse") => Some(moly_kit::mcp::mcp_manager::McpTransport::Sse(url.clone())),
                _ => Some(moly_kit::mcp::mcp_manager::McpTransport::Http(url.clone())),
            }
        } else {
            None
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn to_transport(&self) -> Option<()> {
        None
    }
}

/// Represents the complete MCP servers configuration (follows MCP standard format)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServersConfig {
    pub servers: HashMap<String, McpServer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputConfig>,
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
    
    pub fn add_input(&mut self, input: InputConfig) {
        self.inputs.push(input);
    }
    
    pub fn get_input(&self, id: &str) -> Option<&InputConfig> {
        self.inputs.iter().find(|input| input.id == id)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn create_sample() -> Self {
        let mut config = Self::new();

        // Filesystem server (stdio)
        config.add_server(
            "filesystem".to_string(),
            McpServer::stdio(
                "npx".to_string(),
                vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    "/Users/username/Desktop".to_string(),
                    "/Users/username/Downloads".to_string(),
                ],
            ).with_enabled(false),
        );

        // Image sorcery server with environment variable
        let mut env = HashMap::new();
        env.insert("SECRET_TOKEN".to_string(), "${input:secret_token}".to_string());
        config.add_server(
            "imagesorcery".to_string(),
            McpServer::stdio(
                "uvx".to_string(),
                vec!["imagesorcery-mcp".to_string()],
            ).with_env(env).with_enabled(false),
        );

        // Node server with working directory
        config.add_server(
            "my-node-server".to_string(),
            McpServer::stdio(
                "node".to_string(),
                vec!["server.js".to_string()],
            ).with_working_directory("/path/to/mcp/server".to_string()).with_enabled(false),
        );

        // HTTP server example
        config.add_server(
            "my-mcp-server-4b11bf70".to_string(),
            McpServer::http("http://localhost:8931".to_string()),
        );

        // SSE server example
        config.add_server(
            "browser".to_string(),
            McpServer::sse("http://localhost:8931/sse".to_string()).with_enabled(false),
        );

        // Add sample input configuration
        config.inputs.push(InputConfig {
            id: "secret_token".to_string(),
            input_type: "promptString".to_string(),
            description: "Secret Token for MCP servers".to_string(),
            password: true,
        });

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
        assert_eq!(config.inputs.len(), deserialized.inputs.len());
    }

    #[test]
    fn test_mcp_standard_format() {
        let config = McpServersConfig::create_sample();
        let json = config.to_json().unwrap();
        println!("Generated JSON: {}", json);

        // Verify it follows the MCP standard structure
        assert!(json.contains("\"servers\""));
        assert!(json.contains("\"inputs\""));
        
        // Verify stdio servers have command field
        assert!(json.contains("\"command\": \"npx\""));
        assert!(json.contains("\"command\": \"uvx\""));
        
        // Verify network servers have url field
        assert!(json.contains("\"url\": \"http://localhost:8931\""));
        assert!(json.contains("\"url\": \"http://localhost:8931/sse\""));
        
        // Verify transport types
        assert!(json.contains("\"type\": \"http\""));
        assert!(json.contains("\"type\": \"sse\""));
    }
    
    #[test]
    fn test_stdio_server_creation() {
        let server = McpServer::stdio(
            "node".to_string(),
            vec!["server.js".to_string()]
        ).with_working_directory("/path/to/server".to_string());
        
        assert!(server.is_stdio());
        assert!(!server.is_network());
        assert_eq!(server.get_transport_type(), Some("stdio"));
        assert_eq!(server.command.as_ref().unwrap(), "node");
        assert_eq!(server.args, vec!["server.js"]);
        assert_eq!(server.working_directory.as_ref().unwrap(), "/path/to/server");
    }
    
    #[test]
    fn test_http_server_creation() {
        let server = McpServer::http("http://localhost:8080".to_string());
        
        assert!(!server.is_stdio());
        assert!(server.is_network());
        assert_eq!(server.get_transport_type(), Some("http"));
        assert_eq!(server.url.as_ref().unwrap(), "http://localhost:8080");
        assert_eq!(server.transport_type.as_ref().unwrap(), "http");
    }
    
    #[test]
    fn test_sse_server_creation() {
        let server = McpServer::sse("http://localhost:8080/sse".to_string());
        
        assert!(!server.is_stdio());
        assert!(server.is_network());
        assert_eq!(server.get_transport_type(), Some("sse"));
        assert_eq!(server.url.as_ref().unwrap(), "http://localhost:8080/sse");
        assert_eq!(server.transport_type.as_ref().unwrap(), "sse");
    }
    
    #[test]
    fn test_claude_format_compatibility() {
        // Test that we can parse VS Code format
        let vscode_json = r#"{
            "servers": {
                "github": {
                    "url": "https://api.githubcopilot.com/mcp/",
                    "type": "http"
                },
                "imagesorcery": {
                    "command": "uvx",
                    "args": ["imagesorcery-mcp"]
                }
            },
            "inputs": []
        }"#;
        
        let config = McpServersConfig::from_json(vscode_json).unwrap();
        assert_eq!(config.servers.len(), 2);
        
        let github_server = config.get_server("github").unwrap();
        assert!(github_server.is_network());
        assert_eq!(github_server.url.as_ref().unwrap(), "https://api.githubcopilot.com/mcp/");
        
        let imagesorcery_server = config.get_server("imagesorcery").unwrap();
        assert!(imagesorcery_server.is_stdio());
        assert_eq!(imagesorcery_server.command.as_ref().unwrap(), "uvx");
        assert_eq!(imagesorcery_server.args, vec!["imagesorcery-mcp"]);
    }
}
