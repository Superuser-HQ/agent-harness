use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Canonical memory types shipped in v1
/// Types beyond these 4 are deferred to v2 (per PRD §4.5 scope cut)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// An explicit choice made during an agent run
    /// (e.g. "We decided to use LanceDB for the memory store")
    Decision,
    /// A factual claim about the world or the system
    /// (e.g. "The Slack channel ID for #ai-collab is C0ADS4K6014")
    Fact,
    /// Who the agent is — persistent identity state
    /// (e.g. "My name is Kani. I work for SuperuserHQ.")
    Identity,
    /// User or system preferences that shape agent behaviour
    /// (e.g. "Yao prefers bullet lists over tables in Telegram")
    Preference,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryType::Decision => write!(f, "decision"),
            MemoryType::Fact => write!(f, "fact"),
            MemoryType::Identity => write!(f, "identity"),
            MemoryType::Preference => write!(f, "preference"),
        }
    }
}

/// A single memory record stored in LanceDB
///
/// The `embedding` field is populated by the store before write.
/// All canonical types are eligible for export via the ADR-0005 pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// Unique ID
    pub id: Uuid,
    /// Memory type — determines export class and recall priority
    pub memory_type: MemoryType,
    /// Human-readable content — what the agent will recall
    pub content: String,
    /// Optional structured metadata (JSON)
    pub metadata: Option<serde_json::Value>,
    /// Session that created this record
    pub session_id: String,
    /// When this record was written
    pub created_at: DateTime<Utc>,
    /// Embedding vector — populated by MemoryStore before write
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl MemoryRecord {
    pub fn new(
        memory_type: MemoryType,
        content: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            memory_type,
            content: content.into(),
            metadata: None,
            session_id: session_id.into(),
            created_at: Utc::now(),
            embedding: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_creation() {
        let r = MemoryRecord::new(MemoryType::Decision, "Use LanceDB", "session-1");
        assert_eq!(r.memory_type, MemoryType::Decision);
        assert_eq!(r.content, "Use LanceDB");
        assert!(r.embedding.is_none());
    }
}
