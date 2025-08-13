use rmcp::ServiceExt;
use rmcp::service::{RunningService, DynService};
use rmcp::model::Tool;
use rmcp::RoleClient;
use rmcp::transport::{SseClientTransport, TokioChildProcess};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// The transport to use for the MCP server
pub enum McpTransport {
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
    
    async fn list_tools(&self) -> Result<rmcp::model::ListToolsResult, rmcp::service::ServiceError> {
        self.service.list_tools(Default::default()).await
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

    pub async fn add_server(&self, id: &str, transport: McpTransport) -> Result<(), Box<dyn std::error::Error>> {
        let running_service = match transport {
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
}
