# ADR-0002: Memory Export Schema Versioning

**Status:** Draft  
**Date:** 2026-02-20  
**Authors:** Kani  
**Deciders:** Yao, Rem, Kani  

---

## Context

The VISION.md memory legibility model requires deterministic, canonical repo exports from the LanceDB runtime store. These exports serve as:

- The human-reviewable record of agent memory
- The recovery source if the DB is lost or corrupted
- The portability layer for migrating between deployments

For this to work across time, the export format must be versioned. Agents will evolve, memory schemas will change (new types, new graph edge kinds, renamed fields), and old exports must remain readable by newer runtimes.

Without an explicit versioning contract:
- Imports break silently after schema changes
- Recovery drills fail in production when they matter most
- Multi-agent setups with mixed runtime versions create import incompatibilities

---

## Decision

**Embed a schema version header in every export file. Require explicit migration steps between versions.**

### Format

Every canonical export file is newline-delimited JSON (`.ndjson`), with the first line always a schema manifest:

```json
{"superagents_export": "1", "schema_version": 1, "exported_at": "2026-02-20T02:00:00Z", "agent_id": "kani", "record_types": ["Fact", "Decision", "Identity"]}
```

Subsequent lines are typed memory records matching the schema for that version.

### Schema Registry

A `docs/schema/` directory tracks versions:

```
docs/schema/
  v1.md      — canonical field definitions for schema version 1
  v2.md      — future: fields added/changed + migration notes
  CHANGELOG.md
```

### Runtime Behaviour

- **Import:** Runtime reads the manifest line, checks `schema_version`, applies any necessary migrations before loading records into LanceDB.
- **Export:** Always writes the current schema version. No silent downgrades.
- **Migration functions:** Defined in `src/memory/migrations/`, one file per version transition (e.g., `v1_to_v2.rs`). Migration functions are pure (no side effects, deterministic output).
- **Compatibility guarantee:** Every runtime version must be able to read exports from any prior version. No forward compatibility required (older runtimes don't need to read newer exports).

---

## Rationale

### Why NDJSON?

- Human-readable and diffable in git
- Streamable — large exports don't require loading the full file into memory
- Trivial to parse in any language for recovery tooling

### Why version in the file, not the filename?

Filenames get copied, renamed, and moved. Embedding the version in the manifest makes the version intrinsic to the data, not the path.

### Why no forward compatibility?

Forward compatibility (old runtime reading new exports) would require all future schema changes to be additive and non-breaking. This is too restrictive for early-stage development. We accept the constraint that upgrades go one way.

---

## Consequences

- Export files are self-describing — any tooling can identify the version without external context
- Schema changes require a migration function before they can be merged
- Recovery drill protocol: `harness memory export` → delete DB → `harness memory import <file>` → verify record count matches. Must pass before any schema migration is released.
- Multi-agent setups must run the same or newer runtime version to share canonical exports

---

## Alternatives Considered

### 1. Version in filename only (`memory-v1-2026-02-20.ndjson`)

Rejected: fragile when files are renamed or copied. Version lost on move.

### 2. Separate schema sidecar file

Rejected: two-file dependency creates partial-write failure modes. Self-contained is safer.

### 3. Protobuf / binary format

Rejected for Phase 1: sacrifices human-readability which is a core legibility principle. Revisit if performance requires it.
