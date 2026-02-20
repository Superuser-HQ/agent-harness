# ADR-0002: Session Tree Model

**Status:** Proposed  
**Date:** 2026-02-20  
**Authors:** Rem  
**Deciders:** Yao, Gerald, Kani, Rem

---

## Context

The PRD v1 (§4.1) specifies a "session tree model" for the runtime — main sessions plus branch sessions. VISION.md elaborates: sessions are tree-structured, not flat history. Agents branch for sub-tasks without polluting the main context.

This ADR defines the session model concretely: what a session is, how sessions relate to each other, lifecycle semantics, and the constraints that the runtime must enforce.

### Current state (OpenClaw)

OpenClaw has a working session model with:
- **Main session** — persistent, tied to a channel (Telegram DM, Slack channel)
- **Spawned sessions** — `sessions_spawn` creates isolated sub-agent runs with their own context
- **Compaction** — when context grows too large, the system compresses history and optionally flushes a memory summary to disk
- **No formal tree** — sessions reference a parent loosely but there's no structured tree, no depth limits, no propagation semantics

This works for single-agent use. It breaks down when:
- Multiple agents need to coordinate on shared state
- A branch needs to report back structured results (not just a text summary)
- Branches spawn their own branches (unbounded depth)
- Context needs to be shared selectively between siblings

---

## Decision

**Sessions form a rooted tree with typed edges, bounded depth, and explicit lifecycle states.**

### Session identity

Every session has:
- `session_id: UUID` — unique, immutable
- `agent_id: AgentId` — which agent owns this session
- `parent_id: Option<UUID>` — null only for root sessions
- `kind: SessionKind` — one of `Main`, `Branch`, `Worker`
- `state: SessionState` — one of `Active`, `Suspended`, `Completed`, `Failed`, `Expired`
- `created_at: Timestamp`
- `ttl: Option<Duration>` — max wallclock lifetime (enforced by Cortex)
- `context: SessionContext` — the LLM conversation state (messages, tool results, system prompt)

### Session kinds

| Kind | Purpose | Has identity/soul? | Sees parent context? | Max depth |
|------|---------|-------------------|---------------------|-----------|
| **Main** | Primary human conversation | Yes | N/A (root) | 0 (root) |
| **Branch** | Sub-task forked from Main or another Branch | Yes (inherits) | Summary only | 3 |
| **Worker** | Stateless task execution | No | No | 1 (leaf only) |

- **Main** sessions are long-lived. One per agent per channel. They survive restarts (state persisted to LanceDB + canonical export).
- **Branch** sessions fork from a parent with a task description and optional context summary. They return a structured result to the parent on completion. Branches can spawn sub-branches up to depth 3.
- **Worker** sessions are fire-and-forget. No personality, no memory writes, no sub-spawning. Used for pure computation (summarization, formatting, search). Always leaf nodes.

### Tree constraints

1. **Max depth: 4** (Main → Branch → Branch → Branch → Worker). Enforced at spawn time. Spawn requests exceeding depth are rejected with an error.
2. **Max children per session: 8**. Prevents fork bombs. Configurable per-agent.
3. **Workers cannot spawn children.** Workers are always leaves.
4. **Orphan cleanup:** If a parent transitions to `Completed`, `Failed`, or `Expired`, all active children are sent a cancellation signal. Children have 30s to wrap up before forced termination. Cortex enforces this.

### Context propagation

Context does NOT flow automatically between sessions. This is deliberate — unbounded context sharing defeats the purpose of branching.

| Direction | Mechanism |
|-----------|-----------|
| Parent → Child (at spawn) | Explicit `context_summary: String` provided by parent. No automatic history forwarding. |
| Child → Parent (on completion) | Structured `SessionResult { status, summary, artifacts, memory_ids }`. Parent decides what to incorporate. |
| Sibling → Sibling | Not allowed directly. Siblings communicate through shared memory (LanceDB) or by routing through the parent. |

### Lifecycle

```
                  spawn()
    ┌──────────────────────────────┐
    │                              ▼
 [Parent]                      [Active]
    │                           │    │
    │              complete()   │    │  fail() / expire()
    │                    ┌──────┘    └──────┐
    │                    ▼                  ▼
    │              [Completed]          [Failed/Expired]
    │                    │                  │
    │                    ▼                  ▼
    │              result sent         error sent
    │              to parent           to parent
    │                    │                  │
    └────────────────────┴──────────────────┘
                         │
                    children cleaned up
```

- **Suspend/Resume** is supported for Branch sessions only. A suspended branch retains its context but yields its compute slot. Cortex can suspend branches under memory pressure.
- **TTL expiry** is the primary garbage collection mechanism. Default TTLs: Branch = 30 min, Worker = 5 min. Configurable per-spawn.

### Persistence

- **Main sessions:** Full context persisted to LanceDB on every message cycle. Canonical export on schedule (per ADR-0005).
- **Branch sessions:** Context persisted only on suspend or completion. Intermediate state is in-memory only (acceptable loss on crash — branches are retryable).
- **Worker sessions:** No persistence. Ephemeral by design.

On restart, the runtime rebuilds the session tree from LanceDB:
1. Load all Main sessions (one per agent-channel pair)
2. Load any Suspended branches
3. Discard all other sessions (Active branches/workers from before crash are marked Failed, parent notified)

---

## Rationale

### Why trees, not flat sessions?

Flat session history is the #1 scalability problem in OpenClaw. A single long conversation accumulates context from unrelated sub-tasks, leading to:
- Context window exhaustion requiring aggressive compaction
- Compaction losing relevant context from earlier tasks
- Sub-task failures polluting the main conversation
- No way to isolate experimental or risky work

Trees solve all four: branch, do work, return result, discard branch context.

### Why bounded depth?

Unbounded recursion is a footgun. If an agent can spawn branches that spawn branches indefinitely:
- Resource exhaustion (memory, API calls)
- Debugging becomes impossible (which branch spawned which?)
- Cortex supervision complexity explodes

Depth 4 is sufficient for: Main → research branch → sub-research → worker. If a task needs deeper nesting, it should be restructured.

### Why no automatic context propagation?

Every token of context costs money and attention. Automatic propagation means:
- Children inherit irrelevant parent context (waste)
- Parents get polluted with child internals (noise)
- Cost scales multiplicatively with tree size

Explicit summaries force the spawning agent to think about what context is actually needed. This is how humans delegate — you give someone a brief, not your entire email history.

### Why structured results instead of text?

OpenClaw's `sessions_spawn` returns a text summary. This loses structure:
- Was the task successful or did it fail gracefully?
- What files were created/modified?
- What memory was written?

`SessionResult` preserves this. The parent can make informed decisions about what to do next.

---

## Alternatives Considered

| Option | Verdict |
|--------|---------|
| **Flat sessions (OpenClaw model)** | ❌ Context pollution, no isolation, compaction-dependent |
| **DAG (directed acyclic graph)** | ❌ Unnecessary complexity — multiple parents create ambiguous ownership. Trees are sufficient. |
| **Unbounded trees** | ❌ Resource exhaustion, supervision complexity |
| **Automatic context sharing** | ❌ Cost explosion, defeats purpose of branching |
| **Message-passing between siblings** | ❌ Creates hidden coupling. Shared memory (LanceDB) is the coordination layer. |

---

## Consequences

### What this enables
- Clean sub-task isolation without context pollution
- Predictable resource bounds (depth × children caps)
- Crash recovery with minimal state loss (only in-flight branches lost)
- Cortex can reason about the tree structure for supervision (stuck branches, orphans, resource hogs)

### What this requires
- Runtime must maintain an in-memory tree index (lightweight — just IDs and states)
- Spawn API must validate depth and children limits before creating sessions
- Cortex must implement orphan cleanup and TTL enforcement
- LLM abstraction must support creating new conversation contexts from a summary string

### Open questions
- **Branch priority:** Should branches inherit parent's model tier, or should workers always use cheaper models? (Leaning toward: configurable per-spawn, default to parent's tier for branches, cheapest tier for workers.)
- **Cross-agent branches:** Can Agent A spawn a branch owned by Agent B? (Leaning toward: not in v1. Cross-agent coordination goes through shared memory or RPC layer in Phase 2.)

---

## References

- PRD v1 §4.1 (session tree model)
- VISION.md §Session Model
- Spacebot process model (Channel/Branch/Worker/Cortex)
- OpenClaw `sessions_spawn` implementation (prior art)
