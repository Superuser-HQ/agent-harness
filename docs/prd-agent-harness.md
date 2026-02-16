# PRD: SHQ Agent Harness
**Status:** Draft v0.3
**Author:** Kani (driven), Rem (reviewer)
**Date:** 2026-02-16
**Stakeholders:** Yao, Gerald

---

## 1. Problem Statement

OpenClaw's creator is leaving for OpenAI; the project moves to a foundation with uncertain roadmap. SHQ depends on OpenClaw for multi-agent operations (Kani + Rem), messaging across surfaces, memory continuity, and proactive automation. Without a replacement, we're locked into a platform we can't steer.

**Opportunity:** Build a lean, open-source agent harness under SHQ that preserves what works, fixes what's missing, and is designed for multi-agent collaboration from day one.

## 2. Vision

An agent harness — not a chatbot framework — that treats scaffolding, feedback loops, and mechanical enforcement as first-class concerns. Every task should leave the system smarter (compound engineering). Humans steer, agents execute.

## 3. Design Principles

1. **Harness-first** — The scaffolding, constraints, and feedback loops ARE the product. Not the LLM, not the chat UI.
2. **Compound by default** — Every solved problem becomes a reusable pattern. The system gets smarter with every PR.
3. **Mechanical enforcement** — Encode taste into linters, tests, and structural constraints. Documentation rots; code doesn't.
4. **Start smaller than you think** — Thin core + extensions. Resist building features into the core that belong in the skill layer.
5. **Agent legibility** — If it's not in the repo, it doesn't exist. Repository-local, versioned artifacts are the system of record.
6. **Distributed ownership** — No single maintainer. Bus-factor > 1 from day one.

## 4. Core Architecture

### 4.1 Session Model: Trees, Not Lists

Sessions are tree-structured, not flat history. Agents can branch (sub-tasks, reviews, research) without polluting the main context. Branches can be discarded or merged back.

- **Main session** — primary conversation with human
- **Branch sessions** — sub-tasks, side-quests, background work
- **Session handoff** — state dump to file before context resets (solves the "cold pickup" problem)

### 4.2 Base Tools (5 primitives)

The core ships with exactly 5 tools. Everything else is an extension.

| Tool | Purpose |
|------|---------|
| **Read** | Read files, images, structured data |
| **Write** | Create/overwrite files |
| **Edit** | Surgical text replacement |
| **Shell** | Execute commands, manage processes |
| **Message** | Send/receive across surfaces (Slack, Telegram, Signal, etc.) |

Memory is an **architectural layer**, not a tool (see 4.5). Tools interact with memory through Read/Write (files) and a thin `remember`/`recall` API.

### 4.3 Extension/Skill System

- **Skills** = modular, versioned instruction sets (like OpenClaw's SKILL.md pattern)
- **Hot-reload** — agents can create, modify, and reload extensions at runtime
- **Progressive disclosure** — small AGENTS.md (~100 lines) as table of contents, deeper docs in structured `docs/`
- **Compound capture** — after every significant task, the system prompts to capture learnings as tagged, searchable solution docs in `docs/solutions/`

### 4.4 LLM Abstraction Layer

- Multi-provider from day 1 (Anthropic, OpenAI, Google, local models)
- Hot-swap models mid-session without config surgery
- Thinking trace conversion between provider formats
- Split tool results: structured data for model, clean summary for human

### 4.5 Memory Layer

Three tiers, all file-based (git-friendly):

1. **Session memory** — tree-structured conversation history
2. **Daily memory** — `memory/YYYY-MM-DD.md` raw logs
3. **Long-term memory** — `MEMORY.md` curated knowledge + `docs/solutions/` tagged, searchable solution docs with YAML frontmatter

Memory is git-backed. Multiple agents read/write via shared repo with conventions (clear ownership per section, structured files).

### 4.6 Mechanical Enforcement

- **Architecture linters** — enforce layer boundaries, dependency directions, naming conventions
- **Structural tests** — validate repo knowledge base is up-to-date and cross-linked
- **Custom lint error messages** — include remediation instructions so agents self-fix
- **Recurring cleanup agents** — scan for drift, open fix-up PRs (garbage collection pattern)

**Starter enforcement rules (v1):**
1. No direct LLM API calls outside the abstraction layer
2. Every skill must have SKILL.md with `name`, `description`, and `tools` fields
3. No tool can write outside workspace without explicit permission

### 4.7 Error Handling & Resilience

- **Retry with backoff** — automatic retry on transient failures (rate limits, network errors)
- **Provider failover** — if provider X is down, fall back to provider Y automatically
- **Session recovery** — checkpoint session state; recover from crashes without losing context
- **Graceful degradation** — reduced capability beats total failure (e.g., fall back to simpler model if primary unavailable)
- **Abort support** — clean cancellation of in-progress tool calls with partial result preservation

## 5. Multi-Agent Primitives (Phase 2 core)

### 5.1 Agent-to-Agent Communication

Two channels, by design:

- **RPC layer** — structured data transfer between agents (task handoffs, results, typed payloads). Logged but not surfaced to humans by default.
- **Coordination layer** — messaging surface (Slack, etc.) for decisions, status updates, and human-visible collaboration. Humans see what agents are deciding.

Think: Slack is the standup, RPC is the API call.

- **Handoff protocol** — typed task handoffs with context, constraints, and expected output format
- **Shared artifacts via git** — code and specs through PRs, coordination through messaging

### 5.2 Governance Model

- **Humans:** direction, veto power, merge authority on architectural decisions
- **Agents:** first-class contributors (draft RFCs, write code, review PRs), influence but not authority
- **Trust model:** each agent trusts its primary human fully, trusts other team humans for project scope, treats other agents as collaborators (not instructors)
- **RFC process:** any significant change gets a written proposal; agents can author, humans approve

### 5.3 Specialized Agent Roles

- Reviewer agents (security, performance, architecture, data integrity)
- Research agents (repo analysis, framework docs, best practices)
- Maintenance agents (doc gardening, drift detection, cleanup)
- Roles are skill-based, not hardcoded — any agent can load any role

## 6. Messaging Surface Abstraction

Channel-agnostic messaging is OpenClaw's killer feature. Preserve it.

- **Unified interface** — single API for Slack, Telegram, Signal, Discord, IRC, etc.
- **Surface-specific formatting** — auto-adapt output (no markdown tables on Discord, wrap links on WhatsApp, etc.)
- **Channel policies** — allowlists, denylists, requireMention, allowBots per channel
- **Proactive messaging** — heartbeat + cron system for agents that act without being asked

## 7. Proactive Automation

- **Heartbeat system** — periodic check-ins, batched checks (email, calendar, mentions)
- **Cron scheduler** — exact timing, isolated sessions, different models per task
- **Compound loop integration** — recurring agents that run Plan → Work → Review → Compound on maintenance tasks

## 8. Deployment Model

- **Per-agent daemon** — each agent runs as its own process (like OpenClaw today). Keeps isolation simple.
- **Local-first** — runs on user's machine or a VPS. No mandatory cloud dependency.
- **Agent discovery** — agents register with a lightweight coordinator (config file or local service) that maps agent IDs to endpoints. For v1, this can be a shared JSON file in the repo.
- **Containerized deployment (optional)** — Docker Compose for multi-agent setups. Each agent = one container. Shared network for RPC, messaging surface for human-visible coordination.
- **Single-machine multi-agent** — v1 target. Distributed multi-machine is Phase 3.

## 9. What We Deliberately Omit (v1)

- **Dashboard/observability UI** — use logs and CLI for now; build when it hurts
- **Plugin marketplace** — skills are git repos; discovery is manual until scale demands more
- **Visual agent builder** — no drag-and-drop; code-first
- **Background parallel bash** — simplicity > parallelism for v1

## 10. Success Criteria

1. Single agent running on new core, talking on one surface (end of week 3)
2. Two agents collaborating through new system (end of week 4)
3. Feature parity with OpenClaw features we actually use (end of week 6)
4. All SHQ agent operations migrated off OpenClaw (end of week 8)

## 11. Key Decisions & Open Questions

### Resolved (Kani + Rem aligned, pending human approval):
- **Language/runtime:** TypeScript/Node.js — matches OpenClaw, zero ramp-up, strong ecosystem
- **License:** Apache 2.0 — permissive with patent protection
- **Governance:** SHQ-owned to start, foundation later if traction warrants

### Critical Fork (needs human decision):
- **Build own agent loop vs wrap pi-agent-core?** Pi gives us session trees + multi-provider + extension hot-reload for free. But Pi is terminal-first; we need messaging-first. Needs validation that it fits before committing. This is THE architectural decision.

### Open:
- **Name?** Working title TBD. Short, memorable, not taken on npm.
- **Messaging adapter architecture?** How do we abstract Slack/Telegram/Signal cleanly? Port OpenClaw's adapter pattern or design fresh?

## 12. Timeline

| Week | Milestone |
|------|-----------|
| 1 | Audit: what OpenClaw features we actually use + gap analysis |
| 2 | Design doc: PRD refined + architecture decisions (critical fork resolved) |
| 3 | Prototype: single agent on new core, one surface |
| 4 | Multi-agent: two agents collaborating through new system |
| 5 | Dogfood: run one real SHQ task end-to-end, discover what's broken |
| 6-7 | Feature parity with used OpenClaw features (informed by dogfood) |
| 8-9 | Migration: move SHQ operations to new system |

---

## References

- [OpenAI Harness Engineering](https://openai.com/index/harness-engineering/)
- [Compound Engineering (Kieran Klaassen / Cora)](https://every.to/guides/compound-engineering)
- [Pi-AI architecture (pi-mono)](https://github.com/badlogic/pi-mono) (via Rem's digest)
- OpenClaw feature audit (to be completed Week 1)
