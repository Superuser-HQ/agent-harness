# Roadmap: SHQ Agent Harness

> Bridge between [VISION.md](VISION.md) (where we're going) and the [v1 PRD](prd/2026-02-17-agent-harness-v2.md) (what we're building now).

---

## Phase 1: v1 — Foundation (Weeks 1-8)

**Goal:** Production-usable harness for SHQ internal use. Replace OpenClaw dependency.

### Core Deliverables
- Rust single-binary runtime with session tree model
- Base tools: Read, Write, Edit, Shell, Message, Memory
- One primary messaging surface integration
- Memory: SQLite + vector index, hybrid recall (vector + full-text)
- Deterministic repo export pipeline for canonical memory
- Cortex v1: process supervision + health signals only
- Mechanical enforcement: architecture lint, structural tests, CI freshness gates
- Compound capture loop (post-task template to `docs/solutions/`)
- Context compression (log + context split, compaction)
- Validation-retry loop for tool arguments
- Typed dependency injection for tools
- Golden path E2E tests, replay tests
- Controlled migration from OpenClaw + rollback plan

### Reliability Gates
- R1 (Week 3): Supervised restart + stuck-worker cleanup
- R2 (Week 5): Failure injection runbook executed
- R3 (Week 7): 7-day soak meets SLOs
- R4 (Week 8): Rollback + restore drills passed

---

## Phase 2: Multi-Surface + Multi-Agent (Weeks 9-16, estimated)

**Goal:** Channel abstraction proven across multiple surfaces. Multi-agent collaboration is robust.

### Deliverables
- **Second and third messaging surfaces** (e.g., Telegram, Discord)
- **Channel abstraction layer** — unified API, surface-specific formatting, channel policies
- **Multi-agent RPC** — structured agent-to-agent communication with cryptographic identity
- **Coordination layer** — human-visible agent collaboration via messaging surfaces
- **MCP compatibility** — skills exposable as MCP servers for ecosystem interop
- **Deferred approval pattern** — non-blocking approval gates (agent continues other work)
- **Action/Observation formalism** — structured execution model for audit + replay
- **Custom agent distributions** — pre-configured agent templates (`--template research`, etc.)
- **Remote skill discovery** — curated registry, `harness skill add <url>`
- **Container-level sandboxing** — Docker isolation with minimal capabilities, network egress controls
- **OpenTelemetry integration** — connect v1 instrumentation hooks to collectors
- **Lightweight web dashboard** — read-only view of agent status, activity, costs
- **PM adapter: GitHub** — one-way sync from Backlog.md to GitHub Issues/Projects

### From v1.8 PRD (§7, §8, §9)
- Agent governance model (humans steer, agents contribute)
- Specialized agent roles (reviewer, research, maintenance)
- Repo-local task management (Backlog.md) + PM adapter pattern
- Decision capture via emoji trigger / CLI

---

## Phase 3: Platform Maturity (Weeks 17-24, estimated)

**Goal:** Full-featured platform. Cortex intelligence. Broad surface coverage.

### Deliverables
- **Cortex pattern mining** — detect recurring problems, auto-generate solutions
- **Cortex memory bulletins** — proactive knowledge sharing across agents
- **Cortex admin chat interface** — human-readable Cortex interaction
- **Advanced model routing** — heuristic-based tier selection beyond simple policy
- **PM adapters: Linear + Jira** — bidirectional sync (repo ↔ external)
- **Workflow engine with suspend/resume** — durable multi-step workflows spanning hours/days
- **Network egress allowlist** — operator-configured, agent-unmodifiable
- **Security posture automation** — periodic permission audits, anomaly detection, incident response playbook
- **Cross-agent observability** — interaction patterns, bottleneck detection, team-wide cost profiles
- **Benchmark suite** — 10-15 standard scenarios run on every PR (SWE-bench equivalent)

### From v1.8 PRD (§6, §13, §14)
- Full security threat model implementation (prompt injection defense, supply chain vetting)
- Extension vetting tiers (built-in → allowlisted → untrusted)
- Human interaction UX (onboarding, team dynamics, error communication protocol)
- Graduated urgency levels for notifications

---

## Phase 4: Ecosystem (Beyond Week 24)

**Goal:** Open ecosystem. Distributed deployment. Community growth.

### Deliverables
- **Plugin marketplace / skill registry** — discovery beyond manual git URLs
- **Visual agent builder** — optional drag-and-drop for non-technical users
- **Distributed multi-machine deployment** — agents across multiple hosts
- **Foundation governance** — if traction warrants, transition from SHQ-owned
- **Broad surface matrix** — IRC, Email, WhatsApp, QQ, Feishu, DingTalk, etc.
- **Voice interaction layer** — TTS/STT, voice notes, audio briefings
- **Mobile-optimized experience** — concise responses, progressive disclosure

---

## Deferred Backlog (Unscheduled)

Items identified during planning that don't yet have a phase assignment:

- Full OpenClaw feature parity (may never be a goal — build what we need)
- Visual dashboard with write capabilities (Phase 2 dashboard is read-only)
- Agent self-modification with auto-approval (currently requires PR review)
- Cross-team channel behavior policies
- Billing/metering for multi-tenant deployments

---

## Key Dependencies Between Phases

```
Phase 1 (v1)
  └─ Rust runtime, memory, one surface, Cortex supervision
       │
Phase 2
  ├─ Channel abstraction (builds on Phase 1 messaging)
  ├─ Multi-agent RPC (builds on Phase 1 session model)
  ├─ Container sandboxing (builds on Phase 1 filesystem boundaries)
  └─ MCP compatibility (builds on Phase 1 skill system)
       │
Phase 3
  ├─ Cortex intelligence (builds on Phase 2 multi-agent data)
  ├─ Workflow engine (builds on Phase 2 suspend/resume patterns)
  └─ Bidirectional PM sync (builds on Phase 2 one-way sync)
       │
Phase 4
  └─ Ecosystem (builds on Phase 3 maturity + stability)
```

---

*Last updated: 2026-02-18*
