# CLAUDE.md — superagents

Agent context for Claude Code. Read this before touching anything.

## What This Is

A multi-agent harness built in Rust. The goal: replace OpenClaw with a single-binary, channel-agnostic runtime where memory is an architectural layer, not an afterthought. The key differentiator is channel abstraction — no existing framework has it.

This is Superuser HQ's keystone internal project. Quality matters here.

## Repo Structure

```
src/
  main.rs          — CLI entrypoint (clap, two subcommands: start, health)
  lib.rs           — library root
  core/            — session tree, agent lifecycle
  cortex/          — agent reasoning loop (not yet implemented)
  memory/          — LanceDB-backed memory store (record, store)
  messaging/       — channel abstraction layer (not yet implemented)
  tools/           — tool registry (not yet implemented)
docs/
  prd/             — PRD v2 (2026-02-17) — source of truth for requirements
  adr/             — architectural decisions (ADR-0001 to ADR-0003)
  research/        — competitive analysis, framework audits
  ROADMAP.md       — 4-phase plan
  VISION.md        — long-term direction
scripts/
  check-schema-changelog.sh — CI schema validation
```

## Tech Stack

- **Rust** 1.75+, edition 2021
- **Tokio** — async runtime (full features)
- **LanceDB** 0.26 + arrow-array/arrow-schema 57 — memory store (ADR-0003)
- **Clap** 4 — CLI
- **Serde/serde_json** — serialization
- **Anyhow/thiserror** — error handling
- **Tracing/tracing-subscriber** — observability
- **UUID** v4 — session/memory IDs
- **Chrono** — timestamps

## Conventions

- `cargo fmt` before every commit — CI enforces it
- `cargo clippy` warnings are errors — fix them, don't suppress
- Error handling: use `anyhow::Result` in binaries, `thiserror` for library error types
- Async: `#[async_trait]` for trait objects that need async methods
- No `unwrap()` in library code — propagate errors properly
- Modules: keep `mod.rs` as re-exports only, logic in named files

## Commit Format

Conventional commits. Always include co-author trailer:

```
feat(memory): implement LanceDB write path

Co-Authored-By: Rem <rem@superuserhq.dev>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

## What NOT To Do

- Don't change the channel abstraction design without reading `docs/adr/` first
- Don't add dependencies without checking if something in the existing stack already does it
- Don't implement the cortex or messaging layers yet — Phase 1 is memory + core only
- Don't `unwrap()` in library code
- Don't ignore `clippy` warnings

## CI

GitHub Actions on every push/PR to main:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo build`
- `cargo test`
- Schema changelog check (`scripts/check-schema-changelog.sh`)

PRs must pass all checks before merge.

## Key Reading (if context allows)

- `docs/prd/2026-02-17-superagents-v2.md` — full requirements
- `docs/adr/ADR-0003-memory-store-and-recall.md` — memory design
- `docs/ROADMAP.md` — what's in scope for Phase 1
