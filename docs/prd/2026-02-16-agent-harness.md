# PRD: SHQ Agent Harness
**Status:** v1.2
**Author:** Kani (driven), Rem (reviewer)
**Date:** 2026-02-16
**Stakeholders:** Yao, Gerald

---

## 1. Why

Strip the problem to its root: **we need agents that persist across conversations, collaborate with humans and each other, and reach people where they already are.**

That's it. Everything in this document serves one of those three needs. If a feature doesn't trace back to persistence, collaboration, or reach — it's cut.

### 1.1 The Irreducible Problem

OpenClaw's creator is joining OpenAI. The project moves to a foundation with uncertain roadmap. SHQ depends on OpenClaw for:

1. **Persistence** — memory continuity across sessions (SOUL.md, MEMORY.md, daily logs)
2. **Collaboration** — multi-agent operations (Kani + Rem), human-agent governance
3. **Reach** — messaging across surfaces (Slack, Telegram, Signal, Discord)

Without a replacement, we're locked into a platform we can't steer. But we don't need to replicate OpenClaw. We need to build the *smallest possible thing* that solves persistence + collaboration + reach, then compound from there.

### 1.2 Why Not Just Use an Existing Framework?

We audited 9 frameworks (Mastra, CrewAI, Goose, AgentStack, nanobot, OpenHands, Pydantic AI, pi-agent-core, pi-mom). The finding is unambiguous:

> **No framework has channel abstraction.** Zero. This is our differentiator.

The market has mature solutions for agent reasoning, memory, tool systems, multi-agent orchestration, and workflows. But nobody has built channel-agnostic agent deployment with unified messaging. That's the gap, and that's what we build.

### 1.3 Why Now?

OpenClaw's transition creates a 2-3 month window. If we don't build now, we either migrate to a degrading platform or lose capability. The research is done. The design space is mapped. Ship.

---

## 2. Vision

An agent harness — not a chatbot framework — that treats scaffolding, feedback loops, and mechanical enforcement as first-class concerns. Every task should leave the system smarter (compound engineering). Humans steer, agents execute.

---

## 3. Design Principles

1. **Harness-first** — The scaffolding, constraints, and feedback loops ARE the product. Not the LLM, not the chat UI.
2. **Compound by default** — Every solved problem becomes a reusable pattern. The system gets smarter with every PR.
3. **Mechanical enforcement** — Encode taste into linters, tests, and structural constraints. Documentation rots; code doesn't.
4. **Start smaller than you think** — Thin core + extensions. Resist building features into the core that belong in the skill layer.
5. **Agent legibility** — If it's not in the repo, it doesn't exist. Repository-local, versioned artifacts are the system of record.
6. **Distributed ownership** — No single maintainer. Bus-factor > 1 from day one.
7. **Repo-native everything** — No Notion, no external wikis. Markdown files, git history, GitHub primitives. If it's not in the repo, it didn't happen.

---

## 4. Core Architecture

### 4.1 Session Model: Trees, Not Lists

Sessions are tree-structured, not flat history. Agents can branch (sub-tasks, reviews, research) without polluting the main context. Branches can be discarded or merged back.

- **Main session** — primary conversation with human
- **Branch sessions** — sub-tasks, side-quests, background work
- **Session handoff** — state dump to file before context resets (solves the "cold pickup" problem)

> **Research validation:** pi-agent-core's event streaming architecture (`agent_start` → `turn_start` → `message_update` → `tool_execution_*` → `turn_end`) provides the granular event model we need for session trees. Their `transformContext() → convertToLlm()` message pipeline cleanly separates app-level messages from LLM messages. We adopt these patterns.

#### Second-Order Effects
- Tree-structured sessions create **garbage collection needs** — abandoned branches accumulate. We need a branch pruning policy (TTL or explicit close).
- Session trees enable **audit trails** — every decision path is preserved, which feeds into our ADR process (§8).
- Branches that produce useful output should trigger **compound capture** — the system prompts to generalize the solution.

### 4.2 Base Tools (5 primitives)

The core ships with exactly 5 tools. Everything else is an extension.

| Tool | Purpose |
|------|---------|
| **Read** | Read files, images, structured data |
| **Write** | Create/overwrite files |
| **Edit** | Surgical text replacement |
| **Shell** | Execute commands, manage processes |
| **Message** | Send/receive across surfaces (Slack, Telegram, Signal, etc.) |

> **Research validation:** pi-mom's 5 core tools (`bash`, `read`, `write`, `edit`, `attach`) are almost identical. This independent convergence validates the design. pi-mom's self-managing model — where the agent creates its own tools as bash scripts — demonstrates that 5 primitives are sufficient for compound capability growth.

Memory is an **architectural layer**, not a tool (see §4.5). Tools interact with memory through Read/Write (files) and a thin `remember`/`recall` API.

### 4.3 Extension/Skill System

- **Skills** = modular, versioned instruction sets (like OpenClaw's SKILL.md pattern)
- **Hot-reload** — agents can create, modify, and reload extensions at runtime
- **Progressive disclosure** — small AGENTS.md (~100 lines) as table of contents, deeper docs in structured `docs/`
- **Compound capture** — after every significant task, the system prompts to capture learnings as tagged, searchable solution docs in `docs/solutions/`
> **Research insight:** nanobot's self-configuring skill URLs (agent reads a URL, follows instructions, configures itself) is an elegant zero-setup pattern worth adopting.

#### Second-Order Effects
- Self-managing tools (pi-mom pattern) means **agents can extend their own capabilities without human intervention**. This is powerful but needs guardrails — new tools should be PR'd, not silently deployed.

#### Phase 2: MCP Compatibility
- Skills should be exposable as MCP servers for interop with Goose, Mastra, and the broader ecosystem. This is deliberately Phase 2 — we don't need external agents consuming our skills on day one. Get the skill system working first, add MCP exposure when it's stable. (Goose's MCP-native extension architecture validates MCP as the right protocol.)

### 4.4 LLM Abstraction Layer

- Multi-provider from day 1 (Anthropic, OpenAI, Google, local models)
- Hot-swap models mid-session without config surgery
- Thinking trace conversion between provider formats
- Split tool results: structured data for model, clean summary for human

> **Research insight:** Mastra routes 600+ models through a unified layer (wrapping Vercel AI SDK). pi-agent-core provides `setModel()`, `setThinkingLevel()` at runtime. Pydantic AI's validation-retry loop (invalid tool args get sent back to LLM for self-correction) should be ported — use Zod/TypeBox for schema validation.

### 4.5 Memory Layer

Three tiers, all file-based (git-friendly):

1. **Session memory** — tree-structured conversation history
2. **Daily memory** — `memory/YYYY-MM-DD.md` raw logs
3. **Long-term memory** — `MEMORY.md` curated knowledge + `docs/solutions/` tagged, searchable solution docs with YAML frontmatter

**File-based vector store:** Files remain the source of truth. FAISS or USearch provides an index layer over markdown files for semantic search. No separate database — vectors are derived from files, rebuilt on change. This means `grep` still works, `git diff` still works, and the vector index is a cache, not a store.

> **Research insight:** Mastra has the most sophisticated memory system we found — working memory (structured, persistent), semantic recall (vector-based), and observational memory (background agents maintaining dense observation logs). CrewAI's unified Memory class adds composite scoring (semantic similarity × recency × importance) with tunable weights and self-organizing scope trees. We adopt Mastra's tiered architecture and CrewAI's scoring model, but keep files as ground truth with vectors as index.

#### Second-Order Effects
- File-based vectors mean **memory is portable** — clone the repo, rebuild the index, full memory on any machine.
- Vector indexing over markdown means **search quality depends on file structure** — we need conventions for how memory files are formatted (frontmatter, headers, atomic facts).
- Git-backed memory creates **merge conflicts when multiple agents write simultaneously** — need clear ownership per file/section, or use append-only patterns.

Memory is git-backed. Multiple agents read/write via shared repo with conventions (clear ownership per section, structured files).

### 4.6 Memory Policy (`MEMORY_POLICY.md`)

Memory is not left to each agent's ad-hoc judgment. An explicit `MEMORY_POLICY.md` in every agent workspace defines:

**What to remember:**
- Decisions and their rationale
- User preferences and corrections
- Lessons learned from failures
- Project context that changes rarely but matters always

**What NOT to remember:**
- Secrets, credentials, tokens (use env vars or secret managers)
- Transient state (build output, temp files)
- Verbatim conversation logs beyond daily summaries
- Other people's private information encountered in shared channels

**When to remember:**
- End of every significant task → update daily memory
- Weekly → review daily logs, distill into MEMORY.md
- On explicit trigger → human says "remember this" or reacts with 📌

**How to remember:**
- Atomic facts, not narratives (one fact per line, greppable)
- YAML frontmatter on solution docs (`tags`, `date`, `problem`, `solution`)
- Vector index rebuilt on git push (CI hook or file watcher)

**Hygiene rules:**
- Memory files have a max size (e.g., MEMORY.md < 10KB). When exceeded, archive older entries.
- Daily files older than 90 days get summarized and archived.
- Duplicate facts across files get deduplicated during weekly review.

#### Second-Order Effects
- Explicit memory policy means **new agents onboard faster** — they read the policy, not reverse-engineer conventions from examples.
- Size limits force **curation over accumulation** — agents learn to distinguish signal from noise.
- Archive rules create a **two-tier recall system** — hot memory (recent, in-context) and cold memory (archived, vector-searchable).

### 4.7 Mechanical Enforcement

- **Architecture linters** — enforce layer boundaries, dependency directions, naming conventions
- **Structural tests** — validate repo knowledge base is up-to-date and cross-linked
- **Custom lint error messages** — include remediation instructions so agents self-fix
- **Recurring cleanup agents** — scan for drift, open fix-up PRs (garbage collection pattern)

**Starter enforcement rules (v1):**
1. No direct LLM API calls outside the abstraction layer
2. Every skill must have SKILL.md with `name`, `description`, and `tools` fields
3. No tool can write outside workspace without explicit permission
4. MEMORY_POLICY.md must exist in every agent workspace
5. ADRs must have status, date, and decision fields

### 4.8 Error Handling & Resilience

- **Retry with backoff** — automatic retry on transient failures (rate limits, network errors)
- **Provider failover** — if provider X is down, fall back to provider Y automatically
- **Session recovery** — checkpoint session state; recover from crashes without losing context
- **Graceful degradation** — reduced capability beats total failure (e.g., fall back to simpler model if primary unavailable)
- **Abort support** — clean cancellation of in-progress tool calls with partial result preservation

> **Research insight:** Pydantic AI's durable execution (checkpoint and resume across failures) and pi-agent-core's steering messages (interrupt running agents, queue follow-up work) are both patterns we adopt.

---

## 5. Multi-Agent Primitives (Phase 2 core)

### 5.1 Agent-to-Agent Communication

Two channels, by design:

- **RPC layer** — structured data transfer between agents (task handoffs, results, typed payloads). Logged but not surfaced to humans by default.
- **Coordination layer** — messaging surface (Slack, etc.) for decisions, status updates, and human-visible collaboration. Humans see what agents are deciding.

Think: Slack is the standup, RPC is the API call.

- **Handoff protocol** — typed task handoffs with context, constraints, and expected output format
- **Shared artifacts via git** — code and specs through PRs, coordination through messaging

#### Second-Order Effects
- Two communication channels means **agents must decide which to use** — need clear conventions (data → RPC, decisions → coordination layer).
- Human-visible coordination means **agents become accountable** — their reasoning is auditable in Slack history, which feeds into decision logging (§8).
- Git-based shared artifacts means **agent work is reviewable** — PRs from agents get the same review process as PRs from humans.

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

---

## 6. Messaging Surface Abstraction

Channel-agnostic messaging is our **core differentiator** — no existing framework provides this.

- **Unified interface** — single API for Slack, Telegram, Signal, Discord, IRC, etc.
- **Surface-specific formatting** — auto-adapt output (no markdown tables on Discord, wrap links on WhatsApp, etc.)
- **Channel policies** — allowlists, denylists, requireMention, allowBots per channel
- **Proactive messaging** — heartbeat + cron system for agents that act without being asked

> **Research insight:** nanobot's channel gateway pattern is the closest prior art — a central `nanobot gateway` process multiplexes across 9+ chat platforms (Telegram, Discord, WhatsApp, Slack, Email, QQ, Feishu, DingTalk, Mochat) via config-driven adapters. We study this architecture deeply but build in TypeScript with stronger typing and plugin isolation.

#### Second-Order Effects
- Channel abstraction means **agents are platform-independent** — migrate from Slack to Discord without touching agent logic.
- Unified messaging means **cross-platform coordination is natural** — an agent can receive a task on Slack and report results on Telegram.
- Platform-specific formatting rules create a **growing compatibility matrix** — need automated tests per platform to catch regressions.

---

## 7. Collaboration Interface

### 7.1 Two Layers: Repo-Local + External Adapters

The same principle that drives our messaging abstraction (§6) applies to project management: **don't hardcode GitHub, Linear, or Jira — abstract them.**

| Layer | What | Examples |
|---|---|---|
| **Repo-local (always available)** | Task files, ADRs, backlog — works offline, no API needed | [Backlog.md](https://github.com/MrLesk/Backlog.md), `docs/adr/` |
| **PM adapter (pluggable)** | Syncs repo-local tasks to external trackers | GitHub Issues, Linear, Jira |

The harness depends on the repo-local layer only. PM adapters are optional plugins that sync outward.

### 7.2 Repo-Local Task Management (Backlog.md)

[Backlog.md](https://github.com/MrLesk/Backlog.md) is the agent-native task layer:

- **Each task = individual markdown file** in `backlog/` → zero merge conflicts by design
- **AI-native** — MCP + CLI integration, agents create/pick up/complete tasks naturally
- **Kanban board** in terminal (`backlog board`) or web (`backlog browser`)
- **100% repo-local, git-friendly** — no API keys, no external dependencies
- **Ownership rule:** the agent _assigned_ to a task updates its status. Unassigned items are updated by whoever picks them up.

This is the source of truth for day-to-day agent work. Always available, even offline.

### 7.3 PM Adapters (Phase 2)

Optional adapters sync Backlog.md tasks to external project management tools:

- **GitHub adapter** — sync to GitHub Issues/Projects. Agents participate in Discussions, author PRs.
- **Linear adapter** — sync to Linear. SHQ's current PM tool.
- **Jira adapter** — sync to Jira. Enterprise teams.

Adapters are bidirectional: create a task in Linear → it appears in `backlog/`. Complete a task in `backlog/` → Linear updates. Conflict resolution: repo-local wins (same principle as file-based memory).

### 7.4 Conventions

- **Agents are first-class contributors** — they author PRs, comment on tasks, participate in discussions regardless of which PM tool is in use.
- **Labels/tags** distinguish human-created vs agent-created tasks (`source:human`, `source:agent`).
- **ADRs** (`docs/adr/`) are always repo-local, never synced to external tools (decisions live in git).

#### Second-Order Effects
- Repo-local as source of truth means **the harness works without any external service** — clone the repo, you have everything.
- PM adapters as plugins means **teams aren't locked into our tool choices** — use whatever PM tool you already have.
- Abstracting PM the same way we abstract messaging creates **a consistent extension pattern** — adapters for everything.

---

## 8. Decision Capture Protocol

Decisions happen in conversations (Slack, GitHub Discussions, PRs). The record lives in the repo.

### 8.1 Architecture Decision Records (ADRs)

Location: `docs/adr/NNNN-title.md`

```markdown
# ADR-0001: TypeScript as primary language

**Status:** Accepted
**Date:** 2026-02-16
**Deciders:** Yao, Kani, Rem

## Context
[What prompted this decision]

## Decision
[What we decided]

## Consequences
[What happens as a result — good and bad]

## Alternatives Considered
[What else we looked at and why we didn't pick it]
```

### 8.2 Explicit Capture Trigger

Decisions are captured via **explicit trigger**, not magic detection:

- **Slack:** React with 📋 (clipboard emoji) on a message containing a decision. The agent creates an ADR draft as a PR.
- **GitHub Discussion:** Label a comment with `decision`. The agent extracts and drafts an ADR.
- **CLI:** `harness decision "We chose X because Y"` — creates ADR directly.
- **Agent-initiated:** When an agent recognizes it's making a significant choice, it drafts an ADR and requests human approval before merging.

### 8.3 What Qualifies as a Decision

Not everything is an ADR. Capture when:
- An architectural or design choice is made that constrains future options
- A technology, library, or pattern is chosen over alternatives
- A convention or policy is established
- A significant tradeoff is accepted

#### Second-Order Effects
- Explicit triggers mean **no false positives** — decisions are captured intentionally, not guessed at.
- Git-based ADRs mean **decisions are reviewable, revertible, and linkable** — "why did we do X?" is always answerable.
- Agent-drafted ADRs mean **the capture cost is near-zero** — react with an emoji, get a PR.

---

## 9. Proactive Automation

- **Heartbeat system** — periodic check-ins, batched checks (email, calendar, mentions)
- **Cron scheduler** — exact timing, isolated sessions, different models per task
- **Compound loop integration** — recurring agents that run Plan → Work → Review → Compound on maintenance tasks

---

## 10. Deployment Model

- **Per-agent daemon** — each agent runs as its own process (like OpenClaw today). Keeps isolation simple.
- **Local-first** — runs on user's machine or a VPS. No mandatory cloud dependency.
- **Agent discovery** — agents register with a lightweight coordinator (config file or local service) that maps agent IDs to endpoints. For v1, this can be a shared JSON file in the repo.
- **Containerized deployment (optional)** — Docker Compose for multi-agent setups. Each agent = one container. Shared network for RPC, messaging surface for human-visible coordination.
- **Single-machine multi-agent** — v1 target. Distributed multi-machine is Phase 3.

---

## 11. What We Deliberately Omit (v1)

- **Dashboard/observability UI** — use logs and CLI for now; build when it hurts (but add OpenTelemetry hooks cheaply — Pydantic AI validates this pattern)
- **Plugin marketplace** — skills are git repos; discovery is manual until scale demands more
- **Visual agent builder** — no drag-and-drop; code-first
- **Background parallel bash** — simplicity > parallelism for v1
- **Notion/Jira/external trackers** — repo-native or nothing

---

## 12. Success Criteria

1. Single agent running on new core, talking on one surface (end of week 3)
2. Two agents collaborating through new system (end of week 4)
3. Feature parity with OpenClaw features we actually use (end of week 6)
4. All SHQ agent operations migrated off OpenClaw (end of week 8)
5. MEMORY_POLICY.md enforced by linter (end of week 3)
6. First ADR captured via emoji trigger (end of week 4)
7. Vector search over memory files returning relevant results (end of week 5)

---

## 13. Critical Fork: Build vs. Wrap pi-agent-core

This is THE architectural decision. It must be resolved in Week 2.

### Option A: Wrap pi-agent-core

**What we get for free:**
- Session state management, event streaming, tool execution, context management
- TypeScript, MIT licensed, same 5 primitives we'd build anyway
- `transformContext() → convertToLlm()` message pipeline
- Steering messages (interrupt running agents) and follow-up queues
- Dynamic model/tool swapping at runtime
- Saves 2-3 weeks of core loop development

**What we fight:**
- Terminal-first assumptions — pi-agent-core is built for coding agents, not messaging-first agents
- Flat history only — no session trees
- No multi-agent awareness in the core
- Monorepo coupling — extracting pi-agent-core pulls in pi-ai and potentially more
- Dependency on Mario Zechner's maintenance priorities and roadmap
- Context compaction assumes terminal interaction patterns

### Option B: Build own core, port patterns

**What we get:**
- Messaging-first from day one — session model designed for channel abstraction
- Session trees native in the core
- Multi-agent primitives baked in, not bolted on
- Full control over maintenance and roadmap
- Clean dependency tree

**What it costs:**
- 4-5 weeks additional development for agent loop, event system, context management, session trees, multi-provider abstraction, and error handling (corrected from initial 2-3 week estimate — that was optimistic)
- Risk of re-inventing solved problems
- Smaller community (just us) vs. pi-mono's existing users

### Tradeoff Analysis

| Dimension | Wrap pi-agent-core | Build own |
|---|---|---|
| Time to first agent | ~1 week | ~3 weeks |
| Time to messaging-first | ~3 weeks (fighting abstractions) | ~3 weeks (building right) |
| Time to multi-agent | ~5 weeks (bolting on) | ~5 weeks (native, but done right) |
| Long-term maintenance | Upstream dependency risk | Full ownership |
| Architecture fit | 70% match | 100% match |

### Recommendation

**Build own core, steal patterns aggressively.** The patterns from pi-agent-core (event model, message pipeline, steering, context compaction) are more valuable than the code. We adopt the *design* but own the *implementation*. Specific patterns to port:

1. Event streaming architecture (granular events for UI, logging, inter-agent coordination)
2. Message flow pipeline (app messages → transform → LLM messages)
3. Steering and follow-up queues
4. Context compaction strategy
5. Dynamic model/tool swapping

From Mastra, additionally adopt:
- Working memory as structured, persistent state (not just conversation history)
- Observational memory pattern (background agents maintaining dense observation logs)

From CrewAI, additionally adopt:
- Composite memory scoring (semantic × recency × importance with tunable weights)
- Scope tree concept for hierarchical memory organization

### Decision Status: **PENDING — requires human (Yao) approval.**

---

## 14. Key Decisions & Open Questions

### Resolved (Kani + Rem aligned, pending human approval):
- **Language/runtime:** TypeScript/Node.js — matches OpenClaw, zero ramp-up, strong ecosystem
- **License:** Apache 2.0 — permissive with patent protection
- **Governance:** SHQ-owned to start, foundation later if traction warrants
- **Task tracking:** Repo-native (backlog.md + GitHub Issues/Projects). No Notion.
- **Decision records:** Git-based ADRs with explicit capture triggers
- **Memory store:** File-based with FAISS/USearch vector index. Files are truth, vectors are cache.

### Critical Fork #2: Mastra as Dependency
This is on par with the pi-agent-core decision. Mastra is the strongest architectural match (TypeScript, memory, MCP, workflows). Three options:
- **A) Depend on Mastra** — use as library for agent primitives, memory, and workflows. Fast start, upstream dependency risk.
- **B) Fork patterns** — study Mastra's architecture, reimplement in our core. Slower, full ownership.
- **C) Complement** — use Mastra for what it does well (memory, workflows), build our own for what it doesn't (messaging, session trees). Mix-and-match.

Needs hands-on evaluation in Week 2. **Decision Status: PENDING.**

### Open:
- **Name?** Working title TBD. Short, memorable, not taken on npm.
- **Messaging adapter architecture?** Study nanobot's gateway pattern, then design. Port OpenClaw's adapter pattern or design fresh?
- **Vector store choice?** FAISS (mature, C++ with Node bindings) vs USearch (newer, potentially faster, better Node support). Benchmark needed.

---

## 15. Testing Strategy

Agent frameworks are notoriously hard to test. Our approach:

1. **Unit tests** — tool implementations, message formatting, memory read/write, vector index operations. Standard Jest/Vitest.
2. **Integration tests** — messaging adapters (mock Slack/Telegram APIs), LLM abstraction (mock provider responses), session lifecycle (create → branch → merge → prune).
3. **Golden path E2E** — send message → agent processes → tools execute → response delivered → memory updated. One test per messaging surface. Run on every PR.
4. **Replay tests** — record real agent sessions, replay against new code to catch regressions in behavior (not just API contracts).
5. **Memory integrity** — vector index matches file content after writes, concurrent agent writes don't corrupt state.

Tests are not Phase 2. The golden path E2E ships with the first prototype (Week 3).

---

## 16. Timeline

| Week | Milestone |
|------|-----------|
| 1 | Audit: what OpenClaw features we actually use + gap analysis. Set up repo with GitHub Projects, ADR template, MEMORY_POLICY.md. |
| 2 | Architecture decisions: critical fork resolved (ADR-0001), Mastra dependency decision (ADR-0002), messaging adapter design (ADR-0003). |
| 3 | Prototype: single agent on new core, one messaging surface, memory policy enforced by linter. |
| 4 | Multi-agent: two agents collaborating. Decision capture via emoji trigger working. GitHub Issues/Projects integration. |
| 5 | Memory: vector index over memory files. Compound capture loop. Dogfood one real SHQ task end-to-end. |
| 6-7 | Feature parity with used OpenClaw features (informed by dogfood). Second messaging surface. |
| 8-9 | Migration: move SHQ operations to new system. Retrospective ADR on what worked/didn't. |

---

## References

- [OpenAI Harness Engineering](https://openai.com/index/harness-engineering/)
- [Compound Engineering (Kieran Klaassen / Cora)](https://every.to/guides/compound-engineering)
- [Pi-AI architecture (pi-mono)](https://github.com/badlogic/pi-mono)
- [Mastra framework](https://mastra.ai) — TypeScript agent framework, strongest architectural match
- [CrewAI](https://docs.crewai.com) — Multi-agent orchestration, unified memory with composite scoring
- [nanobot](https://github.com/HKUDS/nanobot) — Channel gateway pattern across 9+ platforms
- [Goose (Block)](https://github.com/block/goose) — MCP-native extension architecture
- [Pydantic AI](https://github.com/pydantic/pydantic-ai) — Typed dependency injection, durable execution
- [OpenHands](https://github.com/All-Hands-AI/OpenHands) — Sandbox execution, context compression
- Framework comparison matrix: `docs/research/2026-02-16-framework-comparison.md`
- OpenClaw feature audit (to be completed Week 1)
