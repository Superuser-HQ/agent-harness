# ADR-0003: Channel Abstraction

**Status:** Proposed  
**Date:** 2026-02-20  
**Authors:** Rem (draft), Kani (review)  
**Deciders:** Yao, Gerald, Kani, Rem

---

## Context

Channel abstraction is the project's core differentiator. No existing agent framework provides channel-agnostic messaging (see VISION.md footnote 1 — 9 frameworks audited, zero have it). Chatbot frameworks (Botpress, Rasa) have channel abstraction but lack the agent harness model.

The PRD v1 (§4.1) scopes one primary messaging surface in v1, with a second behind a reliability gate. VISION.md describes the full vision: unified API, surface-specific formatting, channel policies, proactive messaging, cross-platform coordination.

This ADR defines the channel abstraction layer: the `Channel` trait, the surface capability model, formatting normalisation, policy enforcement, and how channels interact with the session tree (ADR-0002) and tool interface (ADR-0004).

### Current state (OpenClaw)

OpenClaw has a working but ad-hoc channel system:
- Supports Telegram, Slack, Discord, Signal, WhatsApp, iMessage, Google Chat
- Per-channel config: `requireMention`, `allowBots`, `allowlist`/`denylist`, `capabilities`
- Formatting rules are baked into agent instructions (prose like "no tables on Discord", "wrap links in `<>` on Discord")
- Bot credentials per surface in config
- Proactive messaging via heartbeat polls and cron jobs
- Reactions, threads, pins via the `message` tool
- Debounce for rapid inbound messages

What works: the model is proven across 7+ surfaces. What doesn't: formatting rules are prose (not enforced), capability differences are memorised by the agent (not declared), and policy logic is scattered between config and runtime.

---

## Decision

**Channels implement a `Channel` trait with declarative capabilities, policy enforcement at the layer boundary, and formatting normalisation as an output pipeline stage.**

### Channel trait

```rust
#[async_trait]
trait Channel: Send + Sync {
    /// Unique channel identifier (e.g., "slack:C09KR7WLTDW", "telegram:12345")
    fn channel_id(&self) -> &ChannelId;

    /// Surface type (Slack, Telegram, Discord, etc.)
    fn surface(&self) -> Surface;

    /// What this surface supports — declarative capability flags
    fn capabilities(&self) -> &SurfaceCapabilities;

    /// Channel-level policy (mention requirements, allowlists, bot policy)
    fn policy(&self) -> &ChannelPolicy;

    /// Send a message (formatting normalised automatically)
    async fn send(&self, message: OutboundMessage) -> Result<MessageId, ChannelError>;

    /// Send a reaction to a message
    async fn react(&self, message_id: &MessageId, emoji: &str) -> Result<(), ChannelError>;

    /// Edit an existing message
    async fn edit(&self, message_id: &MessageId, new_content: &str) -> Result<(), ChannelError>;

    /// Delete a message
    async fn delete(&self, message_id: &MessageId) -> Result<(), ChannelError>;

    /// Read message history
    async fn read(&self, opts: ReadOptions) -> Result<Vec<InboundMessage>, ChannelError>;

    /// Pin/unpin a message
    async fn pin(&self, message_id: &MessageId, pin: bool) -> Result<(), ChannelError>;

    /// Upload a file/attachment
    async fn upload(&self, attachment: Attachment) -> Result<MessageId, ChannelError>;
}
```

Bot identity and credentials are encapsulated inside the `Channel` implementation. They are never exposed upward to the agent, tools, or session layer. The agent knows it can `send()` — it doesn't know (or need to know) the bot token, webhook URL, or API client.

### Surface enumeration

```rust
enum Surface {
    Slack,
    Telegram,
    Discord,
    Signal,
    WhatsApp,
    IMessage,
    GoogleChat,
    Irc,
    // Extensible — new variants added per-surface
}
```

### SurfaceCapabilities

Declarative flags describing what each surface supports. Tools and the formatting pipeline read these flags — **no `if surface == Slack` checks anywhere in the codebase**.

```rust
struct SurfaceCapabilities {
    /// Supports markdown formatting
    markdown: bool,
    /// Supports markdown tables specifically
    markdown_tables: bool,
    /// Supports inline code blocks
    code_blocks: bool,
    /// Supports message threads (replies within a thread)
    threads: bool,
    /// Supports emoji reactions on messages
    reactions: bool,
    /// Supports inline buttons / interactive components
    inline_buttons: InlineButtonSupport,
    /// Supports file/image uploads
    file_uploads: bool,
    /// Supports message editing after send
    editable: bool,
    /// Supports message deletion
    deletable: bool,
    /// Supports pinning messages
    pinnable: bool,
    /// Supports voice messages / audio
    voice: bool,
    /// Maximum message length (characters)
    max_message_length: usize,
    /// Supports link previews / embeds (and whether they should be suppressed)
    link_previews: LinkPreviewPolicy,
    /// Supports headers (## style)
    headers: bool,
}

enum InlineButtonSupport {
    None,
    Dm,
    Group,
    All,
}

enum LinkPreviewPolicy {
    /// Surface shows previews, no action needed
    Default,
    /// Wrap URLs in <> to suppress embeds (Discord)
    SuppressWithAngleBrackets,
    /// No preview support
    None,
}
```

Each surface implementation provides its own `SurfaceCapabilities` as a const. These are the source of truth for what the surface can do.

### Formatting normalisation pipeline

Outbound messages pass through a normalisation pipeline before hitting the surface API. The pipeline reads `SurfaceCapabilities` and transforms content accordingly:

```
Agent output (markdown) → Normaliser → Surface API
```

Normalisation rules (all driven by capability flags, not surface identity):

| Capability flag | If false | Transform |
|-----------------|----------|-----------|
| `markdown_tables` | Tables → bullet lists |
| `headers` | `## Heading` → `**HEADING**` |
| `code_blocks` | Triple backtick → indent or plain text |
| `markdown` | Strip all markdown formatting |
| `link_previews = SuppressWithAngleBrackets` | Wrap bare URLs in `<url>` |
| `max_message_length` | Split into multiple messages at paragraph boundaries |

The agent writes natural markdown. The pipeline handles the rest. No formatting instructions in the system prompt. No "remember, don't use tables on Discord." The abstraction handles it.

### ChannelPolicy

```rust
struct ChannelPolicy {
    /// Require @mention for the agent to respond
    require_mention: bool,
    /// Allow messages from other bots
    allow_bots: bool,
    /// Allowlisted user IDs (empty = all allowed)
    allowlist: Vec<UserId>,
    /// Denylisted user IDs
    denylist: Vec<UserId>,
    /// Debounce window for rapid inbound messages
    debounce_ms: u64,
    /// Abstract permission level — dispatcher maps this to tool availability
    permission_level: ChannelPermissionLevel,
    /// Whether this channel can receive proactive messages (heartbeat/cron)
    proactive_allowed: bool,
    /// Ack reaction policy
    ack: AckPolicy,
}

enum ChannelPermissionLevel {
    /// Read-only tools only (e.g., restricted channels)
    ReadOnly,
    /// Standard tool set (default)
    Standard,
    /// Elevated tools allowed (trusted channels)
    Elevated,
}

struct AckPolicy {
    /// Which messages get ack reactions
    scope: AckScope,
    /// Emoji to react with on receipt (e.g., "eyes")
    on_receipt: Option<String>,
    /// Emoji to react with on completion (e.g., "white_check_mark")
    on_complete: Option<String>,
}

enum AckScope {
    /// No ack reactions
    None,
    /// All inbound messages
    All,
    /// Only messages that mention the agent
    MentionsOnly,
}
```

The runtime calls `channel.react(msg_id, policy.ack.on_receipt)` on inbound receipt, and `channel.react(msg_id, policy.ack.on_complete)` after the agent finishes. The channel layer guards against surfaces that don't support reactions (capability check before API call).

**`ChannelPermissionLevel`** replaces the earlier `denied_tools` field. Channels don't know tool names — they express an abstract permission level. The dispatcher maps `ChannelPermissionLevel` to tool availability using the permission tiers from ADR-0004. This keeps the channel layer opaque to tool semantics.

### Rate limiting

Each channel holds its own `RateLimiter` instance — one shared algorithm, per-channel config:

```rust
struct RateLimiter {
    msgs_per_second: f64,
    burst: u32,
    // token bucket internals
}

// Constructed per-surface at channel init:
// SlackChannel::new()  → RateLimiter::new(1.0, 3)
// TelegramChannel::new() → RateLimiter::new(30.0, 10)
```

**Policy enforcement happens at the channel layer boundary**, not in the dispatcher or the agent. When an inbound message arrives:

1. Check `denylist` → drop silently if matched.
2. Check `allowlist` → drop if non-empty and sender not in list.
3. Check `allow_bots` → drop if sender is a bot and flag is false.
4. Check `require_mention` → drop if agent not mentioned (exception: proactive sends bypass this — see below).
5. Check `debounce_ms` → buffer if within window.

Messages that pass all checks are delivered to the session. Messages that fail are dropped silently (no error to sender — same as OpenClaw's current behaviour).

**Policy inheritance in the session tree:** When a Branch session is spawned from a Main session, it inherits the parent's channel handle (per ADR-0004). The channel handle carries its policy with it — the Branch operates under the same policy as the Main session. Policy is not re-evaluated per tool call; it's baked into the channel handle at construction time. If the parent's channel policy changes mid-session (config reload), active branches keep the old policy until they complete.

### Proactive messaging

Heartbeat polls and cron-triggered messages are **proactive sends** — they originate from the runtime, not from an inbound user message. Proactive sends:

- **Bypass `require_mention`** — the agent is initiating, not responding. Mention checks are irrelevant.
- **Respect `proactive_allowed`** — channels can opt out of proactive messages entirely.
- **Route through a dedicated proactive channel handle** — not tied to any session's inbound channel. The runtime constructs a proactive `Channel` handle from config at startup, used by heartbeat and cron dispatchers.
- **Subject to the same formatting pipeline** — proactive messages are normalised like any other outbound message.

```rust
struct ProactiveRouter {
    /// Map of channel IDs to their proactive handles
    channels: HashMap<ChannelId, Arc<dyn Channel>>,
}

impl ProactiveRouter {
    /// Send a proactive message to a specific channel
    async fn send(&self, channel_id: &ChannelId, message: OutboundMessage) -> Result<MessageId, ChannelError>;
}
```

Cron jobs specify a target channel ID. Heartbeat results route to the agent's configured primary channel. This is explicit — no implicit "send to wherever the last message came from."

### Inbound message model

```rust
struct InboundMessage {
    /// Unique message ID on the surface
    id: MessageId,
    /// Channel this message arrived on
    channel_id: ChannelId,
    /// Sender identity
    sender: SenderId,
    /// Whether sender is a bot
    is_bot: bool,
    /// Message content (text, attachments)
    content: MessageContent,
    /// Thread/reply context (if threaded)
    thread_id: Option<ThreadId>,
    /// Whether this message mentions the agent
    mentions_agent: bool,
    /// Timestamp
    timestamp: Timestamp,
}

struct OutboundMessage {
    /// Text content (markdown — normalised by pipeline before send)
    text: String,
    /// Reply to a specific message
    reply_to: Option<MessageId>,
    /// Attachments
    attachments: Vec<Attachment>,
    /// Whether to send silently (no notification)
    silent: bool,
}
```

### Cross-platform addressing

v1 does not support cross-platform message routing (receive on Slack, reply on Telegram). Each session is bound to one channel. Cross-platform coordination is Phase 2 — it requires a message routing layer that maps agent-level intents to surface-level sends, which is unnecessary complexity for single-surface v1.

However, the architecture does not prevent it: the `Channel` trait is the same regardless of surface. A future routing layer would simply hold multiple `Channel` handles and dispatch based on configuration.

---

## Rationale

### Why declarative capabilities instead of surface-specific code?

Surface-specific branching (`if surface == Discord`) is the #1 maintenance problem in multi-surface systems. Every new surface requires auditing every branch. Every formatting rule requires N implementations.

Declarative capabilities invert this: each surface declares what it supports once. The formatting pipeline and tools read capabilities generically. Adding a new surface means implementing `Channel` and declaring `SurfaceCapabilities` — no changes to the pipeline, tools, or agent.

### Why enforce policy at the channel boundary?

Policy enforcement inside the dispatcher or agent means policy violations can slip through if a new code path forgets to check. Enforcing at the boundary (the channel layer itself) means nothing enters the system without passing policy. Defense in depth.

### Why no cross-platform in v1?

Cross-platform routing adds a message broker abstraction between sessions and channels. This is meaningful complexity for a feature that isn't needed when running one surface. The trait-based architecture means we can add it later without redesigning the channel layer.

### Why proactive sends bypass requireMention?

`requireMention` exists to prevent the agent from responding to every message in a busy channel. Proactive sends are agent-initiated — there's no message to "respond to." The mention check is semantically meaningless for outbound-only sends. Respecting `proactive_allowed` is sufficient gating.

---

## Alternatives Considered

| Option | Verdict |
|--------|---------|
| **Surface-specific formatting in agent prompt** | ❌ Current OpenClaw approach — fragile, relies on agent memory, no enforcement |
| **Separate formatter per surface** | ❌ N formatters that drift. Capability-driven pipeline is single implementation. |
| **Policy in agent config, not channel layer** | ❌ Policy enforcement outside the boundary — leaky, bypassable |
| **Cross-platform routing in v1** | ❌ Premature complexity. Trait architecture is forward-compatible. |
| **Webhook-only outbound (no bidirectional trait)** | ❌ Loses reactions, edits, threads — features we actively use |

---

## Consequences

### What this enables
- Add a new messaging surface by implementing `Channel` + declaring `SurfaceCapabilities`. No pipeline or tool changes.
- Formatting is correct by construction — agent writes markdown, pipeline adapts.
- Policy enforcement is guaranteed at the boundary — no message enters or exits without passing policy.
- Proactive messaging has a clean, explicit path through `ProactiveRouter`.
- Future cross-platform routing slots in naturally (multiple `Channel` handles per agent).

### What this requires
- First surface implementation must be complete and tested before the pipeline is considered stable.
- `SurfaceCapabilities` for each surface must be accurate — incorrect flags produce incorrect formatting. Needs integration tests per surface.
- Formatting normalisation pipeline must handle edge cases: nested markdown, code blocks containing tables, very long messages.
- Proactive router must be separate from session-bound channel handles — different lifecycle, different policy.

### Interaction with other ADRs
- **ADR-0002 (Session Tree):** Main sessions own a channel handle. Branches inherit it. Workers don't get one.
- **ADR-0004 (Tool Interface):** `ToolContext.channel` is `Option<Arc<dyn Channel>>`. Message tool delegates to it. Tool availability matrix (Workers have no channel) is enforced structurally.
- **ADR-0005 (Export Schema):** Channel messages may be exported as part of canonical session history. Export format is independent of surface.
- **ADR-0006 (Memory Store):** Channel metadata (which surface, which channel ID) is stored as session metadata in LanceDB.

### Resolved questions
- **Ack reactions:** Channel-layer with runtime lifecycle hooks. `AckPolicy` in `ChannelPolicy` defines scope and emojis. Runtime triggers `on_receipt` and `on_complete` at the appropriate lifecycle points. Channel layer guards against unsupported surfaces.
- **Rate limiting:** Shared `RateLimiter` algorithm, per-channel instance with surface-specific config. One implementation, N configurations. No cross-channel contention.
- **Tool restrictions per channel:** Replaced `denied_tools` (layer violation — channels shouldn't know tool names) with `ChannelPermissionLevel` (ReadOnly | Standard | Elevated). Dispatcher maps permission levels to tool availability via ADR-0004 tiers.

---

## References

- PRD v1 §4.1 (one messaging surface), §4.2 (second surface behind gate)
- VISION.md §Core Differentiator: Channel Abstraction
- OpenClaw channel configuration (prior art — 7+ surfaces)
- nanobot channel gateway pattern (prior art — config-driven adapters)
- ADR-0002 (Session Tree — channel handle inheritance)
- ADR-0004 (Tool Interface — ToolContext.channel)
