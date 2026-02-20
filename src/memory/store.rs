use anyhow::Result;

use super::record::{MemoryRecord, MemoryType};

/// Recall query — supports hybrid search (vector + FTS) over LanceDB
pub struct RecallQuery {
    pub text: String,
    pub limit: usize,
    pub filter_type: Option<MemoryType>,
}

impl RecallQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: 10,
            filter_type: None,
        }
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n;
        self
    }

    pub fn filter_type(mut self, t: MemoryType) -> Self {
        self.filter_type = Some(t);
        self
    }
}

/// Memory store interface — backed by LanceDB (ADR-0003)
///
/// Implementation is in `store_impl` (not yet wired — Phase 1).
/// This trait defines the contract that the agent loop depends on.
pub struct MemoryStore {
    // TODO: Phase 1 — hold lancedb::Connection
    _db_path: String,
}

impl MemoryStore {
    /// Open (or create) the LanceDB store at the given path
    pub async fn open(db_path: impl Into<String>) -> Result<Self> {
        let path = db_path.into();
        tracing::info!("Opening memory store at {}", path);
        // TODO: Phase 1 — lancedb::connect(&path).execute().await?
        Ok(Self { _db_path: path })
    }

    /// Write a memory record.
    /// Embedding is generated before write if not already set.
    pub async fn write(&self, _record: MemoryRecord) -> Result<()> {
        // TODO: Phase 1
        todo!("MemoryStore::write not yet implemented")
    }

    /// Hybrid recall: vector search + FTS over stored records
    pub async fn recall(&self, _query: RecallQuery) -> Result<Vec<MemoryRecord>> {
        // TODO: Phase 1
        todo!("MemoryStore::recall not yet implemented")
    }

    /// Export canonical records to markdown (ADR-0002 pipeline)
    /// Called on schedule by Cortex and on explicit checkpoint events
    pub async fn export_canonical(&self, _output_dir: &str) -> Result<()> {
        // TODO: Phase 2 — export schema versioned per ADR-0002
        todo!("MemoryStore::export_canonical not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::record::MemoryType;

    #[test]
    fn recall_query_defaults() {
        let q = RecallQuery::new("LanceDB decision");
        assert_eq!(q.text, "LanceDB decision");
        assert_eq!(q.limit, 10);
        assert!(q.filter_type.is_none());
    }

    #[test]
    fn recall_query_builder() {
        let q = RecallQuery::new("runtime choice")
            .limit(5)
            .filter_type(MemoryType::Decision);
        assert_eq!(q.limit, 5);
        assert_eq!(q.filter_type, Some(MemoryType::Decision));
    }

    #[test]
    fn recall_query_limit_zero_allowed() {
        // limit(0) is valid — callers control this
        let q = RecallQuery::new("test").limit(0);
        assert_eq!(q.limit, 0);
    }
}
