# ADR-0006: QMD as the Memory Recall Engine

**Status:** Accepted  
**Date:** 2026-02-20  
**Authors:** Rem, Kani, Gerald  
**Deciders:** Rem, Kani, Gerald  

---

## Context

The Superagents PRD v2 (§4.1) specifies hybrid memory recall — combining full-text (BM25) and semantic (vector) search over the agent's memory store. The open question was whether to build this in Rust using native libraries (tantivy + usearch/lance) or adopt an existing tool.

QMD (Query Markdown Documents) was raised as a candidate. It provides:
- BM25 full-text search
- Vector semantic search (gemma-300M embeddings)
- Qwen3 reranking
- Query expansion
- CLI and MCP server interfaces
- Fully local, no external API dependencies

QMD was not evaluated in the original 9-framework audit (Mastra, CrewAI, Goose, AgentStack, nanobot, OpenHands, Pydantic AI, pi-agent-core, pi-mom) as it is a search tool, not an agent framework.

---

## Decision

**Use QMD as the recall engine for the memory layer**, invoked via CLI (shelling out from Rust) or MCP server mode for persistent connections.

QMD handles the recall path. SQLite remains the canonical write-path store for typed structured memory (Decision, Fact, Identity, Preference, etc.) and runtime agent state.

---

## Rationale

### Why QMD wins over building in Rust

Implementing hybrid search with BM25 + vector + reranking + query expansion in Rust (tantivy + usearch) is several weeks of integration work. QMD already does all of this, is battle-tested in our daily OpenClaw setup, and produces meaningfully better recall quality than any single-modality approach.

The reranker is the critical differentiator: BM25 alone misses semantic matches; vector alone drowns in noise. The Qwen3 reranker combination is what makes recall agent-suitable. Replicating that quality in Rust from scratch is non-trivial and not core to our differentiation.

### Why the Node dependency is acceptable

Superagents targets developers. Node.js is already present on developer machines. On Ubuntu/Linux, `apt install nodejs` is a one-line setup step — not a meaningful ops burden relative to the engineering cost saved.

The "single binary, zero dependencies" ideal is worth revisiting at distribution time if superagents moves to non-developer targets. At that point, the pure-Rust path remains available. For now, pragmatism over purity.

---

## Alternatives Considered

| Option | Verdict |
|---|---|
| **tantivy + usearch (Rust-native)** | Rejected for now — weeks of integration, reranking non-trivial to replicate |
| **Pure vector search (usearch/lance)** | Rejected — BM25 + vector combination is materially better for agent recall |
| **Grep / SQLite FTS5** | Rejected — no semantic matching, no reranking |
| **QMD via MCP server** | Deferred — preferred long-term if persistent connection needed; CLI sufficient for Phase 1 |

---

## Consequences

### What changes
- QMD handles all memory recall queries in the agent runtime
- Canonical memory exports (repo-synced markdown files) become the QMD-indexed corpus
- Setup docs must include QMD installation steps

### Dependencies introduced
- Node.js required on host (runtime prerequisite, not embedded in binary)
- QMD installed globally (`npm install -g qmd` or equivalent)
- ~2GB GGUF model download on first query (gemma-300M embeddings, Qwen3 reranker, query expansion model) — factor into setup/onboarding docs

### Known unknowns (document, don't block)
- **Scaling:** QMD is sub-second on ~50 markdown files. Behaviour at 10K+ documents is untested. Revisit if corpus grows significantly.
- **MCP server mode:** Could replace CLI shell-outs for lower overhead persistent recall. Evaluate in Phase 2.
- **SQLite direct indexing:** QMD indexes markdown files, not SQLite rows. The export-to-markdown step (canonical memory sync) is the bridge. If sync lag becomes a problem, investigate direct SQLite recall alternatives.

### Operational notes (from live usage, Feb 2026)
- First query triggers ~2GB model downloads (one-time)
- Sub-second query latency on current corpus (~50 files)
- Index rebuilds automatically on file changes — no manual reindex needed
- MCP server mode available if CLI-per-query overhead becomes a concern
- No issues observed in daily use since Feb 19 deployment

---

## Review

This ADR supersedes the open question in PRD v2 §4.1 ("Vector store choice? Benchmark Rust-native options"). That benchmark is deferred unless QMD proves insufficient at scale.

ADR-0005 (Export Schema Versioning) remains a prerequisite — canonical exports must have versioned schemas before QMD indexes them.
