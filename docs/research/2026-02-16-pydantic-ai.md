# Framework Research: Pydantic AI

**Date:** 2026-02-16
**URL:** https://github.com/pydantic/pydantic-ai
**Language:** Python | **License:** MIT | **Stars:** ~30k+

---

## Core Architecture

Pydantic AI is a Python agent framework from the Pydantic team, designed with a "FastAPI feeling" — type-safe, ergonomic, production-grade. Agents are the core abstraction: generic containers parameterized by dependency type and output type (`Agent[Deps, Output]`).

- **Agent as unit:** Each `Agent` bundles instructions, tools, output type, dependencies, and model config.
- **Run modes:** `run()` (async), `run_sync()`, `run_stream()`, `run_stream_events()`, `iter()` (graph-level node iteration).
- **Graph support:** Complex workflows can be modeled as typed graphs to avoid spaghetti control flow.
- **Durable execution:** Agents can checkpoint and resume across failures — handles long-running and human-in-the-loop workflows.

## Tool Model

- **Decorator-based:** `@agent.tool` registers functions. Docstrings become tool descriptions. Parameter types become JSON schema via Pydantic.
- **Dependency injection:** Tools receive `RunContext[Deps]` with typed dependencies — DB connections, user context, etc.
- **Validation loop:** If LLM returns invalid tool args, Pydantic validates and sends errors back for retry.
- **MCP integration:** Native Model Context Protocol support for external tool servers.
- **Human-in-the-loop:** Tools can be flagged as requiring approval before execution (deferred tools).
- **Toolsets:** Groupable, composable tool collections.

## Memory Approach

- **No built-in memory layer.** Conversation history is passed explicitly via message objects.
- **Message history:** Can pass `message_history` from previous runs to continue conversations.
- Memory persistence is left to the application layer.
- Focus is on structured I/O rather than long-term state management.

## Multi-Agent Support

- **Agent-to-agent via A2A protocol** (Google's Agent2Agent standard).
- **Handoff pattern:** Agents can delegate to other agents within a run.
- **Graph-based orchestration:** Multi-agent workflows modeled as typed graphs with agent nodes.
- No built-in coordination layer (no shared memory, no RPC beyond A2A).

## What We'd Steal

| Feature | Relevance to Agent Harness |
|---------|---------------------------|
| **Typed dependency injection** | `RunContext[Deps]` pattern is elegant for passing DB connections, user context, API clients into tools. Port to TypeScript with generics. |
| **Validation-retry loop** | When tool args fail Pydantic validation, errors go back to LLM for self-correction. Our harness should do this with Zod/TypeBox. |
| **Deferred/approval tools** | Human-in-the-loop tool approval is exactly what we need for safety (PRD §4.2 base tools + trust model). |
| **Structured output guarantee** | Output type is validated — agent retries until it produces valid structured data. Strong pattern for our harness-first approach. |
| **Durable execution** | Checkpoint and resume across failures. Maps to our session recovery requirement (PRD §4.7). |
| **Graph-based workflows** | For complex multi-step processes, explicit graphs beat implicit chains. Consider for Phase 2 multi-agent. |
| **OpenTelemetry observability** | Tight Logfire/OTel integration. We deliberately omitted dashboards (PRD §8) but OTel hooks would be cheap to add. |

## What We'd Avoid

- Python-only — can't use directly, but patterns are portable.
- No file-based memory — doesn't align with our git-backed memory layer (PRD §4.5).
- Agent abstraction is conversation-centric, not harness-centric. We want scaffolding as the product, not the LLM call.
- A2A protocol is Google-centric and early-stage. Our RPC layer (PRD §5.1) should be simpler and more opinionated.
