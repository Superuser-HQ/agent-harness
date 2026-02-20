/// Memory store and recall using LanceDB (ADR-0006)
///
/// LanceDB is the single system for both write path and search/recall:
/// - No separate SQLite + vector index split
/// - No async sync gap (no split-brain under multi-agent concurrent writes)
/// - Embedded Rust crate — no server process
///
/// Canonical memory types (v1 — 4 types, defer rest to v2):
///   Decision | Fact | Identity | Preference
///
/// Canonical export pipeline (ADR-0005) runs on schedule and writes
/// versioned markdown to the repo. That export is for auditability + backup
/// ONLY — LanceDB handles all runtime search/recall.

pub mod record;
pub mod store;

pub use record::{MemoryRecord, MemoryType};
pub use store::MemoryStore;
