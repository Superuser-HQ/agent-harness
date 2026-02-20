use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Tool permission tier — controls which sessions can invoke which tools
///
/// Tier hierarchy (lowest to highest risk):
///   Read < Write < Network < Destructive
///
/// Sessions inherit a maximum tier from their config.
/// Branch sessions default to Read unless escalated.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolTier {
    /// Safe reads — file read, memory recall, web fetch
    Read,
    /// Local writes — file write, memory write, git
    Write,
    /// External network calls — messaging, webhooks, APIs
    Network,
    /// Destructive or irreversible actions — delete, deploy, send
    Destructive,
}

/// Input/output envelope for tool invocations
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub params: Value,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

/// Every agent capability implements this trait
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn tier(&self) -> ToolTier;
    async fn invoke(&self, call: ToolCall) -> Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_tier_ordering() {
        assert!(ToolTier::Read < ToolTier::Write);
        assert!(ToolTier::Write < ToolTier::Network);
        assert!(ToolTier::Network < ToolTier::Destructive);
    }

    #[test]
    fn tool_call_construction() {
        let call = ToolCall {
            name: "read_file".to_string(),
            params: serde_json::json!({"path": "/tmp/test"}),
        };
        assert_eq!(call.name, "read_file");
    }

    #[test]
    fn tool_result_success() {
        let result = ToolResult {
            success: true,
            output: serde_json::json!({"content": "hello"}),
            error: None,
        };
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn tool_result_failure() {
        let result = ToolResult {
            success: false,
            output: serde_json::Value::Null,
            error: Some("file not found".to_string()),
        };
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("file not found"));
    }
}
