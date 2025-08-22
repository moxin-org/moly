#[cfg(not(target_arch = "wasm32"))]
use futures::channel::{mpsc, oneshot};
#[cfg(not(target_arch = "wasm32"))]
use rmcp::ServiceExt;
#[cfg(not(target_arch = "wasm32"))]
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientWorker,
};
#[cfg(not(target_arch = "wasm32"))]
use rmcp::transport::{SseClientTransport, TokioChildProcess};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

use crate::protocol::Tool;

/// Creates an OpenAI-compatible namespaced tool name using double underscores
/// Normalizes server_id and tool_name by replacing hyphens with underscores
fn namespaced_name(server_id: &str, tool_name: &str) -> String {
    format!(
        "{}__{}",
        server_id.replace(['-'], "_"),
        tool_name.replace(['-'], "_")
    )
}

/// Parses a namespaced tool name into server_id and tool_name components
/// "filesystem__read_file" -> ("filesystem", "read_file")
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
pub fn display_name_from_namespaced(namespaced_name: &str) -> String {
    if let Ok((server_id, tool_name)) = parse_namespaced_tool_name(namespaced_name) {
        format!("{}: {}", server_id, tool_name)
    } else {
        // Fallback to original name if parsing fails
        namespaced_name.to_string()
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
    tools: HashMap<String, ToolRegistryEntry>,
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

// Conditional type aliases for WASM compatibility
#[cfg(not(target_arch = "wasm32"))]
type CallToolResult = rmcp::model::CallToolResult;
#[cfg(target_arch = "wasm32")]
type CallToolResult = serde_json::Value;

#[cfg(not(target_arch = "wasm32"))]
type ServiceError = rmcp::service::ServiceError;
#[cfg(target_arch = "wasm32")]
type ServiceError = String;

#[cfg(not(target_arch = "wasm32"))]
type ListToolsResult = rmcp::model::ListToolsResult;
#[cfg(target_arch = "wasm32")]
type ListToolsResult = Vec<Tool>;

// The transport to use for the MCP server
pub enum McpTransport {
    Http(String), // The URL for the HTTP endpoint (streamable)
    Sse(String),  // The URL for the SSE endpoint
    #[cfg(not(target_arch = "wasm32"))]
    Stdio(tokio::process::Command), // The command to launch the child process
}

// Message types for communicating with MCP service workers
#[cfg(not(target_arch = "wasm32"))]
enum McpRequest {
    ListTools {
        response: oneshot::Sender<Result<ListToolsResult, ServiceError>>,
    },
    CallTool {
        name: String,
        arguments: serde_json::Map<String, serde_json::Value>,
        response: oneshot::Sender<Result<CallToolResult, ServiceError>>,
    },
}

// WASM stub for McpRequest (not used but needed for compilation)
#[cfg(target_arch = "wasm32")]
enum McpRequest {
    #[allow(dead_code)]
    _Phantom,
}

// Handle for communicating with a spawned MCP service worker
#[cfg(not(target_arch = "wasm32"))]
struct ServiceHandle {
    sender: mpsc::UnboundedSender<McpRequest>,
    join_handle: tokio::task::JoinHandle<()>,
}

#[cfg(target_arch = "wasm32")]
struct ServiceHandle {
    _phantom: std::marker::PhantomData<()>,
}

#[derive(Clone)]
pub struct McpManagerClient {
    #[cfg(not(target_arch = "wasm32"))]
    services: Arc<Mutex<HashMap<String, ServiceHandle>>>,
    #[cfg(not(target_arch = "wasm32"))]
    registry: Arc<Mutex<ToolRegistry>>,
    latest_tools: Vec<Tool>,
}

impl McpManagerClient {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            services: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(not(target_arch = "wasm32"))]
            registry: Arc::new(Mutex::new(ToolRegistry::new())),
            latest_tools: Vec::new(),
        }
    }

    pub async fn add_server(
        &self,
        id: &str,
        transport: McpTransport,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
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

            // Create channel for communicating with the service worker
            let (sender, mut receiver) = mpsc::unbounded::<McpRequest>();

            // Spawn worker task that owns the RunningService
            let join_handle = tokio::spawn(async move {
                use futures::StreamExt;
                while let Some(request) = receiver.next().await {
                    match request {
                        McpRequest::ListTools { response } => {
                            let result = running_service.list_tools(Default::default()).await;
                            let _ = response.send(result);
                        }
                        McpRequest::CallTool {
                            name,
                            arguments,
                            response,
                        } => {
                            let request = rmcp::model::CallToolRequestParam {
                                name: name.into(),
                                arguments: Some(arguments),
                            };
                            let result = running_service.call_tool(request).await;
                            let _ = response.send(result);
                        }
                    }
                }
            });

            let service_handle = ServiceHandle {
                sender,
                join_handle,
            };

            self.services
                .lock()
                .unwrap()
                .insert(id.to_string(), service_handle);

            // Discover tools from the newly added server
            match self.discover_tools_for_server(id).await {
                Ok(tools) => {
                    self.registry.lock().unwrap().add_server_tools(id, tools);
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
        {
            let _ = (id, transport);
            Err("MCP servers are not supported in web builds".into())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn discover_tools_for_server(
        &self,
        server_id: &str,
    ) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let sender = {
            let services_guard = self.services.lock().unwrap();
            services_guard
                .get(server_id)
                .map(|handle| handle.sender.clone())
        };

        let Some(sender) = sender else {
            return Err(format!("Server '{}' not found", server_id).into());
        };

        let (response_tx, response_rx) = oneshot::channel();
        sender
            .unbounded_send(McpRequest::ListTools {
                response: response_tx,
            })
            .map_err(|_| "Service worker disconnected")?;

        let list_tools_result = response_rx
            .await
            .map_err(|_| "Service worker disconnected")??;

        let tools: Vec<Tool> = list_tools_result
            .tools
            .into_iter()
            .map(|rmcp_tool| rmcp_tool.into())
            .collect();

        Ok(tools)
    }

    /// Lists and caches tools from all connected MCP servers.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn list_tools(&mut self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let senders = {
            let services_guard = self.services.lock().unwrap();
            services_guard
                .values()
                .map(|handle| handle.sender.clone())
                .collect::<Vec<_>>()
        };

        let mut futures = Vec::new();
        for sender in senders {
            let (response_tx, response_rx) = oneshot::channel();
            if sender
                .unbounded_send(McpRequest::ListTools {
                    response: response_tx,
                })
                .is_ok()
            {
                futures.push(response_rx);
            }
        }

        let results = futures::future::join_all(futures).await;

        let mut all_tools = Vec::new();
        for result in results {
            match result {
                Ok(Ok(list_tools_result)) => {
                    // Convert rmcp tools to our unified Tool type
                    let converted_tools: Vec<Tool> = list_tools_result
                        .tools
                        .into_iter()
                        .map(|rmcp_tool| rmcp_tool.into())
                        .collect();
                    all_tools.extend(converted_tools);
                }
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => return Err("Service worker disconnected".into()),
            }
        }

        self.latest_tools = all_tools.clone();
        Ok(all_tools)
    }

    pub fn get_latest_tools(&self) -> Vec<Tool> {
        self.latest_tools.clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_all_namespaced_tools(&self) -> Vec<Tool> {
        self.registry.lock().unwrap().get_all_tools()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_all_namespaced_tools(&self) -> Vec<Tool> {
        Vec::new()
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn call_tool(
        &self,
        namespaced_tool_name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        // Parse the namespaced tool name to get server_id and original tool name
        let (server_id, original_tool_name) = parse_namespaced_tool_name(namespaced_tool_name)?;

        // Get the tool entry from registry for validation
        let tool_entry = {
            let registry = self.registry.lock().unwrap();
            registry.get_tool_entry(namespaced_tool_name).cloned()
        };

        let Some(_tool_entry) = tool_entry else {
            return Err(format!("Tool '{}' not found in registry. Available tools can be retrieved with get_all_namespaced_tools()", namespaced_tool_name).into());
        };

        // Get the specific server's sender
        let sender = {
            let services_guard = self.services.lock().unwrap();
            services_guard
                .get(&server_id)
                .map(|handle| handle.sender.clone())
        };

        let Some(sender) = sender else {
            return Err(format!("MCP server '{}' not found or disconnected", server_id).into());
        };

        // TODO: Add argument validation against tool_entry.schema here

        // Send the request to the specific server
        let (response_tx, response_rx) = oneshot::channel();
        let request = McpRequest::CallTool {
            name: original_tool_name.clone(),
            arguments,
            response: response_tx,
        };

        sender
            .unbounded_send(request)
            .map_err(|_| format!("Service worker for server '{}' disconnected", server_id))?;

        // Wait for response from the specific server
        match response_rx.await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => {
                let error_message = format!(
                    "Tool '{}' failed on server '{}': {}",
                    original_tool_name, server_id, e
                );
                Err(error_message.into())
            }
            Err(_) => Err(format!(
                "Service worker for server '{}' disconnected during tool execution",
                server_id
            )
            .into()),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn call_tool(
        &self,
        tool_name: &str,
        _arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        Err(format!(
            "MCP servers are not yet supported in WASM builds. Cannot call tool '{}'",
            tool_name
        )
        .into())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn remove_server(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(handle) = self.services.lock().unwrap().remove(id) {
            // Drop the sender to signal the worker to stop
            drop(handle.sender);
            // Wait for the worker to finish
            let _ = handle.join_handle.await;
        }

        // Remove tools from registry
        self.registry.lock().unwrap().remove_server(id);

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn remove_server(&self, _id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
