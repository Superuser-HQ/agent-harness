# PRD Review: SHQ Agent Harness v1.7

## Verdict
The v1.7 direction (Rust + structured memory + Channel/Branch/Worker/Cortex) is stronger than prior versions, but the document is still internally inconsistent and over-scoped. Right now it reads like two PRDs merged without full reconciliation: old file-first/OpenClaw conventions and new process+DB architecture.

## Findings (Ordered by Severity)

### Critical 1: Rust decision is not fully propagated; TypeScript-era artifacts remain in core decision machinery
- `docs/prd/2026-02-16-agent-harness.md:500` ADR example still says `TypeScript as primary language`.
- `docs/prd/2026-02-16-agent-harness.md:772` references Zod/TypeBox in a claim about existing PRD behavior, but v1.7 now frames validation as Rust + serde (`docs/prd/2026-02-16-agent-harness.md:108`).
- `docs/prd/2026-02-16-agent-harness.md:753` still schedules a “Mastra dependency decision” despite “build own core in Rust” being accepted (`docs/prd/2026-02-16-agent-harness.md:683-721`).

Why this is bad:
- These are not cosmetic. They pollute ADR and implementation priorities with stale assumptions and will cause roadmap churn.

Fix:
- Rewrite ADR sample to a Rust-era ADR.
- Remove Zod/TypeBox references or explicitly mark as historical note.
- Replace “Mastra dependency decision” with concrete Rust decisions (runtime, async model, DB/vector backend, process supervisor API).

### Critical 2: Process model and guardrails/security model are not operationally aligned
- Process architecture says channel never blocks and delegates (`docs/prd/2026-02-16-agent-harness.md:188-196`), but approval gates are defined as hard blocking (`docs/prd/2026-02-16-agent-harness.md:258`). No non-blocking deferred-approval design in core sections.
- Branches are “deleted after returning results” (`docs/prd/2026-02-16-agent-harness.md:190`) while audit/replay requirements demand detailed traceability (`docs/prd/2026-02-16-agent-harness.md:262`, `docs/prd/2026-02-16-agent-harness.md:740-742`). Deletion semantics vs forensic/replay retention are unresolved.
- Kill switch says “kills current session and queues no follow-up work” (`docs/prd/2026-02-16-agent-harness.md:260`), but no explicit cascade behavior for branch/worker trees or in-flight worker side effects.
- Multi-agent primitives are marked Phase 2 (`docs/prd/2026-02-16-agent-harness.md:399`) while success requires two-agent collaboration in week 4 (`docs/prd/2026-02-16-agent-harness.md:672`, `docs/prd/2026-02-16-agent-harness.md:755`).

Why this is bad:
- You cannot safely operate Channel/Branch/Worker without exact gate propagation, cancellation, and retention semantics. This is a security and correctness gap, not a polish gap.

Fix:
- Add a formal execution-state model: `pending_approval`, `running`, `canceled`, `completed`, `compensated`.
- Define kill-switch propagation and idempotent cancellation rules per process type.
- Specify branch retention policy for audit/replay (e.g., compacted logs retained N days even if branch object deleted).
- Either move multi-agent into v1 timeline or remove week-4 multi-agent milestone.

### Critical 3: Memory architecture is split-brain (DB-first in architecture, file-first in policy/guardrails/UX)
- Architecture says DB is operational store and repo exports are canonical artifacts (`docs/prd/2026-02-16-agent-harness.md:126-131`).
- Policy and guardrails still enforce legacy `MEMORY.md`/daily-log mechanics as if they are primary state (`docs/prd/2026-02-16-agent-harness.md:154-166`, `docs/prd/2026-02-16-agent-harness.md:278-281`, `docs/prd/2026-02-16-agent-harness.md:576`).
- Observability references FAISS (`docs/prd/2026-02-16-agent-harness.md:621`) while vector backend is unresolved and Rust options are listed separately (`docs/prd/2026-02-16-agent-harness.md:730`).
- Success metric still says “vector search over memory files” (`docs/prd/2026-02-16-agent-harness.md:677`) instead of over memory graph/exports.

Why this is bad:
- This will produce duplicated writes, inconsistent recall, and unclear source-of-truth ownership.

Fix:
- Define memory classes by storage authority:
  - Runtime truth: SQLite + vector index.
  - Canonical exports: only selected classes (Decision, Identity, long-lived Fact, Goal transitions).
  - Legacy files: optional views/generated artifacts, not authoritative.
- Replace MEMORY.md size-lint with export-size and schema-conformance linting.
- Remove FAISS mention unless chosen.

### High 4: Security claims and v1 delivery scope conflict
- v1 security relies on filesystem boundaries (`docs/prd/2026-02-16-agent-harness.md:371`) while high-assurance controls (container sandboxing, stronger extension isolation) are deferred.
- Yet timeline expects full migration by week 8 (`docs/prd/2026-02-16-agent-harness.md:674`) and “feature parity” by week 6 (`docs/prd/2026-02-16-agent-harness.md:673`).
- Network egress deny-by-default is specified (`docs/prd/2026-02-16-agent-harness.md:375`) but there is no concrete v1 allowlist lifecycle/ops model despite messaging + multi-provider + adapters requiring substantial outbound traffic.

Why this is bad:
- The migration target assumes production posture while key controls are not in v1 or not operationalized.

Fix:
- Define two explicit release bars:
  - `v1-internal`: trusted operators only, reduced threat model.
  - `v1-production`: requires egress policy management, extension trust workflow, and verified audit immutability.
- Tie migration milestone to `v1-production`, not to week count alone.

### High 5: Timeline is not credible for Rust + process concurrency + safety + UX in 8-9 weeks
- Week 3 requires prototype + policy linter (`docs/prd/2026-02-16-agent-harness.md:754`) while testing section also requires golden-path E2E at week 3 (`docs/prd/2026-02-16-agent-harness.md:744`).
- By week 4 it adds multi-agent collaboration + decision capture + external PM integration (`docs/prd/2026-02-16-agent-harness.md:755`).
- By week 6-7 it targets feature parity plus second messaging surface (`docs/prd/2026-02-16-agent-harness.md:757`) despite unresolved vector backend decision (`docs/prd/2026-02-16-agent-harness.md:730`) and multiple guardrail/security subsystems.

Why this is bad:
- This schedule ignores integration tax and reliability hardening in a brand new Rust core.

Fix:
- Rebase timeline around 3 gates:
  1. Core reliability gate (single-surface stable, replayable, cancellable)
  2. Safety gate (approval/cancellation/audit/egress proven)
  3. Migration gate (OpenClaw feature subset validated in shadow mode)
- Expect 12-16 weeks unless scope is cut.

### Medium 6: Redundant/conflicting sections after v1.7 merge
- Broken section numbering indicates stale edits: `## 7` contains `### 6.1/6.2/6.3` (`docs/prd/2026-02-16-agent-harness.md:399-423`), `## 9` contains `### 8.1` (`docs/prd/2026-02-16-agent-harness.md:448`).
- “No Notion/Jira/external trackers” (`docs/prd/2026-02-16-agent-harness.md:665`) conflicts with explicit Linear/Jira adapters (`docs/prd/2026-02-16-agent-harness.md:471-476`).
- “Background parallel bash omitted in v1” (`docs/prd/2026-02-16-agent-harness.md:664`) conflicts in spirit with concurrent worker-heavy process model.
- Audit log described as append-only and in repo (`docs/prd/2026-02-16-agent-harness.md:262`) and also as outside workspace for tamper resistance (`docs/prd/2026-02-16-agent-harness.md:377`) without one canonical storage strategy.

Why this is bad:
- These inconsistencies will fragment implementation and reviews.

Fix:
- Run a full consistency pass with a “single source of truth” table for: storage, phase ownership, and v1/v2 boundaries.

## Focused Answers to Requested Areas

1. Internal contradictions after Rust/DB/Cortex updates:
- Yes, several TypeScript/Node leftovers remain and directly conflict with accepted Rust decisions (ADR template, Zod/TypeBox mention, Mastra dependency milestone).

2. Process model vs guardrails/security gaps:
- Major gap. No formal semantics for approval blocking, cancellation propagation, worker side-effect compensation, and branch retention under audit/replay constraints.

3. Memory layer conflicts:
- Yes. DB-first architecture conflicts with file-first policy/guardrails/UX language and success metrics.

4. Timeline realism:
- Not realistic at current scope. Safety + observability + process concurrency + multi-surface adapters in Rust is under-scoped by multiple weeks.

5. Redundant/conflicting after v1.7:
- Multiple stale or contradictory sections remain (numbering, PM tool policy, audit storage, legacy memory phrasing).

## Recommended Immediate PRD Edits (Before Implementation)
1. Freeze v1 scope to one messaging surface, one worker type, one vector backend.
2. Publish one-page process state machine (Channel/Branch/Worker/Cortex + approval + cancellation + kill switch).
3. Rewrite memory policy around DB authority and deterministic export contracts.
4. Replace week-based promises with gate-based readiness criteria.
5. Clean stale Rust/TS contradictions and section numbering in one dedicated editorial PR.
