# AgentStack — Framework Audit

**Date:** 2026-02-16
**URL:** https://github.com/agentstack-ai/AgentStack | https://docs.agentstack.sh

---

## Overview

AgentStack is a **CLI scaffolding tool** for AI agent projects. Built by **AgentOps AI**. Think "create-react-app but for agents." It's not a framework itself — it generates projects that use existing frameworks (CrewAI, LangGraph, OpenAI Swarms, LlamaStack) and adds a curated tool ecosystem on top.

**Language:** Python
**License:** MIT
**GitHub Stars:** ~2k+

---

## Architecture & Core Concepts

AgentStack is a **meta-framework / scaffolder**, not a runtime:

- **`agentstack init`** — Generates a project structure with your chosen framework.
- **`agentstack generate agent/task`** — Code-gen for agents and tasks.
- **`agentstack tools add`** — Installs framework-agnostic tools from a curated registry.
- **`agentstack run`** — Runs the generated project.
- **Config-driven:** `agents.yaml` and `tasks.yaml` define agent configurations.

The actual agent runtime is delegated to the underlying framework (CrewAI, LangGraph, etc.).

---

## Plugin/Tool System

- **Largest framework-agnostic tool repository** — tools work across supported frameworks.
- `agentstack tools add` installs and wires tools automatically.
- Tools listed at docs.agentstack.sh/tools/community.
- LLM support via LiteLLM or LangChain (most providers supported).

---

## Memory/State Management

- **No native memory system** — memory depends entirely on the chosen underlying framework.
- If you pick CrewAI, you get CrewAI's memory. If LangGraph, you get LangGraph's.
- AgentStack itself adds no memory abstraction.

---

## Multi-Agent Support

- Depends on chosen framework. CrewAI gives crews, LangGraph gives graphs, OpenAI Swarms gives swarms.
- AgentStack's `generate agent` makes it easy to add agents but orchestration is framework-level.

---

## Model Provider Support

- Via LiteLLM or LangChain — effectively all major providers supported.
- Not AgentStack's own layer; it's pass-through.

---

## Deployment & Hosting

- **`agentstack run`** for development.
- Production deployment: "coming soon" per their docs.
- No hosted offering.
- You'd deploy the generated project however you deploy Python apps.

---

## Community & Maturity

- **Team:** AgentOps AI (also makes the AgentOps observability platform).
- **Baked-in observability** via AgentOps.
- **Community:** Discord, growing but smaller than the frameworks it wraps.
- **Maturity:** Active development. Framework support expanding (roadmap: Pydantic AI, Eliza, AG2).

---

## Strengths for Our Use Case

1. **Framework-agnostic tools** — the curated tool registry is genuinely useful as a reference.
2. **Quick prototyping** — fast to scaffold and test different framework approaches.
3. **AgentOps integration** — good observability story.

---

## Weaknesses / Gaps

- **Not a framework** — it's scaffolding. No runtime, no architecture, no memory. You're really using CrewAI or LangGraph underneath.
- **Python only** — doesn't match our TypeScript stack.
- **No channel abstraction** — no messaging layer.
- **No production deployment story** — still "coming soon."
- **Thin abstraction** — you quickly outgrow it and work directly with the underlying framework.
- **No unique value for our use case** — everything it provides, we'd get from the underlying framework directly.

---

## Key Takeaway

**AgentStack is a scaffolding CLI, not a framework.** It's useful for quickly prototyping with different Python agent frameworks, but provides no unique runtime, memory, or channel abstractions. For our use case, it's essentially a pass-through to CrewAI or LangGraph. Not architecturally relevant to OpenClaw, but its framework-agnostic tool registry concept is worth noting.
