# Vision: SHQ Agent Harness

> A living document capturing the long-term vision and aspirations for the agent harness. For current execution plans, see the [v1 PRD](prd/2026-02-17-agent-harness-v2.md). For phasing, see the [Roadmap](ROADMAP.md).

---

## The Why

Strip the problem to its root: **we need agents that persist across conversations, collaborate with humans and each other, and reach people where they already are.**

That's it. Everything traces back to one of those three needs: persistence, collaboration, or reach. If a feature doesn't serve one of them — it's cut.

### The Irreducible Problem

SHQ depends on multi-agent operations for daily work. Platform uncertainty creates delivery and control risk. We need a harness we can operate, evolve, and trust.

But we don't need to replicate any existing platform. We need to build the *smallest possible thing* that solves persistence + collaboration + reach, then compound from there.

### Why Not Just Use an Existing Framework?

We audited 9 frameworks (Mastra, CrewAI, Goose, AgentStack, nanobot, OpenHands, Pydantic AI, pi-agent-core, pi-mom). The finding is unambiguous:

> **No framework has channel abstraction.** Zero. This is our differentiator.

The market has mature solutions for agent reasoning, memory, tool systems, multi-agent orchestration, and workflows. But nobody has built channel-agnostic agent deployment with unified messaging. That's the gap, and that's what we build.

---

## North Star

An **agent harness** — not a chatbot framework — that treats scaffolding, feedback loops, and mechanical enforcement as first-class concerns. Every task should leave the system smarter (compound engineering). Humans steer, agents execute.

---

## Design Principles

1. **Harness-first** — The scaffolding, constraints, and feedback loops ARE the product. Not the LLM, not the chat UI.
2. **Compound by default** — Every solved problem becomes a reusable pattern. The system gets smarter with every PR.
3. **Mechanical enforcement** — Encode taste into linters, tests, and structural constraints. Documentation rots; code doesn't.
4. **Start smaller than you think** — Thin core + extensions. Resist building features into the core that belong in the skill layer.
5. **Agent legibility** — If it's not in the repo, it doesn't exist. Repository-local, versioned artifacts are the system of record.
6. **Distributed ownership** — No single maintainer. Bus-factor > 1 from day one.
7. **Repo-native everything** — No Notion, no external wikis. Markdown files, git history, GitHub primitives.

---

## Core Differentiator: Channel Abstraction

Channel-agnostic messaging is our **core differentiator** — no existing framework provides this.

- **Unified interface** — single API for Slack, Telegram, Signal, Discord, IRC, etc.
- **Surface-specific formatting** — auto-adapt output (no markdown tables on Discord, wrap links on WhatsApp, etc.)
- **Channel policies** — allowlists, denylists, requireMention, allowBots per channel
- **Proactive messaging** — heartbeat + cron system for agents that act without being asked
- **Platform independence** — migrate from Slack to Discord without touching agent logic
- **Cross-platform coordination** — receive a task on Slack, report results on Telegram

> **Prior art:** nanobot's channel gateway pattern multiplexes across 9+ chat platforms via config-driven adapters. We build on this with strong typing and plugin isolation in Rust.

---

## Full Architecture Vision

### Session Model: Trees, Not Lists

Sessions are tree-structured, not flat history. Agents can branch (sub-tasks, reviews, research) without polluting the main context. Branches can be discarded or merged back.

- **Main session** — primary conversation with human
- **Branch sessions** — sub-tasks, side-quests, background work
- **Session handoff** — state dump to file before context resets
- **Branch pruning** — TTL or explicit close for garbage collection
- **Audit trails for free** — every decision path is preserved

### Process Model: Channel / Branch / Worker / Cortex

Inspired by Spacebot's production-validated architecture. Split the monolith into specialized concurrent processes:

- **Channel** — User-facing. Has soul, identity, personality. Always responsive — never blocked by work. Delegates thinking to branches and heavy work to workers.
- **Branch** — Fork of channel context. Operates independently; channel only sees the conclusion. Deleted after returning results.
- **Worker** — Independent process for specific tasks. No channel context, no personality. Fire-and-forget or interactive.
- **Cortex (Supervisor)** — Sees across all processes. Full vision includes:
  - Process supervision (stuck cleanup, retries, kill policies)
  - Health signals and alerts
  - **Pattern mining** — detect recurring problems and auto-generate solutions
  - **Memory bulletins** — proactive knowledge sharing across agents
  - **Admin chat interface** — human-readable Cortex interaction
  - Anomaly detection and security monitoring

### Memory Layer (Full Vision)

Structured DB for runtime, repo exports for canonical record. Typed memory graph with hybrid recall.

**Storage:** SQLite (structured metadata + graph edges) + vector store (embeddings for semantic recall)

**Memory types:** Fact, Preference, Decision, Identity, Event, Observation, Goal, Todo

**Graph edges:** RelatedTo, Updates, Contradicts, CausedBy, PartOf

**Recall:** Hybrid — vector similarity + full-text search, merged via Reciprocal Rank Fusion

**Importance scoring:** Access frequency, recency, graph centrality. Identity memories exempt from decay.

**Cross-agent memory:** Scoped access (per-agent, per-team, global) with enforced isolation.

**Legibility model:**
- DB is runtime cache/index and operational store
- Repo export is canonical, reviewable, and portable record
- Import/export determinism required
- Recovery drill: rebuild fresh DB from canonical exports

### Extension/Skill System

- **Skills** = modular, versioned instruction sets
- **Hot-reload** — agents create, modify, and reload extensions at runtime
- **Compound capture** — after every significant task, capture learnings as tagged, searchable solution docs
- **MCP compatibility** (Phase 2) — skills exposable as MCP servers for ecosystem interop
- **Remote skill discovery** — curated registry mapping skill names to git URLs
- **Self-managing tools** — agents extend their own capabilities (with guardrails)

### LLM Abstraction Layer

- Multi-provider from day 1 (Anthropic, OpenAI, Google, local models)
- Hot-swap models mid-session without config surgery
- Thinking trace conversion between provider formats
- Split tool results: structured data for model, clean summary for human
- Model tier routing by process type (expensive for channels, cheap for workers)

---

## Multi-Agent Vision

### Agent-to-Agent Communication

Two channels, by design:
- **RPC layer** — structured data transfer (task handoffs, results, typed payloads)
- **Coordination layer** — messaging surface for decisions, status, human-visible collaboration

### Governance Model

- **Humans:** direction, veto power, merge authority on architectural decisions
- **Agents:** first-class contributors (draft RFCs, write code, review PRs), influence but not authority
- **Trust model:** each agent trusts its primary human fully, treats other agents as collaborators (not instructors)

### Specialized Agent Roles

- Reviewer agents (security, performance, architecture)
- Research agents (repo analysis, framework docs, best practices)
- Maintenance agents (doc gardening, drift detection, cleanup)
- Roles are skill-based, not hardcoded — any agent can load any role

---

## Guardrails & Security Vision

### Tiered Tool Permissions

| Tier | Scope | Default |
|------|-------|---------|
| **Read-only** | Observe, never mutate | Allowed |
| **Workspace-scoped** | Mutate within workspace | Allowed |
| **System-wide** | Mutate outside workspace | Requires approval |
| **Elevated** | Irreversible/high-impact | Requires explicit human approval |

### Defense-in-Depth Security

- Input sanitization and untrusted content marking
- Output filtering (credential/secret scanning)
- Container-level sandboxing (Phase 2+)
- Extension vetting (built-in → allowlisted → untrusted tiers)
- Network egress controls (deny-all-outbound with allowlist)
- Append-only audit logging
- Cryptographic agent identity for RPC
- Periodic permission audits and anomaly detection

### Human-in-the-Loop

- Configurable approval gates per action type
- Kill switch from any channel
- Deferred approval pattern (agent continues non-blocked work while awaiting approval)
- Full audit trail — every action traceable to triggering conversation

### Cost Controls

- Token budgets per session, per task, per day
- Circuit breakers for velocity anomalies
- Model tier restrictions by process type
- Rate limiting on external API calls

---

## Collaboration Interface Vision

### Repo-Local + External Adapters

| Layer | What | Examples |
|---|---|---|
| **Repo-local** | Task files, ADRs, backlog — works offline | Backlog.md, `docs/adr/` |
| **PM adapter** | Syncs to external trackers | GitHub Issues, Linear, Jira |

### Decision Capture

- ADRs in `docs/adr/` with explicit triggers (emoji reaction, label, CLI)
- Agents draft ADRs, humans approve
- Git-based, reviewable, revertible

---

## Human Interaction Vision

### Onboarding

- Agent introduces itself on first interaction (reads SOUL.md, AGENTS.md)
- Conversational setup that generates standard config files
- Custom agent distributions/templates for quick onboarding

### Team Dynamics

- One primary agent per human, shared agents for team functions
- Context-aware behavior (informal in DMs, conservative in group channels)
- Handoff support when humans are away (canonical exports = handover notes)

### Daily Workflow

- Morning summary, human prioritizes, agent executes, end-of-day recap
- Code changes as PRs, task updates in Backlog.md
- Graduated urgency levels (silent → normal → urgent → critical)

### Error Communication Protocol

1. Explain what failed and why (plain language)
2. Say what was tried
3. Suggest what the human can do
4. Don't retry silently on semantic failures

---

## Observability Vision

- Structured JSON logging (always-on, not debug-only)
- OpenTelemetry integration for distributed tracing
- CLI-first observability (`harness status`, `harness costs`, `harness health`)
- Lightweight web dashboard (Phase 2)
- Cross-agent observability for multi-agent setups (Phase 3)
- Benchmark-driven development (10-15 standard scenarios run on every PR)

---

## Deployment Vision

- **Per-agent daemon** — each agent as its own process
- **Local-first** — runs on user's machine or VPS, no mandatory cloud
- **Agent discovery** — lightweight coordinator mapping agent IDs to endpoints
- **Containerized deployment** — Docker Compose for multi-agent setups (optional)
- **Single-machine multi-agent** (v1) → **Distributed multi-machine** (Phase 3)

---

## Research Patterns to Incorporate

These patterns from our 9-framework audit inform future development:

| Pattern | Source | Phase |
|---------|--------|-------|
| Context compression (log + context split) | OpenHands, pi-mom | v1 |
| Validation-retry loop for tool args | Pydantic AI | v1 |
| Structured output guarantee | Pydantic AI | v1 |
| Deferred approval (non-blocking gates) | Pydantic AI | v2 |
| Action/Observation formalism | OpenHands | v2 |
| Typed dependency injection | Pydantic AI | v1 |
| Custom agent distributions | Goose | v2 |
| Remote skill discovery | nanobot | v2 |
| Workflow engine with suspend/resume | Mastra | v3 |
| Benchmark-driven development | OpenHands | v1 |

---

## References

- [OpenAI Harness Engineering](https://openai.com/index/harness-engineering/)
- [Compound Engineering (Kieran Klaassen / Cora)](https://every.to/guides/compound-engineering)
- [Spacebot (Spacedrive)](https://github.com/spacedriveapp/spacebot)
- [Pi-AI architecture (pi-mono)](https://github.com/badlogic/pi-mono)
- [Mastra](https://mastra.ai), [CrewAI](https://docs.crewai.com), [nanobot](https://github.com/HKUDS/nanobot)
- [Goose (Block)](https://github.com/block/goose), [Pydantic AI](https://github.com/pydantic/pydantic-ai), [OpenHands](https://github.com/All-Hands-AI/OpenHands)
