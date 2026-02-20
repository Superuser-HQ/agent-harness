/// Channel abstraction — ADR-0003 (pending Rem's draft)
///
/// All messaging surfaces (Slack, Telegram, Signal, Discord) implement
/// the Surface trait. The agent loop routes messages through this abstraction
/// without knowing which surface is active.
///
/// v1: One primary surface. Second surface is Phase 3, behind reliability gate.
use anyhow::Result;
use async_trait::async_trait;

/// An inbound message from any surface
#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub text: String,
    pub thread_id: Option<String>,
}

/// An outbound message to a surface
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub channel_id: String,
    pub text: String,
    pub thread_id: Option<String>,
}

/// Messaging surface interface — implement per provider
#[async_trait]
pub trait Surface: Send + Sync {
    /// Surface name for logging/routing
    fn name(&self) -> &str;

    /// Send a message to the surface
    async fn send(&self, msg: OutboundMessage) -> Result<()>;

    /// Poll for inbound messages (for surfaces without webhook support)
    async fn poll(&self) -> Result<Vec<InboundMessage>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_message_fields() {
        let msg = InboundMessage {
            id: "msg-001".to_string(),
            channel_id: "C0ADS4K6014".to_string(),
            sender_id: "U09KR7W14N4".to_string(),
            text: "hello".to_string(),
            thread_id: None,
        };
        assert_eq!(msg.channel_id, "C0ADS4K6014");
        assert!(msg.thread_id.is_none());
    }

    #[test]
    fn inbound_message_with_thread() {
        let msg = InboundMessage {
            id: "msg-002".to_string(),
            channel_id: "C0ADS4K6014".to_string(),
            sender_id: "U09KR7W14N4".to_string(),
            text: "threaded reply".to_string(),
            thread_id: Some("thread-001".to_string()),
        };
        assert_eq!(msg.thread_id.as_deref(), Some("thread-001"));
    }

    #[test]
    fn outbound_message_fields() {
        let msg = OutboundMessage {
            channel_id: "C0ADS4K6014".to_string(),
            text: "Consider it done.".to_string(),
            thread_id: Some("thread-001".to_string()),
        };
        assert_eq!(msg.text, "Consider it done.");
        assert!(msg.thread_id.is_some());
    }
}
