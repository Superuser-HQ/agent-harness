# Framework Research: OpenHands (formerly OpenDevin)

**Date:** 2026-02-16
**URL:** https://github.com/All-Hands-AI/OpenHands
**Language:** Python | **License:** MIT | **Stars:** ~50k+

---

## Core Architecture

OpenHands is an AI-driven software development platform — purpose-built for coding agents, not general-purpose chat. It ships as multiple products sharing a core SDK:

- **Software Agent SDK:** Composable Python library — the engine behind everything else.
- **CLI:** Claude Code / Codex-like terminal experience.
- **Local GUI:** REST API + React SPA (Devin/Jules-like).
- **Cloud:** Hosted deployment with Slack/Jira/Linear integrations, RBAC, collaboration.
- **Enterprise:** Self-hosted Kubernetes deployment.

Key architectural features:
- **Sandbox execution:** Agents run in isolated Docker containers. Strong security boundary.
- **Agent-Computer Interface (ACI):** Specialized interfaces for browsing, editing, terminal — not just raw bash.
- **Task planning & decomposition:** Built-in planning that breaks complex tasks into steps.
- **Automatic context compression:** Handles long conversations without manual memory management.
- **Model-agnostic:** Works with Claude, OpenAI, Qwen, Devstral, and open-source models.

## Tool Model

- **Action/Observation pattern:** Agents emit Actions (run command, edit file, browse web), environment returns Observations.
- **Strong ACI:** Purpose-built interfaces for code editing, terminal, browser — more structured than raw tool calls.
- **Sandboxed execution:** All tool execution happens in Docker containers. Host is protected.
- Not a generic tool framework — tools are domain-specific to software engineering.

## Memory Approach

- **Context compression:** Automatic summarization of older context to stay within token limits.
- **No persistent memory layer.** Each task/conversation is independent.
- **Conversation history:** Maintained within a session, discarded after.
- Research focus on "Theory-of-Mind" module (separate repo) for understanding user intent across interactions.

## Multi-Agent Support

- **SDK supports multi-agent orchestration:** "Scale to 1000s of agents in the cloud."
- **Cloud features:** Conversation sharing, collaboration features between human users.
- **No native agent-to-agent protocol.** Multi-agent is orchestrated at the SDK level (programmatic).
- **Enterprise:** Multi-user RBAC, but agents don't collaborate with each other — they serve separate users.

## What We'd Steal

| Feature | Relevance to Agent Harness |
|---------|---------------------------|
| **Sandbox-first execution** | Docker isolation for all tool execution is a strong safety pattern. Our harness should default to sandboxed bash (PRD §4.2). |
| **Agent-Computer Interface** | Structured interfaces for edit/browse/terminal > raw bash. Aligns with our "5 primitives" approach (PRD §4.2). |
| **Context compression** | Automatic summarization when context gets long. Critical for our session model — branches accumulate tokens fast. |
| **Task decomposition** | Built-in planning for complex tasks. Could be a default behavior in our agent loop. |
| **Benchmark-driven development** | SWE-bench, SWT-bench performance tracking. We should benchmark our harness against standard evals from day one. |
| **SDK + CLI + GUI layering** | Clean separation: SDK (core) → CLI (power users) → GUI (everyone). Our architecture should support this layering. |

## What We'd Avoid

- **Coding-only focus.** We need a general agent harness, not just a coding agent. OpenHands is too specialized.
- **No messaging surface abstraction.** Slack/Jira integrations are cloud-only, not part of the core SDK.
- **No persistent memory.** Each session is ephemeral — doesn't support compound engineering (PRD §2).
- **Python-only SDK.** We're TypeScript.
- **Heavy infrastructure requirements.** Docker, REST API, React SPA — more weight than our "thin core" principle (PRD §3.4).
- **Enterprise lock-in patterns.** Source-available enterprise features create a two-tier system.
