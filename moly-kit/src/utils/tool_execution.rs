use serde_json::Value;

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
