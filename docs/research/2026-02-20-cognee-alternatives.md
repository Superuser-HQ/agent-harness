# Cognee Alternatives: Knowledge Graph / Relational Memory

**Date:** 2026-02-20  
**Author:** Kani  
**Context:** Evaluating fallback options for the relational memory / knowledge graph layer in the Superagents memory architecture.  
**Cross-reference:** Cognee is the current recommended choice for this role (pending a hands-on spike). See `2026-02-20-cognee-analysis.md` for the full evaluation.

---

## Background

The Superagents memory architecture gives Cognee the following role:

```
SQLite (write-path: Decision, Fact, Identity, Preference)
    ↓
Cognee ingestion pipeline
    ↓
Knowledge graph (entity triplets: subject → relation → object)
    ↓
Graph + vector search for cross-session relational reasoning
```

This is distinct from QMD's role (fast lexical+semantic recall over flat files). Cognee's job is **constructing and traversing structured relationships** — "who decided what, and why does that connect to this other decision?" — across sessions.

Requirements for any alternative in this slot:
- **Knowledge graph construction**: extract entities and relationships from structured or semi-structured data
- **Local / self-hosted**: no mandatory external API (Ollama support a strong plus)
- **Graph + vector hybrid retrieval**: not just flat vector search
- **Agent-friendly interface**: Python SDK, CLI, or REST API
- **Incremental updates**: agents write continuously, not in batch

---

## Comparison Table

| Tool | Runtime | Graph Type | Local/Hosted | LLM Required | Temporal Memory | Verdict |
|------|---------|------------|--------------|--------------|-----------------|---------|
| **Cognee** ⭐ | Python | Entity KG (graph DB + vector) | ✅ Self-hosted | ✅ (Ollama OK) | ⚠️ Limited | ✅ **Recommended (pending spike)** — broad ingestion, modular ECL pipeline |
| **Graphiti** | Python | Temporal KG (Neo4j / FalkorDB) | ✅ Self-hosted | ✅ (Ollama OK) | ✅ Strong | ✅ Best alternative — may outperform Cognee for structured agent writes |
| **Mem0** | Python / JS | Directed graph (Neo4j optional) | ✅ Self-hosted | ✅ (Ollama OK) | ⚠️ Limited | ⚠️ Memory management layer, not raw KG |
| **Zep** | Python | Temporal KG (via Graphiti) | ⚠️ Mostly cloud | ✅ (Ollama OK) | ✅ Strong | ⚠️ Graphiti is the OSS primitive; Zep = managed |
| **LightRAG** | Python | Entity-relation graph (KG + vector) | ✅ Self-hosted | ✅ (Ollama OK) | ❌ No | ⚠️ Strong for document KG, weak as agent memory |
| **Microsoft GraphRAG** | Python | Community-clustered KG | ✅ Self-hosted | ✅ Required | ❌ No | ❌ Not suitable — batch-only, expensive, no incremental |

---

## Per-Tool Analysis

### Graphiti

Graphiti (getzep/graphiti) is an open-source Python framework for building and querying **temporally-aware knowledge graphs** designed specifically for AI agents. It supports incremental real-time data ingestion — continuously integrating new episodes (messages, events, structured data) by extracting entities and relationships and resolving them against the existing graph. Backed by Neo4j or FalkorDB (in-memory, no external service needed), with semantic, keyword, and graph-based hybrid search. An MCP server is available for direct agent integration. Strengths: **best temporal semantics of any option** (relationships have timestamps and validity periods, supports historical queries without full recomputation); MCP server available; Ollama supported; research-backed (ArXiv paper, SOTA benchmark results on agent memory); actively maintained by the Zep team. Weaknesses: Neo4j dependency is significant operational overhead unless using FalkorDB in-memory mode; entity resolution requires LLM calls on every write; Python-only. **Fit: excellent.** This is the closest open-source equivalent to Cognee's relational memory role, with arguably stronger temporal reasoning.

---

### Mem0

Mem0 (mem0ai/mem0) is a Python/JS "universal memory layer" for AI agents. It stores memories as a hybrid of vector search, graph relationships, and key-value data. v1.0.0 released recently. Graph memory mode uses Neo4j (optional) to extract entities and directed relationships from memory writes. Multi-level memory scoping (user, session, agent). Strengths: **extremely popular** (~30K+ GitHub stars, YC-backed), easy single-call API (`memory.add`, `memory.search`), 26% accuracy improvement over OpenAI Memory on LOCOMO benchmark, supports Ollama, cross-platform SDKs. Weaknesses: **designed for preference/personalization memory** rather than structured decision/fact graphs; graph layer is optional and less sophisticated than Graphiti or Cognee; temporal reasoning is limited (no explicit timestamp-based relationship validity); the default operation still leans on vector search with graph as an augment rather than a first-class KG. **Fit: partial.** Mem0 is a strong choice if the primary need is agent personalization and session continuity. It's less appropriate as a structured relational reasoning layer — it would work but would lose the graph-traversal depth that Cognee and Graphiti provide.

---

### Zep

Zep is positioned as an "end-to-end context engineering platform" that provides sub-200ms latency, temporal knowledge graphs, and pre-formatted context retrieval. Graphiti is its open-source engine. The key distinction: **Zep as a self-hosted system is essentially "run Graphiti yourself"** — the `getzep/zep` repository is now examples and integrations for Zep Cloud, not a self-hostable server. Zep Cloud is a managed service (SOC2, HIPAA). Strengths: production-grade managed option if self-hosting is ever too much maintenance burden; best-in-class latency claims; strong Python/JS/Go SDKs. Weaknesses: the "open-source" story is now Graphiti (raw framework) vs. Zep (managed platform) — **there is no currently maintained open-source Zep server to deploy**; for our use case Graphiti directly is the better choice. **Fit: deferred.** Use Graphiti for self-hosted; Zep Cloud for a future managed upgrade if needed.

---

### LightRAG

LightRAG (HKUDS/LightRAG, EMNLP 2025) is a graph + vector RAG framework that builds knowledge graphs from documents using LLMs, then supports hybrid querying across the graph and vector index. Multiple storage backends: SQLite (local), PostgreSQL, MongoDB, Neo4j. Reranker support added August 2025. Actively developed with multimodal support via RAG-Anything. Strengths: **multiple storage backends including SQLite** (no Neo4j required for basic use); reranker support; very active development; Ollama supported; extracts both entities and relationships. Weaknesses: **batch-oriented** — designed for document ingestion and querying, not incremental agent memory writes; **no temporal semantics** (relationships have no timestamps or validity periods); the graph is static after indexing a document corpus; not designed for the "continuous agent writes small facts" pattern. **Fit: partial.** LightRAG is a strong choice if the KG use case is "build a graph over a document corpus for querying" rather than "continuously update a graph as an agent operates." If Cognee's spike reveals problems with its graph construction quality, LightRAG could fill the document-to-graph role, but it won't cover the temporal/relational reasoning over live agent writes.

---

### Microsoft GraphRAG

GraphRAG (microsoft/graphrag) is a Python pipeline that extracts knowledge graph structures from unstructured text using LLMs, then augments RAG with community detection (Leiden algorithm) and global/local query modes. Strengths: excellent for analyzing narrative/document corpora; global query mode enables "what are the main themes?" questions that flat RAG can't answer; research-quality methodology. Weaknesses: **extremely expensive** — LLM calls for every entity extraction, community summarization, etc.; explicitly warns "indexing can be an expensive operation"; **batch-only, not incremental** — re-indexing on each update is impractical for agent memory; no temporal semantics; not designed for programmatic agent integration; deprecated/research-quality codebase status. **Fit: poor.** GraphRAG is a document analysis research tool, not an agent memory layer. Its cost and batch-only architecture disqualify it for the Cognee role.

---

## Recommendation

**If Cognee doesn't work out, use Graphiti.**

Graphiti is the most architecturally aligned alternative: temporal KG, incremental updates, MCP server for agent integration, Ollama support, and a research paper demonstrating SOTA performance on agent memory benchmarks. The main operational concern is Neo4j — but FalkorDB (in-memory, no separate service) eliminates that for single-agent or development use, and Neo4j only becomes necessary for production scale.

Mem0 is a reasonable choice if the use case is lighter — preference/personalization memory rather than structured reasoning over decisions and facts. It trades graph depth for API simplicity and a larger ecosystem.

LightRAG is worth considering if the architecture shifts toward "build a KG over the agent's document corpus" rather than "continuously accumulate structured facts into a graph." It won't serve the incremental write pattern well.

---

## Architecture Impact

One important observation: **Graphiti is the engine Zep is built on**, and Graphiti's team has published a benchmark showing SOTA performance. This suggests Graphiti may actually be superior to Cognee in the specific temporal-reasoning dimension, and Cognee may be stronger on the "30+ data sources / broad ingestion" dimension. The choice between them may come down to:

- **Cognee** if the primary input is diverse documents/files to be graphified
- **Graphiti** if the primary input is structured agent facts/decisions with temporal tracking

Given that our SQLite write-path already structures the data (typed: Decision, Fact, Identity, Preference), **Graphiti may actually be a better fit than Cognee** for the Superagents architecture — it's purpose-built for structured agent writes, not document ingestion. This is worth flagging before the Cognee spike.

---

## Notes

- Both Cognee and Graphiti require LLM calls for entity extraction. This is unavoidable for any KG construction tool that doesn't have pre-defined schemas.
- For fully offline/local operation: both support Ollama. Graphiti also explicitly supports FalkorDB (in-process graph, no Neo4j).
- None of the alternatives eliminate the LLM dependency for KG construction — if we wanted a schema-driven KG (entity types pre-defined), we'd build it ourselves on top of tantivy or LanceDB + a custom graph layer.
- Mem0 v1.0.0 introduced API modernization — some older tutorials may use the deprecated API.
