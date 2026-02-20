# ADR-0004: Tool Interface

**Status:** Proposed  
**Date:** 2026-02-20  
**Authors:** Rem  
**Deciders:** Yao, Gerald, Kani, Rem

---

## Context

The PRD v1 (§4.1) specifies base tools: Read, Write, Edit, Shell, Message, Memory. VISION.md describes a skill/extension system with hot-reload, compound capture, and eventual MCP compatibility.

This ADR defines the tool interface contract: how tools are declared, invoked, validated, and composed. The goal is a minimal interface that supports v1's base tools while being extensible enough for skills, MCP, and agent-authored tools in later phases.

### Current state (OpenClaw)

OpenClaw tools are:
- **Declared in JSON Schema** — each tool has a name, description, and parameters schema
- **Invoked by the LLM** — model emits tool calls, runtime dispatches
- **Skills** — markdown instruction files (`SKILL.md`) that teach the agent how to use external CLIs. Not formal tool declarations — the agent reads instructions and uses Shell/Read/Write to execute.
- **No validation-retry** — if the model sends malformed args, the tool fails and the error goes back as-is
- **No permission tiers** — all tools are equally trusted (policy is prose in AGENTS.md, not enforced)
- **No structured output** — tools return text strings, model parses what it needs

This works but has clear gaps: no type safety on tool results, no permission enforcement, no retry on validation failure, and skills are informal.

---

## Decision

**Tools implement a `Tool` trait with typed input/output, declarative permissions, and built-in validation-retry.**

### Tool trait

```rust
trait Tool: Send + Sync {
    /// Unique tool identifier (e.g., "read", "shell", "memory_search")
    fn name(&self) -> &str;

    /// Human-readable description for the LLM system prompt
    fn description(&self) -> &str;

    /// JSON Schema for the tool's parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Permission tier required to invoke this tool
    fn permission_tier(&self) -> PermissionTier;

    /// Execute the tool with validated parameters
    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError>;
}
```

### PermissionTier

```rust
enum PermissionTier {
    /// Observe only, never mutate (e.g., Read, memory_search, web_search)
    ReadOnly,
    /// Mutate within the agent's workspace (e.g., Write, Edit, memory_store)
    Workspace,
    /// Mutate outside workspace (e.g., Shell, Message send, HTTP requests)
    System,
    /// Irreversible or high-impact (e.g., delete files outside workspace, send emails)
    Elevated,
}
```

Enforcement:
- `ReadOnly` and `Workspace`: always allowed, no approval needed.
- `System`: allowed by default, but can be restricted per-agent or per-channel via policy config.
- `Elevated`: requires explicit human approval before execution. The runtime presents the tool call to the human and blocks until approved or denied. (Deferred approval pattern from PRD — agent continues other non-blocked work while waiting.)

**Elevated approval from deep branches:** Branches pass Elevated requests up the parent chain until they reach a session with a channel handle (typically Main). The approval prompt is displayed there. The originating branch suspends that tool call while waiting; it can continue other non-Elevated work in the meantime. If the entire parent chain has no channel (theoretical edge case in v1 — shouldn't happen since Main always has one), the request is denied with a `PermissionDenied` error.

### ToolContext

The execution context passed to every tool invocation:

```rust
struct ToolContext {
    session_id: SessionId,
    agent_id: AgentId,
    workspace_root: PathBuf,
    memory: Arc<dyn MemoryStore>,       // LanceDB handle for memory tools
    channel: Option<Arc<dyn Channel>>,  // Messaging surface (if session has one)
    config: Arc<AgentConfig>,           // Agent-level config (model, permissions, etc.)
}
```

Tools receive only what they need. A Worker session's `ToolContext` will have `channel: None` (workers don't message humans). Memory tools get the `memory` handle. File tools use `workspace_root` for path resolution and sandboxing.

### ToolResult

```rust
struct ToolResult {
    /// Structured output for the model (JSON — parseable, precise)
    output: serde_json::Value,
    /// Human-readable summary (for channel display, logs, audits)
    display: String,
    /// Artifacts produced (files created/modified, memory records written)
    artifacts: Vec<Artifact>,
}
```

The split between `output` (for the model) and `display` (for the human) is deliberate. Models need structured data to reason about. Humans need readable summaries. Conflating the two degrades both.

### ToolError

```rust
enum ToolError {
    /// Input validation failed — retry eligible
    ValidationError { message: String, field: Option<String> },
    /// Tool execution failed — not retry eligible (e.g., file not found)
    ExecutionError { message: String },
    /// Permission denied — never retry
    PermissionDenied { tier: PermissionTier, reason: String },
    /// Timeout — retry eligible with backoff
    Timeout { elapsed: Duration },
}
```

### Validation-retry loop

When the LLM sends malformed tool arguments:

1. Runtime validates args against `parameters_schema()`.
2. On `ValidationError`: send the error back to the model with the schema and the specific field that failed.
3. Model retries with corrected args.
4. **Max 2 retries** (3 total attempts). After that, surface the error to the parent session.

This pattern is proven in Pydantic AI and eliminates a large class of "tool call failed" noise.

### Tool registration

v1 tools are compiled into the binary — no dynamic loading:

```rust
fn register_base_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(ReadTool),
        Box::new(WriteTool),
        Box::new(EditTool),
        Box::new(ShellTool),
        Box::new(MessageTool),
        Box::new(MemorySearchTool),
        Box::new(MemoryStoreTool),
        Box::new(WebSearchTool),
        Box::new(WebFetchTool),
    ]
}
```

Dynamic tool loading (skills as runtime-registered tools) is Phase 2. In v1, skills remain as they are in OpenClaw: markdown instruction files that teach the agent to use Shell + Read + Write to accomplish tasks. This is pragmatic — skills work today without any framework support.

### Tool availability per session kind

Not all tools are available in all session types:

| Tool | Main | Branch | Worker |
|------|------|--------|--------|
| Read | ✅ | ✅ | ✅ |
| Write | ✅ | ✅ | ❌ |
| Edit | ✅ | ✅ | ❌ |
| Shell | ✅ | ✅ | ❌ |
| Message | ✅ | ✅ (parent's channel) | ❌ |
| Memory Search | ✅ | ✅ | ✅ |
| Memory Store | ✅ | ✅ | ❌ |
| Web Search | ✅ | ✅ | ✅ |
| Web Fetch | ✅ | ✅ | ✅ |

Workers are read-only by design (ADR-0002). They can search memory and the web, but cannot write files, store memories, or send messages.

**Enforcement mechanism:** Tool availability is enforced structurally via `ToolContext` construction, not dispatcher checks. A Worker's `ToolContext` is constructed with no write handle, no channel handle, and no memory store handle. Tools that require these capabilities (Write, Edit, Shell, Message, Memory Store) structurally cannot execute — they receive `None` for the required handle and return `PermissionDenied` immediately. This is more tamper-proof than dispatcher-level filtering: even if a tool is somehow registered for a Worker session, it cannot mutate anything.

Branches inherit the parent's channel handle for Message tool access — a branch can send messages on behalf of the parent. This is intentional: branches doing research should be able to report progress. Channel policies (ADR-0003) govern what the branch can actually send.

---

## Rationale

### Why a trait, not a function signature?

The `Tool` trait bundles declaration (name, description, schema) with execution. This means:
- The LLM system prompt is generated from the same source of truth as the dispatcher
- Permission tier is declared alongside the tool, not in a separate config file that drifts
- Tools are self-describing — useful for future MCP export (a tool can serialize itself into MCP format)

### Why JSON Schema for parameters?

It's the lingua franca for LLM tool calling. Every major provider (Anthropic, OpenAI, Google) accepts JSON Schema for tool parameter definitions. Using it natively means zero translation layer.

### Why split output/display in ToolResult?

OpenClaw returns a single string for both model consumption and human display. This forces either:
- Verbose output that wastes model tokens (human-friendly but model-hostile)
- Terse output that's unreadable in logs (model-friendly but human-hostile)

Splitting them lets each serve its audience. The model gets `{"files": ["a.rs", "b.rs"], "line_count": 42}`. The human gets `"Read 2 files (42 lines total)"`.

### Why compiled-in tools for v1?

Dynamic tool loading adds complexity: discovery, versioning, sandboxing, trust. None of that is needed when the tool set is known at compile time. The trait interface is designed for future dynamic loading, but v1 doesn't pay that cost.

### Why validation-retry instead of just failing?

LLMs make schema errors frequently — wrong types, missing required fields, extra fields. These are recoverable. Without retry, every schema error becomes a visible failure that the agent must handle conversationally. With retry, 90%+ of these resolve silently on the second attempt.

2 retries is the sweet spot: enough to recover from common errors, not enough to loop on fundamentally broken tool calls.

---

## Alternatives Considered

| Option | Verdict |
|--------|---------|
| **Free functions instead of trait** | ❌ Loses self-description (schema, permissions) — would need a parallel registry |
| **Dynamic loading in v1** | ❌ Premature — adds complexity without v1 benefit. Trait is forward-compatible. |
| **MCP as the native interface** | ❌ MCP is a wire protocol, not an internal interface. Internal tools shouldn't pay serialization cost. MCP export is a Phase 2 wrapper over the trait. |
| **No validation-retry** | ❌ Proven in Pydantic AI to significantly reduce tool call failures. Low cost, high value. |
| **Unified output (no split)** | ❌ Forces compromise between model and human readability. Split is cheap. |
| **No permission tiers** | ❌ All-or-nothing is a security gap. Tiered permissions are table stakes. |

---

## Consequences

### What this enables
- Type-safe tool interface with clear contract between runtime and tools
- Permission enforcement at the dispatch layer (not prose policy)
- Validation-retry reduces visible failures from malformed tool calls
- Structured results enable parent sessions to make programmatic decisions about child outputs
- Forward-compatible with MCP export and dynamic skill loading

### What this requires
- All v1 tools must implement the `Tool` trait
- Runtime dispatcher must validate args against schema before calling `execute`
- Retry loop must be implemented in the dispatcher (not in individual tools)
- `ToolContext` construction must respect session kind (Workers get no channel, no write-capable tools)
- Elevated tier tools need an approval flow (can be simple in v1 — block and prompt in the Main session's channel)

### Interaction with other ADRs
- **ADR-0002 (Session Tree):** Tool availability matrix is keyed on `SessionKind`. `ToolContext.session_id` lets tools know which session they're in.
- **ADR-0003 (Channel Abstraction):** `ToolContext.channel` is the interface. Message tool delegates to it. Channel policies constrain what Message can do.
- **ADR-0005 (Export Schema):** Memory tools write to LanceDB; the export pipeline (ADR-0005) handles canonical export separately.
- **ADR-0006 (Memory Store):** Memory tools use `ToolContext.memory` (LanceDB handle) for search and store operations.

---

## References

- PRD v1 §4.1 (base tools), §5 (architecture decisions)
- VISION.md §Extension/Skill System, §Guardrails & Security
- Pydantic AI validation-retry pattern (`research/2026-02-16-pydantic-ai.md`)
- OpenClaw tool implementation (prior art)
- MCP specification (future compatibility target)
