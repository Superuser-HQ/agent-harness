# Goose — Framework Audit

**Date:** 2026-02-16
**URL:** https://github.com/block/goose | https://block.github.io/goose

---

## Overview

Goose is an **open-source, local AI agent** built by **Block** (formerly Square, Jack Dorsey's company). It's designed as an on-machine developer assistant that goes beyond code suggestions — it can build projects, execute code, debug, orchestrate workflows, and interact with APIs autonomously. Available as both a **desktop app** and **CLI**.

**Language:** Rust (core) with TypeScript/Python extensions
**License:** Apache 2.0
**GitHub Stars:** ~15k+

---

## Architecture & Core Concepts

Goose is a **single-agent, extension-driven** architecture:

- **Core agent** — A Rust-based agent loop that processes user requests, calls tools, and manages conversation.
- **Extensions** — The primary extensibility mechanism. All capabilities beyond the core LLM are provided by extensions.
- **MCP-native** — Extensions are MCP (Model Context Protocol) servers. Any MCP server can be a Goose extension.
- **Sessions** — Conversation-based interaction model (CLI or desktop).
- **Custom Distributions** — Build branded Goose variants with preconfigured providers, extensions, and settings.

This is fundamentally a **developer tool**, not an agent framework for building other agents.

---

## Plugin/Tool System

Goose has the most mature extension system of the four:

**Built-in extensions:**
- **Developer** — General dev tools (file ops, shell, git). Enabled by default.
- **Computer Controller** — Webscraping, file caching, automations.
- **Memory** — Learns and remembers user preferences.
- **Skills** — Load agent skills from project/global directories.
- **Todo** — Task tracking across sessions.
- **Chat Recall** — Search conversation history across all sessions.
- **Apps** — Create/launch custom HTML apps.
- **Extension Manager** — Discover and toggle extensions dynamically during sessions.
- **Code Execution** — JS execution for tool discovery.

**External extensions:** Any MCP server can be added (via CLI or UI). Central extension directory available.

**Security:** Automatic malware scanning of external extensions before activation.

---

## Memory/State Management

- **Memory extension** — Teaches Goose to remember preferences across sessions. Relatively simple key-value style.
- **Chat Recall** — Full-text search across all session history.
- **Todo** — Persistent task tracking.
- **Skills** — Saved reusable agent behaviors.
- No semantic/vector memory. No working memory abstraction. Memory is more "preferences" than "knowledge."

---

## Multi-Agent Support

- **None.** Goose is explicitly a single-agent system. There's no concept of multiple agents collaborating.
- Extensions provide capabilities, but there's one agent orchestrating everything.

---

## Model Provider Support

- **Any LLM** — Provider-agnostic. Supports multi-model configuration (different models for different tasks).
- Providers: OpenAI, Anthropic (recommended — "works best with Claude 4"), Google Gemini, OpenRouter (200+ models), Tetrate Agent Router.
- ChatGPT subscription auth supported.
- Works best with models that have strong tool-calling capabilities.

---

## Deployment & Hosting

- **Local-only** — Runs on your machine (macOS, Linux, Windows).
- Desktop app (Electron-style) + CLI.
- **Not a server/service** — no API, no cloud deployment, no multi-user.
- Custom distributions allow pre-packaged variants for teams.

---

## Community & Maturity

- **Team:** Block (large company backing — Jack Dorsey, formerly Square).
- **Community:** Active Discord, YouTube, LinkedIn, Twitter, Bluesky.
- **Maturity:** Production-quality desktop app. Actively developed. Strong corporate backing.
- **Governance:** Formal governance document (unusual for OSS agent projects — shows seriousness).

---

## Strengths for Our Use Case

1. **MCP-native architecture** — Validates MCP as the right extension protocol. Our skills could be Goose-compatible.
2. **Extension model** — The built-in/external extension split with a directory is a great UX pattern.
3. **Custom distributions** — The concept of branded agent variants is interesting for our multi-persona needs.
4. **Skills system** — Reusable agent behaviors from directories maps to our skills concept.
5. **Security model** — Malware scanning of extensions is a mature touch.

---

## Weaknesses / Gaps

- **Not a framework** — It's an end-user product, not something you build on top of.
- **Local-only, single-user** — No server mode, no API, no multi-tenant.
- **No multi-agent** — Single agent only.
- **No channel abstraction** — Desktop/CLI only. No messaging platform integration.
- **Rust core** — Hard to extend the core agent (extensions are the only customization point).
- **Simple memory** — No semantic recall, no structured working memory.
- **Developer-focused** — Designed for coding tasks, not general-purpose agent harness.

---

## Key Takeaway

**Goose is an excellent developer-facing AI agent product, not a framework.** Its MCP-native extension architecture and skills system are architecturally inspiring, and Block's corporate backing ensures longevity. However, it's a local desktop tool with no server mode, no multi-agent support, and no channel abstraction. Useful as **inspiration for extension architecture and MCP patterns**, but not a foundation to build on.
