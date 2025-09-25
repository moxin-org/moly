#[cfg(not(target_arch = "wasm32"))]
use rmcp::{
    model::{CallToolRequestParam, CallToolResult},
    service::{RoleClient, RunningService, ServiceExt},
    transport::{
        SseClientTransport, TokioChildProcess,
        streamable_http_client::{StreamableHttpClientTransport, StreamableHttpClientWorker},
    },
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::protocol::{Tool, ToolCall, ToolResult};

/// Creates a namespaced tool name using double underscores as separator
/// Preserves original naming including hyphens and casing
fn namespaced_name(server_id: &str, tool_name: &str) -> String {
    format!("{}__{}", server_id, tool_name)
}

/// Parses a namespaced tool name into server_id and tool_name components
/// "filesystem__read_file" -> ("filesystem", "read_file")
/// "mcp-internet-speed__test-speed" -> ("mcp-internet-speed", "test-speed")
pub fn parse_namespaced_tool_name(
    namespaced_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = namespaced_name.splitn(2, "__").collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid namespaced tool name: '{}'. Expected format 'server_id__tool_name'",
            namespaced_name
        )
        .into());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Converts a namespaced tool name to a display-friendly format for UI
/// "filesystem__read_file" -> "filesystem: read_file"
/// "mcp-internet-speed__test-speed" -> "mcp-internet-speed: test-speed"
pub fn display_name_from_namespaced(namespaced_name: &str) -> String {
    if let Ok((server_id, tool_name)) = parse_namespaced_tool_name(namespaced_name) {
        format!("{}: {}", server_id, tool_name)
    } else {
        // Fallback to original name if parsing fails
        namespaced_name.to_string()
    }
}

/// Parse tool arguments from JSON string to Map
pub fn parse_tool_arguments(arguments: &str) -> Result<Map<String, Value>, String> {
    match serde_json::from_str::<Value>(arguments) {
        Ok(Value::Object(args)) => Ok(args),
        Ok(_) => Err("Arguments must be a JSON object".to_string()),
        Err(e) => Err(format!("Failed to parse arguments: {}", e)),
    }
}

#[derive(Clone, Debug)]
pub struct ToolRegistryEntry {
    pub server_id: String,
    pub original_name: String,
    pub namespaced_name: String,
    pub schema: Tool,
}

pub struct ToolRegistry {
    /// A map of all tools, keyed by their namespaced name.
    tools: HashMap<String, ToolRegistryEntry>,
    /// A map of all tools, keyed by their server_id.
    server_tools: HashMap<String, Vec<String>>,
}

impl ToolRegistry {
    fn new() -> Self {
        Self {
            tools: HashMap::new(),
            server_tools: HashMap::new(),
        }
    }

    fn add_server_tools(&mut self, server_id: &str, tools: Vec<Tool>) {
        let mut tool_names = Vec::new();

        for tool in tools {
            let namespaced_name = namespaced_name(server_id, &tool.name);
            let original_name = tool.name.clone();
            let entry = ToolRegistryEntry {
                server_id: server_id.to_string(),
                original_name: original_name.clone(),
                namespaced_name: namespaced_name.clone(),
                schema: tool,
            };

            self.tools.insert(namespaced_name, entry);
            tool_names.push(original_name);
        }

        self.server_tools.insert(server_id.to_string(), tool_names);
    }

    fn get_tool_entry(&self, namespaced_name: &str) -> Option<&ToolRegistryEntry> {
        self.tools.get(namespaced_name)
    }

    fn get_all_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|entry| {
                let mut tool = entry.schema.clone();
                tool.name = entry.namespaced_name.clone();
                tool
            })
            .collect()
    }

    fn remove_server(&mut self, server_id: &str) {
        if let Some(tool_names) = self.server_tools.remove(server_id) {
            for tool_name in tool_names {
                let namespaced_name = namespaced_name(server_id, &tool_name);
                self.tools.remove(&namespaced_name);
            }
        }
    }
}

// The transport to use for the MCP server
pub enum McpTransport {
    Http(String), // The URL for the HTTP endpoint (streamable)
    Sse(String),  // The URL for the SSE endpoint
    #[cfg(not(target_arch = "wasm32"))]
    Stdio(tokio::process::Command), // The command to launch the child process
}

#[cfg(not(target_arch = "wasm32"))]
type DynService = Box<dyn rmcp::service::DynService<RoleClient>>;

#[cfg(not(target_arch = "wasm32"))]
type McpService = RunningService<RoleClient, DynService>;

#[cfg(not(target_arch = "wasm32"))]
type McpServiceHandle = Arc<McpService>;

#[cfg(not(target_arch = "wasm32"))]
type McpServiceRegistry = HashMap<String, McpServiceHandle>;

struct McpManagerInner {
    #[cfg(not(target_arch = "wasm32"))]
    services: Mutex<McpServiceRegistry>,
    #[cfg(not(target_arch = "wasm32"))]
    registry: Mutex<ToolRegistry>,
    latest_tools: Mutex<Vec<Tool>>,
    dangerous_mode_enabled: AtomicBool,
}

/// Manages MCP servers and provides a unified interface for tool discovery and invocation.
pub struct McpManagerClient {
    inner: Arc<McpManagerInner>,
}

impl Clone for McpManagerClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl McpManagerClient {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(McpManagerInner {
                #[cfg(not(target_arch = "wasm32"))]
                services: Mutex::new(HashMap::new()),
                #[cfg(not(target_arch = "wasm32"))]
                registry: Mutex::new(ToolRegistry::new()),
                latest_tools: Mutex::new(Vec::new()),
                dangerous_mode_enabled: AtomicBool::new(false),
            }),
        }
    }

    /// Registers a new MCP server in the registry, and discovers tools from the server.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add_server(
        &self,
        id: &str,
        transport: McpTransport,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let running_service = match transport {
            McpTransport::Http(url) => {
                let worker = StreamableHttpClientWorker::<reqwest::Client>::new_simple(url);
                let transport = StreamableHttpClientTransport::spawn(worker);
                ().into_dyn().serve(transport).await?
            }
            McpTransport::Sse(url) => {
                let transport = SseClientTransport::start(url).await?;
                ().into_dyn().serve(transport).await?
            }
            McpTransport::Stdio(command) => {
                let transport = TokioChildProcess::new(command)?;
                ().into_dyn().serve(transport).await?
            }
        };

        self.inner.services
            .lock()
            .unwrap()
            .insert(id.to_string(), Arc::new(running_service));

        // Discover tools from the newly added server
        match self.discover_tools_for_server(id).await {
            Ok(tools) => {
                self.inner.registry.lock().unwrap().add_server_tools(id, tools);
                ::log::debug!("Successfully discovered tools for MCP server: {}", id);
            }
            Err(e) => {
                ::log::warn!("Failed to discover tools for MCP server '{}': {}", id, e);
                // Don't fail the entire server addition if tool discovery fails
            }
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn add_server(
        &self,
        id: &str,
        _transport: McpTransport,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = id;
        Err("MCP servers are not supported in web builds".into())
    }

    pub fn set_dangerous_mode_enabled(&self, enabled: bool) {
        self.inner.dangerous_mode_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn get_dangerous_mode_enabled(&self) -> bool {
        self.inner.dangerous_mode_enabled.load(Ordering::Relaxed)
    }

    /// Discovers tools from an MCP server.
    #[cfg(not(target_arch = "wasm32"))]
    async fn discover_tools_for_server(
        &self,
        server_id: &str,
    ) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let service = {
            let services_guard = self.inner.services.lock().unwrap();
            services_guard.get(server_id).map(|s| Arc::clone(s))
        };

        let Some(service) = service else {
            return Err(format!("Server '{}' not found", server_id).into());
        };

        let list_tools_result = service.list_tools(Default::default()).await?;

        let tools: Vec<Tool> = list_tools_result
            .tools
            .into_iter()
            .map(|rmcp_tool| rmcp_tool.into())
            .collect();

        Ok(tools)
    }

    /// Lists and caches tools from all connected MCP servers.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let services: Vec<McpServiceHandle> = {
            let services_guard = self.inner.services.lock().unwrap();
            services_guard.values().map(|s| Arc::clone(s)).collect()
        };

        let mut all_tools = Vec::new();
        for service in services {
            match service.list_tools(Default::default()).await {
                Ok(list_tools_result) => {
                    // Convert rmcp tools to our unified Tool type
                    let converted_tools: Vec<Tool> = list_tools_result
                        .tools
                        .into_iter()
                        .map(|rmcp_tool| rmcp_tool.into())
                        .collect();
                    all_tools.extend(converted_tools);
                }
                Err(e) => {
                    ::log::warn!("Failed to list tools from server: {}", e);
                    // Continue with other servers
                }
            }
        }

        *self.inner.latest_tools.lock().unwrap() = all_tools.clone();
        Ok(all_tools)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }

    pub fn get_latest_tools(&self) -> Vec<Tool> {
        self.inner.latest_tools.lock().unwrap().clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_all_namespaced_tools(&self) -> Vec<Tool> {
        self.inner.registry.lock().unwrap().get_all_tools()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_all_namespaced_tools(&self) -> Vec<Tool> {
        Vec::new()
    }

    /// Calls a tool on an MCP server.
    #[cfg(not(target_arch = "wasm32"))]
    async fn call_tool(
        &self,
        namespaced_tool_name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        // Parse the namespaced tool name to get server_id and original tool name
        let (server_id, original_tool_name) = parse_namespaced_tool_name(namespaced_tool_name)?;

        // Get the tool entry from registry for validation
        let tool_entry = {
            let registry = self.inner.registry.lock().unwrap();
            registry.get_tool_entry(namespaced_tool_name).cloned()
        };

        let Some(_tool_entry) = tool_entry else {
            return Err(format!("Tool '{}' not found in registry. Available tools can be retrieved with get_all_namespaced_tools()", namespaced_tool_name).into());
        };

        // Get the specific server
        let service = {
            let services_guard = self.inner.services.lock().unwrap();
            services_guard.get(&server_id).map(|s| Arc::clone(s))
        };

        let Some(service) = service else {
            return Err(format!("MCP server '{}' not found or disconnected", server_id).into());
        };

        // TODO: Add argument validation against tool_entry.schema here

        // Call the tool directly on the service
        let request = CallToolRequestParam {
            name: original_tool_name.clone().into(),
            arguments: Some(arguments),
        };

        match service.call_tool(request).await {
            Ok(result) => Ok(result),
            Err(e) => {
                let error_message = format!(
                    "Tool '{}' failed on server '{}': {}",
                    original_tool_name, server_id, e
                );
                Err(error_message.into())
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn call_tool(
        &self,
        tool_name: &str,
        _arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Err(format!(
            "MCP servers are not yet supported in WASM builds. Cannot call tool '{}'",
            tool_name
        )
        .into())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn remove_server(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.services.lock().unwrap().remove(id);
        self.inner.registry.lock().unwrap().remove_server(id);
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn remove_server(&self, _id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Executes a tool call and returns the result
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn execute_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: &str,
        arguments: Map<String, Value>,
    ) -> ToolResult {
        match self.call_tool(tool_name, arguments).await {
            Ok(result) => {
                // Convert result to content string
                let content = result
                    .content
                    .iter()
                    .filter_map(|item| {
                        // Convert ContentPart to text - for now we just serialize it
                        if let Ok(text) = serde_json::to_string(item) {
                            Some(text)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                ToolResult {
                    tool_call_id: tool_call_id.to_string(),
                    content,
                    is_error: false,
                }
            }
            Err(e) => ToolResult {
                tool_call_id: tool_call_id.to_string(),
                content: e.to_string(),
                is_error: true,
            },
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn execute_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: &str,
        _arguments: Map<String, Value>,
    ) -> ToolResult {
        ToolResult {
            tool_call_id: tool_call_id.to_string(),
            content: format!(
                "MCP servers are not yet supported in WASM builds. Cannot call tool '{}'",
                tool_name
            ),
            is_error: true,
        }
    }

    /// Executes multiple tool calls sequentially and returns the results
    pub async fn execute_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult> {
        let mut tool_results = Vec::new();

        // Execute all tool calls sequentially
        for tool_call in tool_calls {
            let result = self
                .execute_tool_call(&tool_call.name, &tool_call.id, tool_call.arguments.clone())
                .await;
            tool_results.push(result);
        }

        tool_results
    }
}
