# Docs

## Structure

| Folder       | Purpose                          |
|--------------|----------------------------------|
| `prd/`       | Product Requirements Documents   |
| `adr/`       | Architecture Decision Records    |
| `research/`  | Research notes and findings      |

## Naming Conventions

### PRDs & Research

Date-prefixed: `[yyyy-mm-dd]-title.md`

Example: `prd/2026-02-16-superagents-v1.md`

Dates are self-assigning (no coordination needed) and give instant context on when a doc was written.

### ADRs

Sequentially numbered: `[nnnn]-title.md`

Example: `adr/0001-use-event-sourcing.md`

ADRs use numbered IDs following the [standard ADR convention](https://adr.github.io/), making them easy to reference (e.g. "see ADR-0001").
