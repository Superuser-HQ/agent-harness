# PRD: SHQ Agent Harness v1
**Status:** Proposed v1.0 (scope-cut)
**Author:** Kani (draft), Rem (review), gpt-5.3-codex (rewrite)
**Date:** 2026-02-17
**Team Assumption:** 2-3 engineers for 8 weeks
**Stakeholders:** Yao, Gerald

> For long-term vision and aspirational features, see [VISION.md](../VISION.md).
> For phasing beyond v1, see [ROADMAP.md](../ROADMAP.md).

---

## 1. Problem Statement

SHQ currently depends on OpenClaw for daily multi-agent operations, cross-surface messaging, and operational continuity. Platform uncertainty creates delivery and control risk.

We need a replacement harness we can operate, evolve, and trust, without trying to rebuild everything at once.

## 2. v1 Goal (8 Weeks)

Ship a production-usable harness for SHQ internal use with:
- Rust single-binary runtime
- Structured memory DB for runtime recall
- Repo-exported memory artifacts as canonical record
- Cortex limited to process supervision + health signals
- One primary messaging surface first, second surface only if reliability gates pass

**Non-goal:** broad feature parity with OpenClaw in 8 weeks.

## 3. Product Principles

1. **Harness-first**: scaffolding and feedback loops are the product.
2. **Compound engineering**: solved work becomes reusable system capability.
3. **Mechanical enforcement**: lint/tests/structure enforce behavior, not policy docs.
4. **Thin core**: keep core minimal; push variability to skills/extensions.
5. **Legibility with execution pragmatism**: runtime state can live in DB, but canonical history must be exported to repo.
6. **Distributed ownership**: no single maintainer bottleneck.

## 4. Hard v1 Scope

### 4.1 In Scope

- **Core runtime in Rust** — single binary, session tree model (main + branch sessions)
- **Base tools:** Read, Write, Edit, Shell, Message, Memory
- **One messaging surface integration** (primary)
- **Memory runtime:**
  - SQLite for typed memory + metadata
  - Vector index for semantic recall
  - Hybrid recall (vector + full-text)
- **Repo export pipeline:**
  - Deterministic export of decisions/events/identity memory to versioned files
  - Export on schedule and on explicit checkpoint events
- **Cortex v1:**
  - Worker/branch supervision (stuck process cleanup, retries, kill policies)
  - Health signals and alerts (queue depth, failure rate, memory sync lag, process liveness)
- **Mechanical enforcement baseline:**
  - Architecture boundary lint
  - Structural tests for memory export integrity
  - CI gate for required docs/artifact freshness
- **Compound capture loop:**
  - Minimal post-task capture template to `docs/solutions/`

### 4.2 Explicit v1 Cuts (Deferred)

See [ROADMAP.md](../ROADMAP.md) for phasing of deferred features.

- Cortex pattern mining, memory bulletins, admin chat interface
- Plugin marketplace/discovery layer
- Visual dashboard UI (CLI/log-based ops for v1)
- Advanced model routing tiers (keep simple policy-based routing only)
- Broad multi-surface rollout (second surface is optional behind gate)
- MCP server exposure for skills
- PM adapters (Linear, Jira sync)
- Container-level sandboxing
- Multi-agent RPC with cryptographic identity
- Full OpenClaw feature parity

## 5. Architecture Decisions (Locked)

1. **Runtime language:** Rust — single binary, zero deps, validated by Spacebot in same domain.
2. **Memory model:** structured DB + vector index for runtime; repo export as canonical record.
3. **Process model:** Channel/Branch/Worker/Cortex — concurrent specialized processes.
4. **Cortex scope:** v1 limited to supervision + health only.
5. **Core loop:** Build own, port patterns from pi-agent-core, Spacebot, Mastra, CrewAI.

All decisions accepted 2026-02-17.

## 6. Legibility Model

### Policy
- **DB is runtime cache/index and operational store.**
- **Repo export is canonical, reviewable, and portable record.**

### Enforcement
- Every canonical memory class (Decision, Identity, long-lived Fact, Goal state transitions) must export to repo.
- Export freshness SLO enforced in CI for release branches.
- Import/export determinism tests required (same DB snapshot → same exported artifacts).
- Recovery drill: rebuild fresh DB from canonical exports must pass before production cutover.

## 7. Reliability Targets and Milestones

Feature milestones alone are insufficient. v1 exits only if reliability targets are met.

### Reliability SLOs (v1)

| Metric | Target |
|--------|--------|
| Process crash-free runtime | ≥ 99.5% daily uptime in staging week |
| Stuck worker detection | ≤ 60s median, ≤ 5m max cleanup |
| Message handling success (primary surface) | ≥ 99% (excluding upstream outages) |
| Memory export lag (canonical classes) | ≤ 10 minutes p95 |
| Restart recovery time (cold start to healthy) | ≤ 3 minutes |

### Reliability Milestones

- **R1 (Week 3):** Supervised process restart and stuck-worker cleanup proven in staging
- **R2 (Week 5):** Failure injection runbook executed (process kill, DB lock, provider timeout)
- **R3 (Week 7):** 7-day soak test meets SLO thresholds
- **R4 (Week 8):** Rollback drill + restore drill passed in production-like environment

## 8. Delivery Plan (8 Weeks, 2-3 Engineers)

### Phase 0 (Week 1): Scope Lock + Foundations
- Finalize used-feature audit (only SHQ-used flows)
- Define canonical memory classes + export schema
- **Author ADR-0002: Export Schema Versioning** — schema version field in every export artifact, forward-compatible reader, CI test that old exports still import. Must be merged before first export lands. (Committed in [earlier thread](https://github.com/Superuser-HQ/superagents); tracked as non-optional.)
- Stand up Rust skeleton + CI + architecture lint scaffolding

**Gate G0 → G1:** Signed v1 scope (cuts accepted). Ownership assigned for runtime, memory, integration. ADR-0002 merged.

### Phase 1 (Weeks 2-3): Single-Agent Vertical Slice
- Main/branch sessions working
- Core tools wired (Read/Write/Edit/Shell/Message/Memory)
- Primary messaging surface connected
- Basic memory write/recall path functional
- Cortex supervision primitives implemented

**Gate G1 → G2:** End-to-end task success on primary surface. R1 reliability milestone passed.

### Phase 2 (Weeks 4-5): Memory Canonicalization + Enforcement
- Deterministic export pipeline live
- Import/export consistency tests
- Structural tests + CI freshness gates
- Minimal compound capture workflow in place
- Failure injection runbook + tests

**Gate G2 → G3:** Legibility policy enforced in CI. R2 reliability milestone passed.

### Phase 3 (Weeks 6-7): Multi-Agent Minimum + Soak
- Two-agent collaboration on shared artifacts
- Health signals and alerting tuned
- 7-day soak in staging/production-like setup
- Optional second surface only if no SLO regressions

**Gate G3 → G4:** R3 reliability milestone passed. No P0/P1 unresolved for 5 consecutive days.

### Phase 4 (Week 8): Migration + Cutover Safety
- Controlled migration of SHQ operations from OpenClaw
- Rollback rehearsal + restore rehearsal
- Production cutover with canary window

**Gate G4 → GA:** R4 milestone passed. Stakeholder sign-off on SLOs and rollback readiness.

## 9. Rollback Plan

If cutover degrades operations, rollback within same day.

### Triggers
- SLO breach sustained > 30 minutes
- Message delivery failure > 2% for > 15 minutes
- Data export lag > 30 minutes for canonical classes
- Unrecoverable crash loop > 10 minutes

### Steps
1. Freeze new harness write paths.
2. Switch routing back to OpenClaw for affected surfaces.
3. Snapshot runtime DB and logs for postmortem.
4. Replay missed canonical events from queue/log to avoid data loss.
5. Announce incident status and ETA.

### Recovery Requirement
- Run forward-fix in staging first.
- Re-attempt cutover only after failing scenario has automated regression coverage.

## 10. Cost Model (v1)

### Engineering
- 2-3 engineers × 8 weeks (primary cost center)

### Runtime/Infra (order-of-magnitude)
- **LLM usage:**
  - Channel/interactive tasks on higher-tier model
  - Worker/background tasks on lower-cost model
  - Budget guardrails: per-day token cap + per-task hard cap
- **Hosting:**
  - Single service instance + SQLite persistent storage + vector index storage
  - Logs/metrics backend (lightweight)

### Cost Controls in v1
- Model allowlist by process type
- Timeout + retry caps to prevent runaway loops
- Hard concurrency limits for workers
- Daily cost report from token/usage logs

## 11. Ownership Map

| Area | Primary | Backup |
|------|---------|--------|
| Runtime/Core (Rust loop, sessions, tools) | Kani | Rem |
| Memory + canonical export + determinism tests | Rem | Kani |
| Messaging integration + cutover + ops runbooks | Kani | Rem |
| Cortex supervision + health telemetry | Kani/Rem shared | — |
| Mechanical enforcement (lint, CI gates) | Kani/Rem shared | — |
| Product/acceptance sign-off | Yao + Gerald | — |

No subsystem may have single-point knowledge at release gate.

## 12. Success Criteria

| When | Criterion | Reliability Gate |
|------|-----------|-----------------|
| End of Week 3 | Single-agent vertical slice on primary surface | R1 passed |
| End of Week 5 | Deterministic canonical export enforced in CI | R2 passed |
| End of Week 7 | Two-agent minimum collaboration + 7-day soak meeting SLOs | R3 passed |
| End of Week 8 | Controlled migration complete, rollback/restore drills passed, OpenClaw dependency removed for in-scope workflows | R4 passed |

## 13. Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Rust delivery speed | Constrain scope, reuse libraries, avoid bespoke framework work |
| Memory complexity | Freeze canonical classes early; defer nonessential memory types |
| Integration instability | Primary surface first, second surface behind reliability gate |
| Overbuilding Cortex | Enforce v1 scope (supervision + health only) |

## 14. Testing Strategy (v1)

1. **Unit tests** — tool implementations, message formatting, memory read/write, vector index operations
2. **Integration tests** — messaging adapters (mock APIs), LLM abstraction (mock providers), session lifecycle
3. **Golden path E2E** — message → process → tools → response → memory update. One test per surface. Run on every PR.
4. **Replay tests** — record real sessions, replay against new code for regression detection
5. **Memory integrity** — vector index matches file content after writes, concurrent writes don't corrupt

Golden path E2E ships with the first prototype (Week 3).

## 15. Open Questions

- **Messaging adapter architecture?** Study nanobot's gateway pattern, then design.
- **Vector store choice?** Benchmark Rust-native options (lance, qdrant-client, usearch).

## 16. Document Sync Note

- `research/superagents/spacebot-analysis.md` is a **pre-decision snapshot** (2026-02-17), not a live spec.
- `docs/prd/2026-02-16-superagents-v1.md` (v1.8) is the original full-scope PRD — retained as reference.
- This document is the **decision-bearing document** for v1 scope and priorities.
