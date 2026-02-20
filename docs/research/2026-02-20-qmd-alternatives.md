# QMD Alternatives: Hybrid Document/Memory Search

**Date:** 2026-02-20  
**Author:** Kani  
**Context:** Evaluating fallback options for the fast hybrid recall layer in the Superagents memory architecture.  
**Cross-reference:** QMD is the current recommended choice per ADR-0006 (`adr/0006-qmd-recall-engine`). This file documents alternatives in case QMD doesn't work out (install friction, Node.js dependency, model size, or maintenance risk).

---

## Background

The Superagents memory architecture has QMD filling a specific role:

```
SQLite (write-path) → export pipeline → flat markdown files → QMD index (search)
Agent recall: qmd query "what did we decide about X" → top-5 docs + reranked
```

Requirements for any alternative in this slot:
- **Hybrid search**: BM25 (lexical) + vector (semantic) combined
- **Local / offline**: no external API dependencies
- **Callable from Rust**: shell-out or MCP or native crate
- **Indexes flat markdown files**: our canonical export format
- **Fast**: sub-second on small corpora (~hundreds of files)
- **Reranking (bonus)**: cross-encoder or LLM reranking improves result quality

---

## Comparison Table

| Tool | Runtime | Approach | Local | Interface | Reranking | Verdict |
|------|---------|----------|-------|-----------|-----------|---------|
| **QMD** ⭐ | Node.js CLI | BM25 + vector + reranking + query expansion | ✅ | CLI + MCP server | ✅ Qwen3 | ✅ **Recommended (ADR-0006)** — full hybrid stack, battle-tested |
| **MiniSearch** | Node.js (in-memory) | BM25 only | ✅ | JS API only | ❌ | ❌ Too limited — no vector, no reranking |
| **Meilisearch** | Rust binary (server) | BM25 + vector hybrid | ✅ | REST API | ❌ | ⚠️ Viable but heavyweight (server process required) |
| **Typesense** | C++ binary (server) | BM25 + semantic hybrid | ✅ | REST API | ❌ | ⚠️ Viable, similar profile to Meilisearch |
| **tantivy** | Rust library | BM25 only | ✅ | Rust crate / CLI (limited) | ❌ | ⚠️ Best Rust-native BM25, but no vector out of box |
| **LanceDB** | Rust/Python/JS library | BM25 + vector hybrid | ✅ | Native crate + REST | ❌ | ✅ Best overall alternative — embeddable, no server, hybrid |

---

## Per-Tool Analysis

### MiniSearch

MiniSearch (lucaong/minisearch) is a tiny, zero-dependency JavaScript library for in-memory full-text search using BM25. It runs in Node or the browser, has no network requirements, and indexes and queries purely in RAM. Strengths: extremely lightweight (~20KB minified), simple API, fast on small datasets. Weaknesses: BM25 only — no vector embeddings, no semantic recall, no reranking; must reload index from serialized JSON on each process start. **Fit for our use case: poor.** We need hybrid BM25 + vector; MiniSearch only covers the lexical half and would require building the entire embedding layer from scratch.

---

### Meilisearch

Meilisearch is a Rust-based open-source search engine with a REST API. It supports hybrid search: BM25 full-text combined with vector similarity (embeddings can be generated via external embedders or provided pre-computed). Self-hosted as a single binary or Docker container. Strengths: mature project (~50K GitHub stars), fast, well-documented, clean REST API that any language can call via HTTP, supports filtering and facets, actively maintained. Weaknesses: **requires a running server process** — not embeddable, adds operational complexity; no built-in cross-encoder reranking (Meilisearch AI does relevance tuning but not reranking); embeddings require configuring an external source (Ollama, HF, or pre-computed). **Fit: moderate.** It could serve the QMD role but requires managing a sidecar server process, which QMD avoids by being a pure CLI. Would need to build an indexing script to ingest markdown exports.

---

### Typesense

Typesense is a C++ search engine (open-source, MIT) with typo tolerance, BM25-based full-text, and hybrid semantic search via built-in ML embedding models. Self-hosted as a binary or Docker. It supports on-device embedding generation using built-in models (no external API needed). Strengths: typo tolerance is genuinely useful for agent queries, built-in semantic embedding removes one integration step vs. Meilisearch, good REST API. Weaknesses: **same architectural issue as Meilisearch** — requires a server process; no cross-encoder reranking; the built-in embedding models are less tunable than QMD's configurable GGUF models; less Rust-ecosystem native than tantivy or LanceDB. **Fit: moderate.** Similar trade-offs to Meilisearch. Slightly better embedding story out of the box, but same operational overhead.

---

### tantivy

tantivy is a Rust library (crate) implementing a Lucene-inspired full-text search engine with BM25 scoring. It is the underlying engine for several production search tools (Quickwit, etc.) and has a mature Rust API. There is a `tantivy-cli` but it is demo-quality (limited features, basic JSON API server). Strengths: **native Rust crate** — directly embeddable into Superagents runtime with no external process, extremely fast BM25, field boosting, phrase queries, WASM-compatible; zero external dependencies. Weaknesses: **BM25 only — no vector embeddings built-in**; would need to integrate a separate vector store (e.g., usearch crate) and merge/RRF results manually; tantivy-cli too limited for production use. **Fit: partial.** The best option if we want native Rust integration without shelling out, but it covers only the lexical half of hybrid search. Would require significant custom work to reach QMD parity.

---

### LanceDB

LanceDB is an open-source embedded vector database built on the Lance columnar format (Rust/Arrow). It supports **vector similarity search, BM25 full-text search, and SQL queries** in a single library. Available as a Rust crate (`lancedb`), Python SDK, and JavaScript SDK. No separate server — fully embedded, zero infrastructure overhead. Strengths: **true hybrid search combining FTS + vector in one library**; native Rust crate for direct embedding in the runtime; columnar format is efficient for metadata filtering; actively developed by a well-funded team; supports HNSW and IVF_PQ indexing; handles markdown/text file ingestion via the SDK. Weaknesses: **no built-in cross-encoder reranking** (would need to add a separate reranker step); less battle-tested as a CLI tool than QMD; no MCP server out of the box; API is more programmatic than QMD's markdown-native CLI workflow. **Fit: good.** The strongest alternative to QMD — embeddable, no server, hybrid search, Rust-native. Missing only the reranking layer that QMD provides via Qwen3.

---

## Recommendation

**If QMD doesn't work out, use LanceDB.**

LanceDB is the closest architectural match to QMD's role: embeddable, no server process, hybrid BM25 + vector, and directly callable from Rust as a native crate. The missing piece is cross-encoder reranking — QMD's Qwen3 reranker is a meaningful quality advantage — but this could be addressed by adding a separate reranker step (e.g., via a lightweight cross-encoder model called via candle or shell-out to a Python script).

Meilisearch or Typesense are viable if we're willing to run a search sidecar process, but that adds operational complexity that QMD specifically avoids.

tantivy alone is insufficient (lexical only) but could be a component if we're building a custom hybrid stack in Rust.

MiniSearch is not a candidate for this role.

---

## Architecture Impact

None of the alternatives support QMD's **query expansion** feature (semantic expansion of the query before search). This is a meaningful gap — if query expansion is important for recall quality, any replacement would need to replicate it or accept a quality regression.

LanceDB's FTS is powered by tantivy internally, so the BM25 quality is the same — only the integration layer differs.

---

## Notes

- QMD's ~2GB model download (gemma-300M embeddings + Qwen3 reranker) is the primary friction point. LanceDB uses whatever embedding model you configure — could be smaller.
- QMD's MCP server is a genuine advantage for agent integration. LanceDB would need a custom MCP wrapper.
- Neither Meilisearch nor Typesense are "embeddable" in the Rust binary sense — both require running as separate processes.
