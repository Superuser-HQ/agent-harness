# QMD Research Spike: Memory Retrieval Layer

**Date:** 2026-02-20
**Author:** Kani
**Triggered by:** Gerald's question in #ai-collab
**Status:** Research — not a decision

---

## What is QMD?

[QMD (Query Markup Documents)](https://github.com/tobi/qmd) is a local hybrid search engine for documents and markdown files. Built by Tobi (ex-Shopify), written in TypeScript/Bun, runs fully on-device via node-llama-cpp with GGUF models.

**Core capabilities:**
- BM25 full-text search (SQLite FTS5)
- Vector semantic search (embeddings via node-llama-cpp)
- LLM reranking + query expansion (`qmd query` — best quality, slower)
- CLI-first with `--json` and `--files` output for agent workflows
- MCP server exposing `qmd_search`, `qmd_vector_search`, `qmd_deep_search`, `qmd_get`, `qmd_multi_get`, `qmd_status`
- Collection scoping (search within a named group of files)
- Context annotations — attach metadata to collections that surfaces alongside results

**Quick example:**
```sh
qmd collection add ~/agent-memory --name memory
qmd embed
qmd query "what did we decide about session model" --json -n 5
```

---

## Relevance to Superagents Memory Architecture

Our planned memory layer (from ADR decisions and v1 PRD) has two components:

| Component | Purpose |
|-----------|---------|
| **SQLite runtime store** | Typed agent state — Decisions, Facts, Identity, Preferences. High-frequency writes during task execution. |
| **Repo export pipeline** | Canonical, human-readable, version-controlled record. Exported on schedule + checkpoint events. |

QMD is **not** a replacement for either — but it has a potential role as the **search/recall layer over exported artifacts**.

---

## Where QMD Could Fit

### Option A: Search layer over canonical repo exports

The repo export pipeline produces versioned markdown files (decisions, facts, identity). QMD could index these and serve as the agent's recall interface:

```
SQLite (write) → export pipeline → git repo (markdown) → QMD index (search)
Agent reads:  qmd query "deployment decision june" → returns relevant memory files
```

**Pros:**
- We don't build our own hybrid search stack
- Human-readable exports + searchable — solves the "memory you can't audit" concern Rem raised
- MCP server = clean interface, model-agnostic
- Fully local, no external API dependency

**Cons:**
- Adds a Node/Bun runtime dependency to a Rust project
- Search freshness depends on export pipeline lag (our SLO is ≤10 min p95 — acceptable)
- Another process to supervise

### Option B: Replace custom vector index in SQLite runtime store

Instead of building our own vector index (lance, qdrant-client, usearch), use QMD to handle the semantic search side of runtime memory.

**Verdict: Not recommended.** QMD is file-centric, not row-centric. It can't query against typed structured records in SQLite — it reads markdown files. Forcing it here adds impedance mismatch.

### Option C: Compound capture store search

The compound capture workflow writes task solutions to `docs/solutions/`. Engineers and agents searching past solutions could use QMD as the interface.

```sh
qmd collection add docs/solutions --name solutions
qmd query "how did we solve stuck worker detection"
```

**Verdict: High value, low cost.** This is a natural fit with minimal integration work.

---

## Tradeoffs vs Building Our Own

| Dimension | QMD | Roll our own (lance/qdrant-client) |
|-----------|-----|-------------------------------------|
| Time to working hybrid search | Hours | Days–weeks |
| Rust integration | FFI/subprocess | Native crate |
| Customisation | Limited | Full |
| Maintenance burden | Upstream | Ours |
| Runtime deps | Node/Bun + GGUF models | Rust only |
| MCP server | Built-in | Build it |
| Reranking | Built-in (LLM) | Build it |

For Phase 0, using QMD for retrieval over canonical exports buys us the hybrid search story without blocking the Rust core work.

---

## What We'd Need to Validate

1. **Performance on agent-scale collections** — QMD is designed for personal notes. How does it handle 10K+ exported memory artifacts at write speed?
2. **Subprocess vs MCP** — Calling QMD as subprocess from Rust agent is straightforward. MCP integration is cleaner but adds protocol overhead.
3. **Model pull time** — First-run embedding model download could be 1-4GB. Need to handle cold start gracefully.
4. **Export freshness** — Does search freshness within our ≤10 min SLO window matter for agent recall? (Probably yes for recent decisions, no for long-term facts.)

---

## Recommendation

**Phase 0:** No action — focus on core loop. Note QMD as candidate.

**Phase 1 (Week 2-3):** Spike QMD for compound capture store search (`docs/solutions/`). Low risk, high value, isolated from core memory path. Estimate: 1 day.

**Phase 2:** Evaluate QMD as recall layer over repo exports. Depends on whether the export pipeline search experience is good enough with raw `grep`/`git log` first.

**Decision gate:** Don't commit to QMD until we've seen the export pipeline in production. Might be overkill. Might be exactly right.

---

## References

- [QMD GitHub](https://github.com/tobi/qmd)
- [OpenClaw + QMD memory post](https://www.josecasanova.com/blog/openclaw-qmd-memory) — someone already doing this with OpenClaw
- ADR-0002 (memory model — SQLite + vector)
- ADR-0005 (export schema versioning)
- v1 PRD §4.5 (Memory), §4.7 (Repo Export Pipeline)
