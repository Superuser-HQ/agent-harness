# ADR-0006: Memory Store and Recall Engine

**Status:** Accepted  
**Date:** 2026-02-20  
**Authors:** Rem, Kani, Gerald, Yao  
**Deciders:** Yao, Rem, Kani, Gerald  
**Supersedes:** ADR-0006 draft (QMD as recall engine)

---

## Context

The Superagents PRD v2 (§4.1) specifies hybrid memory recall — combining full-text (BM25) and semantic (vector) search over the agent's memory store. The architecture must support:

- Multi-agent concurrent reads and writes
- Single source of truth (no split-brain)
- Sub-second agent recall during task execution
- Low memory footprint — embedded, no server process
- Embeddable in the Rust runtime

An initial evaluation of QMD (a hybrid BM25 + vector + Qwen3 reranker tool) established strong recall quality. However, further design discussion identified a structural constraint: QMD operates over files and requires an async export step from SQLite — creating two systems with a sync gap. Under concurrent multi-agent write load, this introduces eventual consistency windows and split-brain risk.

---

## Decision

**Use LanceDB as the single memory store and recall engine.**

LanceDB (Rust-native, embedded) handles both the write path and recall in a single system:
- Typed structured records (Decision, Fact, Identity, Preference) stored as columnar data
- Vector index built natively over the same store
- Full-text search (BM25-equivalent) over the same store
- No export step, no sync, no split-brain
- Embedded via Rust crate — no server process

SQLite is **not** used for the write path. LanceDB replaces it.

---

## Rationale

### The split-brain problem

The initial design (SQLite write path → async markdown export → QMD search index) requires keeping two systems in sync. Under multi-agent concurrent writes, any async sync creates an eventual consistency window:

- Agent A writes a Decision to SQLite at T=0
- Agent B queries for that Decision at T=1
- QMD hasn't indexed the export yet — query returns stale or empty result

This isn't a tuning problem. It's structural. The only solution is a single system where the storage IS the index.

### Why LanceDB

LanceDB is the only embedded option that satisfies all constraints simultaneously:

| Constraint | LanceDB |
|---|---|
| Single system (write + recall) | ✅ |
| No server process | ✅ |
| Concurrent reads | ✅ |
| Low memory footprint | ✅ |
| Rust-native crate | ✅ |
| Vector search | ✅ |
| Full-text search | ✅ (hybrid, no separate index) |
| ACID-style writes | ✅ (Lance columnar format) |

### What we give up

- **Qwen3 reranker** — QMD's cross-encoder reranker is a meaningful recall quality advantage. LanceDB has no built-in reranker. Recall quality will be slightly lower on ambiguous queries in Phase 0.
- **Query expansion** — QMD expands queries automatically; LanceDB does not.

Both are acceptable Phase 0 tradeoffs. A reranking post-processing step can be added later without changing the storage architecture. The architecture decision is harder to change than the search quality tuning.

---

## QMD: Validated Prior Art

QMD was thoroughly evaluated as part of this ADR process (see `research/2026-02-20-qmd-analysis.md`). Its hybrid recall architecture (BM25 + vector + Qwen3 reranker + query expansion) is the target quality bar for future recall improvements.

**QMD remains the right tool for single-agent setups** (e.g., OpenClaw, where there is one writer and no sync problem). It is not suitable as the recall engine for a multi-agent framework because of the two-system sync constraint.

The reranker pattern from QMD is worth implementing as a post-processing step in a future iteration — once LanceDB handles the storage, a separate reranker pass can improve recall quality without architectural changes.

---

## Alternatives Considered

| Option | Verdict |
|---|---|
| **SQLite + QMD** | ❌ Two-system sync — split-brain risk under multi-agent concurrent writes |
| **SQLite FTS5 only** | ❌ No semantic matching — insufficient recall quality |
| **Postgres + pgvector** | ❌ Requires server process — breaks embedded/low-footprint constraint |
| **QMD as sole engine** | ❌ File-based, requires async export from write store — sync problem |
| **tantivy + usearch (Rust)** | ❌ Two separate libraries, weeks of integration, reranking non-trivial |
| **LanceDB** | ✅ Single system, embedded, handles write + recall, Rust-native |

---

## Consequences

### What changes
- LanceDB is the runtime memory store AND the search/recall layer
- No SQLite runtime database
- No async markdown export for search indexing
- Typed memory records (Decision, Fact, Identity, Preference) stored as LanceDB tables
- Recall = vector search + FTS over the same LanceDB store (one hop)

### Dependencies introduced
- `lancedb` Rust crate (embedded — no host installation required)
- No Node.js or Python runtime dependency for memory

### Multi-agent concurrency
LanceDB's Lance columnar format handles concurrent reads well. Writes use optimistic concurrency — suitable for the agent write patterns in scope (infrequent structured writes, high-frequency reads). Revisit if write contention proves problematic in load testing.

### Recall quality
Phase 0: LanceDB hybrid search (vector + FTS) without reranking. Acceptable for initial agent use cases.

Future: Add a reranking post-processing step (cross-encoder model) once the storage architecture is stable. QMD's Qwen3 reranker architecture is the reference implementation.

### Canonical export (git-synced memory)
LanceDB is the runtime store. The canonical export pipeline (ADR-0005) still applies — periodic export to versioned markdown for human readability, git history, and disaster recovery. This export is NOT used for search (LanceDB handles that). It is used for auditability and backup only.

---

## Review

This ADR supersedes the open question in PRD v2 §4.1 ("Vector store choice?"). The benchmark is closed.

ADR-0005 (Export Schema Versioning) remains a prerequisite — canonical exports must have versioned schemas.

Research that informed this decision is preserved in PR #2 (`research/qmd-memory-search`):
- `research/2026-02-20-qmd-analysis.md`
- `research/2026-02-20-cognee-analysis.md`
- `research/2026-02-20-qmd-alternatives.md`
- `research/2026-02-20-cognee-alternatives.md`
