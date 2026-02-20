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
