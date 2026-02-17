# Spacebot Analysis
**Source:** https://github.com/spacedriveapp/spacebot
**Author:** Jamie Pine (Spacedrive founder)
**Date filed:** 2026-02-17
**Relevance:** Direct competitor/inspiration for SHQ Agent Harness

---

## What It Is

An AI agent harness for teams/communities. Rust, single binary. Multi-surface (Discord, Slack, Telegram). Explicitly positions itself as solving OpenClaw's limitations.

## Key Architecture: 5 Process Types

The core insight: **split the monolith into specialized concurrent processes.**

| Process | Role | Tools | Context |
|---------|------|-------|---------|
| **Channel** | User-facing ambassador. Has soul/personality. Never blocks. | Reply, branch, spawn workers | Conversation + compaction summaries |
| **Branch** | Fork of channel context to think. Like git branch. | Memory recall/save, spawn workers | Fork of channel's context |
| **Worker** | Independent task executor. No personality. | Shell, file, exec, browser | Fresh prompt + task only |
| **Compactor** | Programmatic context monitor. | Monitor + trigger workers | N/A |
| **Cortex** | Inner monologue. Sees across ALL channels/workers. | Memory, consolidation, system monitor | Entire agent scope |

### The Flow
1. User sends message → Channel receives
2. Channel branches to think (preserves channel context)
3. Branch recalls memories, spawns workers if needed
4. Branch returns conclusion → gets deleted
5. Channel responds — was never blocked

### Concurrency Model
- Channel is always responsive (never doing heavy work)
- Multiple branches run in parallel
- Workers are fire-and-forget OR interactive (long-running with follow-ups)
- Message coalescing: batches rapid-fire messages, lets LLM "read the room"

## Memory System

**Typed graph, not markdown files.**

- 8 memory types: Fact, Preference, Decision, Identity, Event, Observation, Goal, Todo
- Graph edges: RelatedTo, Updates, Contradicts, CausedBy, PartOf
- Hybrid recall: vector similarity + full-text search via Reciprocal Rank Fusion
- Storage: SQLite (structured) + LanceDB (vectors)
- Importance scoring: access frequency, recency, graph centrality
- Identity memories exempt from decay
- 3 creation paths: branch-initiated, compactor-initiated, cortex-initiated
- Memory bulletin: cortex generates periodic briefing injected into all conversations
- Memory import: drop markdown files into ingest/ folder for auto-extraction

## Model Routing (4 levels)

1. **Process-type defaults** — channels get best model, workers get cheap model
2. **Task-type overrides** — coding workers upgrade, summarization stays cheap
3. **Prompt complexity scoring** — keyword scorer classifies light/standard/heavy
4. **Fallbacks** — if primary model fails, cascade to alternatives

## Compaction Strategy

- >80%: background compaction (summarize oldest 30%)
- >85%: aggressive (summarize oldest 50%)
- >95%: emergency truncation (hard drop, no LLM)
- Runs alongside channel without blocking

## Skills

- SKILL.md format with frontmatter
- OpenClaw compatible (drop-in)
- Injected into worker system prompts

## Tech Stack

- **Rust** — single binary, no Docker required
- **SQLite** — structured memory
- **LanceDB** — vector embeddings
- Multi-provider LLM (Anthropic, OpenAI, Zhipu/GLM)

## Deployment

- Hosted (spacebot.sh) — one-click
- Self-hosted — single binary
- Docker — container image

---

## Implications for SHQ Agent Harness

### Ideas to Adopt

1. **Channel/Branch/Worker separation** — This is the right architecture. Our PRD's "tree sessions" maps to this but Spacebot's naming and separation of concerns is cleaner.

2. **Cortex concept** — A supervisor process that sees across all sessions. We don't have this in our PRD. Should add it — it's what enables cross-channel memory consolidation and pattern detection.

3. **Message coalescing** — Critical for group/community use. Batch rapid messages, let agent read the room. Our PRD doesn't address this.

4. **Typed memory graph** — Our PRD proposes file-based memory (git-friendly). Spacebot argues structured DB is better. Worth hybrid: structured DB for recall, git-backed exports for portability/versioning.

5. **Model routing by process type** — Smart cost optimization. Channels get expensive models, workers get cheap ones. Our PRD mentions multi-provider but not routing strategy.

6. **Compaction as a separate process** — Not blocking the main channel. Our PRD doesn't address compaction at all.

7. **Interactive workers** — Workers that accept follow-up input for long-running tasks (coding sessions). More flexible than fire-and-forget only.

### Where We Diverge / Can Differentiate

1. **Compound engineering** — Spacebot doesn't mention learning from solved problems. Our "every task makes the system smarter" principle is a differentiator.

2. **Mechanical enforcement** — Linters, structural tests, architecture guardrails. Spacebot doesn't have this.

3. **Multi-agent governance** — Our RFC process, trust model, distributed ownership. Spacebot is single-agent-multi-process, not multi-agent.

4. **Git-native memory** — Spacebot chose DB-first. We could do hybrid (DB for recall + git for audit trail/portability). Git-backed memory is more transparent and composable.

5. **Language choice** — Spacebot chose Rust. We're still deciding. Rust = performance + single binary. Node/TS = ecosystem + faster iteration. Worth noting Spacebot explicitly chose Rust and ships a single binary with zero deps.

### Open Questions This Raises

1. Should we adopt the Channel/Branch/Worker/Cortex naming?
2. Is our file-based memory sufficient or do we need structured DB + vectors?
3. Do we need message coalescing for our use cases (team environments)?
4. Should we build a Cortex-equivalent supervisor from the start?
5. Rust vs Node — Spacebot validates Rust for this domain. Does our team have Rust capacity?
