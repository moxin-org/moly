use rmcp::RoleClient;
use rmcp::ServiceExt;
use rmcp::model::Tool;
use rmcp::service::{DynService, RunningService};
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientWorker,
};
use rmcp::transport::{SseClientTransport, TokioChildProcess};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// The transport to use for the MCP server
pub enum McpTransport {
    Http(String),                   // The URL for the HTTP endpoint (streamable)
    Sse(String),                    // The URL for the SSE endpoint
    Stdio(tokio::process::Command), // The command to launch the child process
}

// A wrapper around the service that implements Send + Sync
struct McpService {
    service: RunningService<RoleClient, Box<dyn DynService<RoleClient>>>,
}

// Safety: We control the usage of this wrapper and ensure thread safety
unsafe impl Send for McpService {}
unsafe impl Sync for McpService {}

impl McpService {
    fn new(service: RunningService<RoleClient, Box<dyn DynService<RoleClient>>>) -> Self {
        Self { service }
    }

    async fn list_tools(
        &self,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::service::ServiceError> {
        self.service.list_tools(Default::default()).await
    }

    async fn call_tool(
        &self,
        name: String,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::service::ServiceError> {
        let request = rmcp::model::CallToolRequestParam {
            name: name.into(),
            arguments: Some(arguments),
        };
        self.service.call_tool(request).await
    }
}

#[derive(Clone)]
pub struct McpManagerClient {
    clients: Arc<Mutex<HashMap<String, Arc<McpService>>>>,
}

impl McpManagerClient {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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

        let service = Arc::new(McpService::new(running_service));
        self.clients.lock().unwrap().insert(id.to_string(), service);

        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let clients = {
            let clients_guard = self.clients.lock().unwrap();
            clients_guard.values().cloned().collect::<Vec<_>>()
        };

        let futures: Vec<_> = clients.iter().map(|client| client.list_tools()).collect();
        let results = futures::future::join_all(futures).await;

        let mut all_tools = Vec::new();
        for result in results {
            match result {
                Ok(list_tools_result) => {
                    all_tools.extend(list_tools_result.tools);
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(all_tools)
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<rmcp::model::CallToolResult, Box<dyn std::error::Error>> {
        let clients = {
            let clients_guard = self.clients.lock().unwrap();
            clients_guard.values().cloned().collect::<Vec<_>>()
        };

        let mut tool_not_found_errors = Vec::new();
        let mut execution_errors = Vec::new();

        // Try to call the tool on each client until we find one that has it
        for client in clients {
            match client
                .call_tool(tool_name.to_string(), arguments.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
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
                            "Tool '{}' not found on this client: {}",
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
            }
        }

        // If we got here, the tool wasn't found in any client
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
}
