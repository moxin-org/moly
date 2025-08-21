#[cfg(not(target_arch = "wasm32"))]
use futures::channel::{mpsc, oneshot};
#[cfg(not(target_arch = "wasm32"))]
use rmcp::RoleClient;
#[cfg(not(target_arch = "wasm32"))]
use rmcp::ServiceExt;
#[cfg(not(target_arch = "wasm32"))]
use rmcp::service::{DynService, RunningService};
#[cfg(not(target_arch = "wasm32"))]
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientWorker,
};
#[cfg(not(target_arch = "wasm32"))]
use rmcp::transport::{SseClientTransport, TokioChildProcess};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

use crate::protocol::Tool;

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
    latest_tools: Vec<Tool>,
    #[cfg(target_arch = "wasm32")]
    _phantom: std::marker::PhantomData<()>,
}

impl McpManagerClient {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            services: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(not(target_arch = "wasm32"))]
            latest_tools: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            _phantom: std::marker::PhantomData,
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

            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (id, transport);
            Err("MCP servers are not supported in web builds".into())
        }
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

    #[cfg(target_arch = "wasm32")]
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        let senders = {
            let services_guard = self.services.lock().unwrap();
            services_guard
                .values()
                .map(|handle| handle.sender.clone())
                .collect::<Vec<_>>()
        };

        let mut tool_not_found_errors = Vec::new();
        let mut execution_errors = Vec::new();

        // Try to call the tool on each service until we find one that has it
        for sender in senders {
            let (response_tx, response_rx) = oneshot::channel();
            let request = McpRequest::CallTool {
                name: tool_name.to_string(),
                arguments: arguments.clone(),
                response: response_tx,
            };

            if sender.unbounded_send(request).is_err() {
                continue; // Service worker disconnected, try next one
            }

            match response_rx.await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => {
                    // Get the error message for analysis
                    let error_string = e.to_string();
                    let debug_string = format!("{:?}", e);

                    // More sophisticated error categorization
                    let is_not_found = error_string.contains("not found")
                        || error_string.contains("unknown")
                        || error_string.contains("does not exist")
                        || debug_string.contains("not found")
                        || debug_string.contains("unknown")
                        || debug_string.contains("does not exist");

                    let is_validation_error = error_string.contains("invalid")
                        || error_string.contains("argument")
                        || error_string.contains("parameter")
                        || error_string.contains("schema")
                        || error_string.contains("validation")
                        || debug_string.contains("ValidationError")
                        || debug_string.contains("InvalidInput");

                    if is_not_found {
                        tool_not_found_errors.push(error_string.clone());
                        println!(
                            "Tool '{}' not found on this service: {}",
                            tool_name, error_string
                        );
                    } else if is_validation_error {
                        // This is an argument validation error - tool exists but args are wrong
                        println!(
                            "Tool '{}' found but validation failed: {}",
                            tool_name, error_string
                        );
                        return Err(format!(
                            "Tool '{}' validation failed: {}",
                            tool_name, error_string
                        )
                        .into());
                    } else {
                        // This is some other execution error
                        execution_errors.push(error_string.clone());
                        println!("Tool '{}' execution error: {}", tool_name, error_string);
                        return Err(format!(
                            "Tool '{}' execution failed: {}",
                            tool_name, error_string
                        )
                        .into());
                    }
                }
                Err(_) => {
                    // Service worker disconnected
                    continue;
                }
            }
        }

        // If we got here, the tool wasn't found in any service
        if !execution_errors.is_empty() {
            // We had execution errors, return the first one
            Err(format!(
                "Tool '{}' failed to execute: {}",
                tool_name, execution_errors[0]
            )
            .into())
        } else {
            // All errors were "not found" errors
            Err(format!("Tool '{}' not found in any connected MCP server", tool_name).into())
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
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn remove_server(&self, _id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
