use crate::mcp::mcp_manager::McpManagerClient;
use crate::protocol::{ToolCall, ToolResult};
use serde_json::{Map, Value};

/// Executes a tool call and returns the result
pub async fn execute_tool_call(
    tool_manager: McpManagerClient,
    tool_name: &str,
    tool_call_id: &str,
    arguments: Map<String, Value>,
) -> ToolResult {
    match tool_manager.call_tool(tool_name, arguments).await {
        Ok(result) => {
            // Convert result to content string
            #[cfg(not(target_arch = "wasm32"))]
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

            #[cfg(target_arch = "wasm32")]
            let content = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Tool executed successfully".to_string());

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

/// Executes multiple tool calls in parallel
pub async fn execute_tool_calls(
    tool_manager: McpManagerClient,
    tool_calls: Vec<ToolCall>,
) -> Vec<ToolResult> {
    let mut tool_results = Vec::new();

    // Execute all tool calls (sequentially for now, could be parallel)
    for tool_call in tool_calls {
        let result = execute_tool_call(
            tool_manager.clone(),
            &tool_call.name,
            &tool_call.id,
            tool_call.arguments.clone(),
        )
        .await;
        tool_results.push(result);
    }

    tool_results
}

/// Parse tool arguments from JSON string to Map
pub fn parse_tool_arguments(arguments: &str) -> Result<Map<String, Value>, String> {
    match serde_json::from_str::<Value>(arguments) {
        Ok(Value::Object(args)) => Ok(args),
        Ok(_) => Err("Arguments must be a JSON object".to_string()),
        Err(e) => Err(format!("Failed to parse arguments: {}", e)),
    }
}

/// Create a formatted summary of tool output for display
pub fn create_tool_output_summary(_tool_name: &str, content: &str) -> String {
    // Try to parse as JSON first for better formatting
    if let Ok(json_value) = serde_json::from_str::<Value>(content) {
        // If it's an object with specific fields, format them nicely
        if let Value::Object(obj) = json_value {
            if let Some(Value::String(summary)) = obj.get("summary") {
                return summary.clone();
            }
            // Otherwise return a truncated pretty print
            if let Ok(pretty) = serde_json::to_string_pretty(&obj) {
                if pretty.len() > 500 {
                    return format!("{}...", &pretty[..500]);
                }
                return pretty;
            }
        }
    }

    // For non-JSON or simple text, truncate if too long
    if content.len() > 500 {
        format!("{}...", &content[..500])
    } else {
        content.to_string()
    }
}
