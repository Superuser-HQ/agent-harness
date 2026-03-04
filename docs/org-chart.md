# Agent Orchestration Org Chart

**Status:** Current design as of 2026-03-04 (v1 implementation + full vision)
**Sources:** PRD v2, VISION.md, ROADMAP.md, Spacebot analysis, `src/core/session.rs`, `src/cortex/mod.rs`

---

## 1. Process Hierarchy

The top-level view: who talks to whom, and who supervises whom.

```mermaid
graph TD
    Human(["👤 Human\n(Slack / Telegram / CLI)"])
    Cortex["🧠 Cortex\n(Supervisor)"]
    Channel["📡 Channel\n(User-facing ambassador)"]
    Branch1["🌿 Branch\n(Thinking fork)"]
    Branch2["🌿 Branch\n(Parallel task)"]
    Worker1["⚙️ Worker\n(File / Shell task)"]
    Worker2["⚙️ Worker\n(Research task)"]
    Memory[("💾 Memory\nLanceDB + SQLite")]
    RepoExport[["📁 Repo Export\n(canonical record)"]]

    Human -- "message" --> Channel
    Channel -- "spawn branch\n(preserves channel context)" --> Branch1
    Channel -- "spawn branch\n(parallel)" --> Branch2
    Branch1 -- "spawn worker\n(heavy task)" --> Worker1
    Branch2 -- "spawn worker\n(heavy task)" --> Worker2

    Branch1 -- "recall / save" --> Memory
    Branch2 -- "recall / save" --> Memory
    Worker1 -- "recall / save" --> Memory
    Worker2 -- "recall / save" --> Memory

    Branch1 -- "return result\nthen deleted" --> Channel
    Branch2 -- "return result\nthen deleted" --> Channel
    Worker1 -- "return result" --> Branch1
    Worker2 -- "return result" --> Branch2

    Channel -- "respond" --> Human

    Cortex -. "supervises all\nprocesses" .-> Channel
    Cortex -. "supervises" .-> Branch1
    Cortex -. "supervises" .-> Branch2
    Cortex -. "supervises" .-> Worker1
    Cortex -. "supervises" .-> Worker2
    Cortex -- "read/write" --> Memory
    Memory -- "scheduled export" --> RepoExport

    style Human fill:#f5f5f5,stroke:#999
    style Cortex fill:#ffe0b2,stroke:#e65100
    style Channel fill:#e3f2fd,stroke:#1565c0
    style Branch1 fill:#e8f5e9,stroke:#2e7d32
    style Branch2 fill:#e8f5e9,stroke:#2e7d32
    style Worker1 fill:#fce4ec,stroke:#880e4f
    style Worker2 fill:#fce4ec,stroke:#880e4f
    style Memory fill:#f3e5f5,stroke:#6a1b9a
    style RepoExport fill:#fffde7,stroke:#f57f17
```

---

## 2. Session Tree Model

Sessions are trees, not flat lists. Branches are scoped, pruned after use; audit trails are preserved.

```mermaid
graph TD
    Main["Main Session\n(persistent, interactive)"]
    B1["Branch Session A\n(sub-task: research)"]
    B2["Branch Session B\n(sub-task: code review)"]
    B3["Branch Session C\n(nested: security check)"]
    W1["Worker: shell exec"]
    W2["Worker: file read/write"]
    W3["Worker: lint"]

    Main --> B1
    Main --> B2
    B2 --> B3

    B1 --> W1
    B2 --> W2
    B3 --> W3

    B1 -. "result → pruned" .-> Main
    B2 -. "result → pruned" .-> Main
    B3 -. "result → pruned" .-> B2

    style Main fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    style B1 fill:#e8f5e9,stroke:#2e7d32
    style B2 fill:#e8f5e9,stroke:#2e7d32
    style B3 fill:#e8f5e9,stroke:#2e7d32
    style W1 fill:#fce4ec,stroke:#880e4f
    style W2 fill:#fce4ec,stroke:#880e4f
    style W3 fill:#fce4ec,stroke:#880e4f
```

**Key properties:**
- Main session is always responsive — it never does heavy work directly
- Branch sessions fork the Channel's context; Worker sessions start fresh (task prompt only)
- Branches are deleted after returning results; audit trails remain (per VISION.md: "every decision path is preserved")
- Nesting is allowed (Branch can spawn Branch), but depth should be kept shallow

---

## 3. Task Flow: End-to-End

A single user message from receipt to response.

```mermaid
sequenceDiagram
    actor Human
    participant Channel
    participant Branch
    participant Worker
    participant Memory
    participant Cortex

    Human->>Channel: send message
    Channel->>Channel: coalesce rapid messages
    Channel->>Branch: spawn (fork context)
    Branch->>Memory: recall relevant context
    Memory-->>Branch: facts, decisions, preferences

    alt heavy task
        Branch->>Worker: spawn worker
        Worker->>Worker: execute (shell/file/search)
        Worker->>Memory: save observations
        Worker-->>Branch: return result
    end

    Branch->>Memory: save decisions/facts
    Branch-->>Channel: return conclusion (then deleted)
    Channel-->>Human: respond

    Note over Cortex: supervising in background
    Cortex->>Cortex: check for stuck sessions (≤60s)
    Cortex->>Cortex: emit health signals
```

---

## 4. Human Intervention Points

Where humans can steer, approve, or stop.

```mermaid
flowchart LR
    subgraph "Always-On Controls"
        KillSwitch["🛑 Kill switch\n(any channel)"]
        TokenBudget["💰 Token budget\n(per-session cap)"]
    end

    subgraph "Approval Gates (Phase 1)"
        ElevatedOp["⚠️ Elevated action\nrequires explicit approval\n(system-wide, irreversible)"]
    end

    subgraph "Steering (Asynchronous)"
        MorningPriority["☀️ Morning: human\nprioritizes task queue"]
        PRReview["🔍 PR review:\nhuman merges or vetoes\narchitectural decisions"]
        ADRApproval["📋 ADR approval:\nhuman signs off\non architecture decisions"]
    end

    subgraph "Deferred Approval (Phase 2)"
        NonBlockingGate["⏸️ Agent continues\nnon-blocked work\nwhile awaiting approval"]
    end

    Human(["👤 Human"]) --> KillSwitch
    Human --> ElevatedOp
    Human --> MorningPriority
    Human --> PRReview
    Human --> ADRApproval

    KillSwitch --> Channel["📡 Channel"]
    ElevatedOp --> Worker["⚙️ Worker"]
    MorningPriority --> Channel
    TokenBudget -. "caps tokens per\nsession / task / day" .-> Channel
    TokenBudget -. "caps tokens per\nsession / task / day" .-> Worker

    style KillSwitch fill:#ffcdd2,stroke:#b71c1c
    style ElevatedOp fill:#fff9c4,stroke:#f57f17
    style TokenBudget fill:#fff9c4,stroke:#f57f17
    style Human fill:#f5f5f5,stroke:#999
```

### Intervention Tiers (from VISION.md)

| Tier | Scope | Human action required |
|------|-------|----------------------|
| Read-only | Observe, never mutate | None |
| Workspace-scoped | Mutate within workspace | None (default allowed) |
| System-wide | Mutate outside workspace | Approval |
| Elevated | Irreversible / high-impact | Explicit human approval |

---

## 5. Multi-Agent Architecture (Phase 2)

When multiple agents collaborate — two communication channels by design.

```mermaid
graph TD
    subgraph "Agent A"
        ChannelA["📡 Channel A"]
        BranchA["🌿 Branch A"]
    end

    subgraph "Agent B"
        ChannelB["📡 Channel B"]
        BranchB["🌿 Branch B"]
    end

    subgraph "Shared Infrastructure"
        RPCLayer["🔗 RPC Layer\n(typed payloads,\ncryptographic identity)"]
        CoordLayer["💬 Coordination Layer\n(messaging surface,\nhuman-visible)"]
        SharedMemory[("💾 Shared Memory\n(team-scoped)")]
    end

    Human(["👤 Human"])

    ChannelA -- "task handoff" --> RPCLayer
    RPCLayer -- "structured result" --> ChannelB
    ChannelA -- "status / decisions" --> CoordLayer
    ChannelB -- "status / decisions" --> CoordLayer
    CoordLayer -- "visible to" --> Human

    BranchA -- "read/write" --> SharedMemory
    BranchB -- "read/write" --> SharedMemory

    style RPCLayer fill:#e8eaf6,stroke:#3949ab
    style CoordLayer fill:#e0f2f1,stroke:#00695c
    style SharedMemory fill:#f3e5f5,stroke:#6a1b9a
    style Human fill:#f5f5f5,stroke:#999
```

**Trust model:** Each agent trusts its primary human fully. Agents treat other agents as collaborators, not authorities — agent-to-agent messages do not bypass human approval gates for elevated actions.

---

## 6. Memory Flow

How knowledge moves from runtime to canonical record.

```mermaid
flowchart LR
    subgraph "Runtime (DB)"
        VectorStore[("LanceDB\nvector embeddings")]
        SQLite[("SQLite\ntyped metadata\ngraph edges")]
    end

    subgraph "Recall"
        Hybrid["Hybrid Recall\nvector similarity\n+ full-text\n(RRF merge)"]
    end

    subgraph "Canonical Record (Repo)"
        GitExport["📁 docs/ export\n(decisions, identity,\nlong-lived facts)"]
        CI["CI freshness gate\n(≤10 min p95 lag)"]
    end

    Branch["🌿 Branch / Worker"] -- "save" --> VectorStore
    Branch -- "save" --> SQLite
    VectorStore --> Hybrid
    SQLite --> Hybrid
    Hybrid -- "inject into\nbranch context" --> Branch

    SQLite -- "scheduled export\n(canonical classes)" --> GitExport
    GitExport --> CI

    style VectorStore fill:#f3e5f5,stroke:#6a1b9a
    style SQLite fill:#f3e5f5,stroke:#6a1b9a
    style GitExport fill:#fffde7,stroke:#f57f17
    style CI fill:#e8f5e9,stroke:#2e7d32
```

**Canonical memory classes** (always exported to repo): Decision, Identity, long-lived Fact, Goal state transitions.

**Export SLO:** p95 lag ≤ 10 minutes, enforced in CI on release branches.

---

## 7. Cortex Supervision (v1 Scope)

Cortex sees across all processes. v1 scope is deliberately narrow.

```mermaid
flowchart TD
    Cortex["🧠 Cortex Supervisor"]

    subgraph "v1 (Implemented)"
        StuckDetect["Stuck worker detection\n≤60s median\n≤5m max cleanup"]
        HealthSignals["Health signals\nqueue depth, failure rate,\nmemory sync lag, liveness"]
        RetryPolicy["Retry / kill policy\nper process type"]
    end

    subgraph "Phase 2+ (Deferred)"
        PatternMining["Pattern mining\n(auto-generate solutions)"]
        MemBulletins["Memory bulletins\n(proactive cross-agent sharing)"]
        AdminChat["Admin chat interface"]
        AnomalyDetect["Anomaly detection\n& security monitoring"]
    end

    Cortex --> StuckDetect
    Cortex --> HealthSignals
    Cortex --> RetryPolicy

    Cortex -. "future" .-> PatternMining
    Cortex -. "future" .-> MemBulletins
    Cortex -. "future" .-> AdminChat
    Cortex -. "future" .-> AnomalyDetect

    style Cortex fill:#ffe0b2,stroke:#e65100,stroke-width:2px
    style StuckDetect fill:#e8f5e9,stroke:#2e7d32
    style HealthSignals fill:#e8f5e9,stroke:#2e7d32
    style RetryPolicy fill:#e8f5e9,stroke:#2e7d32
    style PatternMining fill:#f5f5f5,stroke:#bbb,stroke-dasharray:4
    style MemBulletins fill:#f5f5f5,stroke:#bbb,stroke-dasharray:4
    style AdminChat fill:#f5f5f5,stroke:#bbb,stroke-dasharray:4
    style AnomalyDetect fill:#f5f5f5,stroke:#bbb,stroke-dasharray:4
```

---

## 8. Model Routing by Process Type

Different processes get different model tiers (cost optimization).

```mermaid
flowchart LR
    Channel["📡 Channel"] --> HighTier["High-tier model\n(best quality,\nconversation-facing)"]
    Branch["🌿 Branch"] --> HighTier
    Worker["⚙️ Worker\n(coding)"] --> HighTier
    Worker2["⚙️ Worker\n(summarization\nbackground)"] --> LowTier["Low-cost model\n(cheap, sufficient)"]
    Compactor["📦 Compactor\n(context summary)"] --> LowTier

    style HighTier fill:#e3f2fd,stroke:#1565c0
    style LowTier fill:#f5f5f5,stroke:#999
```

---

## Summary: Who Spawns What

| Actor | Can spawn | Can terminate¹ | Supervises |
|-------|-----------|----------------|------------|
| Human | — | Any (kill switch) | Steers via approval gates |
| Cortex | — | Stuck workers/branches (forceful) | All processes |
| Channel | Branch | Own branches (lifecycle cleanup) | — |
| Branch | Worker, nested Branch | Own workers (lifecycle cleanup) | — |
| Worker | — | — | — |

¹ "Terminate" has two meanings here: Cortex **forcefully kills** stuck processes that exceed the timeout threshold (active intervention). Channel and Branch **clean up** their children after those children return results — this is normal lifecycle completion, not a kill action. The kill switch available to humans bypasses all of this and terminates any process immediately.
