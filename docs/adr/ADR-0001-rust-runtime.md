# ADR-0001: Rust as Runtime Language

**Status:** Accepted  
**Date:** 2026-02-16 (decided), 2026-02-20 (documented)  
**Author:** Kani  
**Deciders:** Yao, Rem, Kani, Gerald  

---

## Context

Superagents needs a runtime language for the core agent loop, session tree, memory integration, and tool execution. The decision was made early in the design process (PRD v1.7) and informed by the Spacebot architecture study (`docs/research/2026-02-17-spacebot-analysis.md`).

Key constraints:
- **Single binary deployment** — no install-time runtime dependencies (no `npm install`, no `pip install`, no Python or Node on the host)
- **Low memory footprint** — agents run persistently, potentially many concurrent sessions
- **Embedded memory store** — LanceDB (ADR-0003) is a Rust crate; the runtime must be Rust to use it natively without FFI overhead
- **Correctness** — multi-agent concurrent memory writes require strong concurrency guarantees
- **Team size** — 2-3 engineers for 8 weeks; no room for runtime/ecosystem debugging

---

## Decision

**Use Rust as the primary runtime language.**

The single binary, LanceDB native crate dependency, and memory safety guarantees make Rust the correct choice for this domain.

---

## Rationale

### Single-binary deployment story

The primary operational advantage of Rust: `cargo build --release` produces a single statically-linked binary with no host runtime dependency. Deployment is `scp binary host:` or `docker copy`.

No equivalent in Python (`uvicorn` + venv + system libs) or TypeScript (`node` + `node_modules` + potential native addons).

The Spacebot project (Spacedrive's production AI integration) uses Rust for exactly this reason. Their analysis is the clearest prior art in the agent domain.

### LanceDB native integration

ADR-0003 locked LanceDB as the memory store. LanceDB's primary interface is a Rust crate. Alternatives:
- Python: via PyLance (wrapper, overhead, extra runtime)
- TypeScript: no official LanceDB Rust-to-TS bridge; would require subprocess or HTTP server

Using Rust means LanceDB is a direct `Cargo.toml` dependency with no IPC boundary between the agent loop and the memory store.

### Memory safety in multi-agent concurrency

Multi-agent writes to a shared memory store are the primary source of correctness risk. Rust's ownership model makes data races a compile-time error rather than a runtime bug. This is not possible in Python or TypeScript.

### What we give up

- **Iteration speed** — Rust compilation is slower than Python/TS for exploratory changes. Mitigation: constrain scope, don't build bespoke abstractions, use proven crates.
- **Contributor familiarity** — fewer engineers know Rust vs Python/TS. Mitigation: good module boundaries so non-core contributors work at the tool/skill layer (potentially in WASM or subprocess).
- **Ecosystem breadth** — Python has more ML/AI libraries. Not relevant for v1 scope (no model training, no embedding model — defer to LanceDB's built-in embedding or an external API).

---

## Alternatives Considered

| Language | Verdict | Reason rejected |
|----------|---------|-----------------|
| **TypeScript (Node.js)** | ❌ | Requires Node runtime on host; LanceDB integration via subprocess; no compile-time concurrency safety |
| **Python** | ❌ | Requires Python + venv; GIL limits true multi-agent concurrency; deployment complexity |
| **Go** | ❌ | No LanceDB native crate; good single-binary story but loses the memory store integration advantage |
| **Rust** | ✅ | Single binary; LanceDB native; memory safety; matches Spacebot precedent |

---

## Consequences

- **Cargo.toml** is the project manifest; `cargo build --release` is the deployment artifact
- **Skills/tools written by external contributors** may use subprocess or WASM bridge to avoid requiring Rust knowledge (not in v1 scope but the architecture supports it via the `Tool` trait)
- **CI must install Rust toolchain** (handled in `.github/workflows/ci.yml` via `dtolnay/rust-toolchain@stable`)
- **Iteration risk acknowledged** — if a component is not stable by its phase gate deadline, the mitigation is scope cut, not language change

---

## References

- `docs/research/2026-02-17-spacebot-analysis.md` — Spacebot (Spacedrive) Rust agent architecture
- ADR-0003 — LanceDB as memory store (Rust crate dependency)
- PRD v2 §5 (Architecture Decisions): "Runtime language: Rust"
