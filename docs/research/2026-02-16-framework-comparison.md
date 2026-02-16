# Agent Framework Comparison Matrix

**Date:** 2026-02-16
**Context:** Evaluating frameworks for building a channel-agnostic AI agent harness (OpenClaw)

---

## Summary Matrix

| Dimension | **Mastra** | **AgentStack** | **Goose** | **CrewAI** |
|---|---|---|---|---|
| **Language** | TypeScript | Python | Rust + extensions | Python |
| **Type** | Full framework | Scaffolding CLI | Desktop agent product | Multi-agent framework |
| **GitHub Stars** | ~10k | ~2k | ~15k | ~25k |
| **Backing** | YC W25 (Gatsby team) | AgentOps AI | Block (Square) | CrewAI Inc + DeepLearning.AI |
| **Model Support** | 600+ models | Via LiteLLM/LangChain | Any LLM | Major providers |
| **Memory** | ⭐⭐⭐⭐⭐ (4 types + processors) | None (framework-dependent) | ⭐⭐ (preferences + chat recall) | ⭐⭐⭐⭐ (unified + composite scoring) |
| **Multi-Agent** | Via workflows/networks | Framework-dependent | None | ⭐⭐⭐⭐⭐ (core value prop) |
| **Tool System** | Typed tools + MCP | Curated registry | MCP-native extensions | CrewAI Tools + LangChain |
| **MCP Support** | ✅ Bidirectional | ❌ | ✅ Native | ✅ Recent addition |
| **Workflows** | ⭐⭐⭐⭐⭐ (graph-based) | Framework-dependent | ❌ | ⭐⭐⭐⭐ (Flows) |
| **Human-in-the-Loop** | ✅ First-class | Framework-dependent | ❌ | ⭐⭐ (limited) |
| **Channel Abstraction** | ❌ | ❌ | ❌ | ❌ |
| **Deployment** | Self-host (any Node env) | "Coming soon" | Local only | Self-host + AMP cloud |
| **Embeddable** | ✅ (library) | ❌ (scaffolding) | ❌ (product) | ⚠️ (heavy) |
| **Production Readiness** | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ (as desktop app) | ⭐⭐⭐⭐⭐ |

---

## Relevance to OpenClaw

### Tier 1: High Relevance
**Mastra** — Strongest architectural match. TypeScript-native, best memory system, embeddable, MCP support. The framework we'd most likely integrate with or draw patterns from.

### Tier 2: Architectural Inspiration
**CrewAI** — Best-in-class multi-agent and sophisticated memory. Python-only is a blocker for direct integration, but its memory system (unified class, composite scoring, scope trees) and Flows architecture are worth studying deeply.

### Tier 3: Pattern Inspiration
**Goose** — Not a framework but its MCP-native extension architecture, skills system, custom distributions, and extension security model are excellent patterns to adopt.

### Tier 4: Low Relevance
**AgentStack** — Scaffolding tool, not a framework. No unique runtime or abstractions. The framework-agnostic tool registry concept is mildly interesting.

---

## Key Insight: The Gap is the Channel Layer

**None of these frameworks provide channel abstraction** (Discord, Slack, WhatsApp, SMS, etc.). This is OpenClaw's core differentiator. The market has mature solutions for:
- Agent reasoning (all four)
- Memory (Mastra, CrewAI)
- Tool/extension systems (all four)
- Multi-agent orchestration (CrewAI)
- Workflows (Mastra, CrewAI Flows)

But **zero solutions** for channel-agnostic agent deployment with unified messaging. This validates our positioning.

---

## Recommendation

1. **Study Mastra deeply** — Closest to our stack. Consider using as a dependency or forking patterns.
2. **Adopt Mastra's memory architecture** — Working memory + semantic recall + observational memory is almost exactly our SOUL.md/MEMORY.md pattern, but formalized.
3. **Use MCP as the extension protocol** — Both Mastra and Goose validate this. Our skills should be MCP-compatible.
4. **Study CrewAI's unified Memory class** — The composite scoring (semantic + recency + importance) and scope tree concepts are worth porting to TypeScript.
5. **Our value-add is the channel layer** — Build the messaging abstraction none of them have.
