# Cognee — Research Analysis

**Date:** 2026-02-20  
**Analyst:** Kani + Rem  
**Context:** Evaluating as the knowledge graph / relational memory layer for Superagents, alongside QMD as the fast recall engine.

---

## What Is Cognee?

Cognee (github.com/topoteretes/cognee) is an open-source Python SDK that builds persistent knowledge graphs from raw data — documents, conversations, files, audio transcriptions, and 30+ other sources.

Core pipeline: **ECL — Extract → Cognify → Load**

1. `cognee.add(data)` — ingest raw data (text, files, URLs, etc.)
2. `cognee.cognify()` — extract entities and relationships as triplets (subject → relation → object), build the knowledge graph
3. `cognee.memify()` — apply memory algorithms to the graph
4. `cognee.search(query)` — query by meaning + graph traversal

The key distinction from RAG or QMD: Cognee doesn't just store and retrieve text — it extracts **structured relationships** and stores them as graph edges. "Yao founded SuperuserHQ" and "SuperuserHQ serves Series A startups" become connected nodes you can traverse, not just chunks to match against.

---

## Capabilities

| Feature | Details |
|---|---|
| **Graph construction** | Entity extraction + triplet (subject→relation→object) storage |
| **Query modes** | Vector semantic search + graph traversal combined |
| **Data sources** | 30+ (documents, PDFs, HTML, conversations, audio, images) |
| **LLM integration** | OpenAI, Ollama (fully local), other providers |
| **Storage backends** | Graph DB (Memgraph, Neo4j, Kuzu, in-memory) + vector store |
| **API surface** | Python SDK, CLI (`cognee-cli`), local UI |
| **Runtime** | Python 3.10–3.13, async |
| **Self-improvement** | Learns from feedback, updates concepts and synonyms |

**Ollama support** means fully local deployment — no API keys, no external dependencies.

---

## Performance Claims

From the Memgraph community call with Cognee's founder:
- RAG baseline recall accuracy: ~60%
- Cognee (graph + vector): ~90%
- The graph layer is what closes the gap — flat vector search misses relational context

Adoption: ~7K GitHub stars, ~200–300 projects using it based on telemetry (as of late 2025).

---

## Comparison with QMD

| Dimension | QMD | Cognee |
|---|---|---|
| **Strength** | Hybrid recall (BM25 + vector + reranking) | Knowledge graph construction + relationship reasoning |
| **Best at** | "Find me the relevant doc/fact" | "How do these concepts connect?" |
| **Query type** | Lexical + semantic over flat files | Semantic + graph traversal over entity relationships |
| **Runtime** | Node.js | Python |
| **Data model** | Indexed markdown/document files | Graph (entities + edges) + vector store |
| **Latency** | Sub-second (batch indexed) | Higher — graph build is async, search may be slower |
| **Operational overhead** | Minimal — auto-reindex on file change | Pipeline management (add → cognify → memify cycle) |

---

## Proposed Integration Architecture for Superagents

QMD and Cognee solve different layers of the memory problem and compose cleanly:

```
                    ┌─────────────────────────────┐
                    │         Agent Runtime        │
                    └──────────┬─────────┬─────────┘
                               │         │
                    Write path │         │ Recall path
                               ▼         ▼
                         ┌─────────┐   Route by query type
                         │ SQLite  │        │
                         │(runtime)│        ├── Simple fact/doc lookup
                         └────┬────┘        │         ▼
                              │             │       ┌─────┐
                    Canonical │             │       │ QMD │  (fast, BM25+vector+reranker)
                    export sync│            │       └─────┘
                              ▼             │
                    ┌──────────────────┐    └── Relational/contextual query
                    │ Versioned        │                  ▼
                    │ Markdown Exports │            ┌─────────┐
                    │ (ADR-0002 schema)│            │ Cognee  │  (graph traversal + semantic)
                    └────┬────────────┘            └─────────┘
                         │              │
                         └──── QMD ─────┘ (indexes exports for fast recall)
                         │
                         └──── Cognee (ingests exports, builds/updates knowledge graph)
```

**Write path:** Agent → SQLite (typed, structured, high-frequency writes)

**Canonical sync:** SQLite exports to versioned markdown files (per ADR-0002)

**Fast recall (QMD):** Indexes the markdown exports. Sub-second BM25 + vector + reranker. Handles: "What did we decide about X?", "Find the relevant session from last week."

**Relational memory (Cognee):** Ingests the same exports periodically (or on significant events). Builds the knowledge graph. Handles: "How does project X relate to client Y?", "What are all the decisions that affect the identity memory for agent Z?", "What patterns exist across sessions?"

**Query routing:** Simple fact/doc lookup → QMD. Relational/contextual/cross-session → Cognee.

---

## Fit for Superagents PRD

| PRD Requirement (§4.1) | Covered by |
|---|---|
| Vector + full-text hybrid recall | QMD ✓ |
| Graph edges between Decision/Identity/Fact records | Cognee ✓ |
| Sub-second agent recall during task execution | QMD ✓ |
| Cross-session relational reasoning | Cognee ✓ |
| Canonical export as source of truth | SQLite → markdown (ADR-0002) → feeds both |

Cognee could replace the need to build a custom graph layer in Rust — which was an open question in the PRD.

---

## Concerns & Open Questions

1. **Python dependency** — adds Python to the host requirements alongside Node (QMD). On developer machines and Ubuntu, this is trivial. Same pragmatic argument as Node.

2. **Graph build latency** — `cognify` is not synchronous with agent writes. This is acceptable if Cognee is the *long-term* memory layer (async, eventually consistent) while QMD handles real-time recall.

3. **Scale unknowns** — like QMD, untested at large corpus sizes (thousands of agent memory exports). Needs a spike.

4. **Graph quality for structured agent data** — Cognee is optimized for documents and conversations. How well does entity extraction work on structured markdown memory exports (typed Decision/Fact/Identity records)? Needs validation.

5. **Double-ingestion cost** — same exports feed both QMD and Cognee. Not a problem architecturally, but worth managing so both stay in sync.

---

## Recommendation

**Include Cognee in the Phase 1 spike alongside QMD.**

The two-tier architecture (QMD for fast recall, Cognee for relational memory) maps cleanly to the PRD's memory requirements without custom graph infrastructure. The Python dependency is acceptable given the target audience.

The open question is graph quality on structured agent memory exports — that's the spike's primary validation target.

Document as ADR-0007 (two-tier memory recall architecture) once the spike validates fit.

---

## References

- GitHub: https://github.com/topoteretes/cognee
- Docs: https://docs.cognee.ai
- Memgraph community call writeup: https://memgraph.com/blog/from-rag-to-graphs-cognee-ai-memory
- Research paper: https://arxiv.org/abs/2505.24478
