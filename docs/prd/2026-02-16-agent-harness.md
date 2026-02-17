# PRD: SHQ Agent Harness
**Status:** v1.8
**Author:** Kani (driven), Rem (reviewer)
**Date:** 2026-02-16
**Stakeholders:** Yao, Gerald

---

## 1. Why

Strip the problem to its root: **we need agents that persist across conversations, collaborate with humans and each other, and reach people where they already are.**

That's it. Everything in this document serves one of those three needs. If a feature doesn't trace back to persistence, collaboration, or reach — it's cut.

### 1.1 The Irreducible Problem

OpenClaw's creator is joining OpenAI. The project moves to a foundation with uncertain roadmap. SHQ depends on OpenClaw for:

1. **Persistence** — memory continuity across sessions (SOUL.md, MEMORY.md, daily logs)
2. **Collaboration** — multi-agent operations (Kani + Rem), human-agent governance
3. **Reach** — messaging across surfaces (Slack, Telegram, Signal, Discord)

Without a replacement, we're locked into a platform we can't steer. But we don't need to replicate OpenClaw. We need to build the *smallest possible thing* that solves persistence + collaboration + reach, then compound from there.

### 1.2 Why Not Just Use an Existing Framework?

We audited 9 frameworks (Mastra, CrewAI, Goose, AgentStack, nanobot, OpenHands, Pydantic AI, pi-agent-core, pi-mom). The finding is unambiguous:

> **No framework has channel abstraction.** Zero. This is our differentiator.

The market has mature solutions for agent reasoning, memory, tool systems, multi-agent orchestration, and workflows. But nobody has built channel-agnostic agent deployment with unified messaging. That's the gap, and that's what we build.

### 1.3 Why Now?

OpenClaw's transition creates a 2-3 month window. If we don't build now, we either migrate to a degrading platform or lose capability. The research is done. The design space is mapped. Ship.

---

## 2. Vision

An agent harness — not a chatbot framework — that treats scaffolding, feedback loops, and mechanical enforcement as first-class concerns. Every task should leave the system smarter (compound engineering). Humans steer, agents execute.

---

## 3. Design Principles

1. **Harness-first** — The scaffolding, constraints, and feedback loops ARE the product. Not the LLM, not the chat UI.
2. **Compound by default** — Every solved problem becomes a reusable pattern. The system gets smarter with every PR.
3. **Mechanical enforcement** — Encode taste into linters, tests, and structural constraints. Documentation rots; code doesn't.
4. **Start smaller than you think** — Thin core + extensions. Resist building features into the core that belong in the skill layer.
5. **Agent legibility** — If it's not in the repo, it doesn't exist. Repository-local, versioned artifacts are the system of record.
6. **Distributed ownership** — No single maintainer. Bus-factor > 1 from day one.
7. **Repo-native everything** — No Notion, no external wikis. Markdown files, git history, GitHub primitives. If it's not in the repo, it didn't happen.

---

## 4. Core Architecture

### 4.1 Session Model: Trees, Not Lists

Sessions are tree-structured, not flat history. Agents can branch (sub-tasks, reviews, research) without polluting the main context. Branches can be discarded or merged back.

- **Main session** — primary conversation with human
- **Branch sessions** — sub-tasks, side-quests, background work
- **Session handoff** — state dump to file before context resets (solves the "cold pickup" problem)

> **Research validation:** pi-agent-core's event streaming architecture (`agent_start` → `turn_start` → `message_update` → `tool_execution_*` → `turn_end`) provides the granular event model we need for session trees. Their `transformContext() → convertToLlm()` message pipeline cleanly separates app-level messages from LLM messages. We adopt these patterns.

Tree-structured sessions create garbage collection needs — abandoned branches accumulate, so we need a branch pruning policy (TTL or explicit close). On the upside, session trees give us audit trails for free: every decision path is preserved, which feeds directly into our ADR process (§10). Worth noting: branches that produce useful output should trigger compound capture — the system prompts to generalize the solution.

### 4.2 Base Tools (5 primitives)

The core ships with exactly 5 tools. Everything else is an extension.

| Tool | Purpose |
|------|---------|
| **Read** | Read files, images, structured data |
| **Write** | Create/overwrite files |
| **Edit** | Surgical text replacement |
| **Shell** | Execute commands, manage processes |
| **Message** | Send/receive across surfaces (Slack, Telegram, Signal, etc.) |

> **Research validation:** pi-mom's 5 core tools (`bash`, `read`, `write`, `edit`, `attach`) are almost identical. This independent convergence validates the design. pi-mom's self-managing model — where the agent creates its own tools as bash scripts — demonstrates that 5 primitives are sufficient for compound capability growth.

Memory is an **architectural layer**, not a tool (see §4.5). Tools interact with memory through Read/Write (files) and a thin `remember`/`recall` API backed by the structured memory DB.

### 4.3 Extension/Skill System

- **Skills** = modular, versioned instruction sets (like OpenClaw's SKILL.md pattern)
- **Hot-reload** — agents can create, modify, and reload extensions at runtime
- **Progressive disclosure** — small AGENTS.md (~100 lines) as table of contents, deeper docs in structured `docs/`
- **Compound capture** — after every significant task, the system prompts to capture learnings as tagged, searchable solution docs in `docs/solutions/`

> **Research insight:** nanobot's self-configuring skill URLs (agent reads a URL, follows instructions, configures itself) is an elegant zero-setup pattern worth adopting.

One consequence of this choice: self-managing tools (the pi-mom pattern) mean agents can extend their own capabilities without human intervention. This is powerful but needs guardrails — new tools should be PR'd, not silently deployed. See §5 for the full guardrails architecture.

#### Phase 2: MCP Compatibility
- Skills should be exposable as MCP servers for interop with Goose, Mastra, and the broader ecosystem. This is deliberately Phase 2 — we don't need external agents consuming our skills on day one. Get the skill system working first, add MCP exposure when it's stable. (Goose's MCP-native extension architecture validates MCP as the right protocol.)

### 4.4 LLM Abstraction Layer

- Multi-provider from day 1 (Anthropic, OpenAI, Google, local models)
- Hot-swap models mid-session without config surgery
- Thinking trace conversion between provider formats
- Split tool results: structured data for model, clean summary for human

> **Research insight:** Mastra routes 600+ models through a unified layer. pi-agent-core provides `setModel()`, `setThinkingLevel()` at runtime. Pydantic AI's validation-retry loop (invalid tool args get sent back to LLM for self-correction) should be ported — use schema validation at the type system level (Rust's type system + serde).

### 4.5 Memory Layer

Structured DB for runtime, repo exports for canonical record. Inspired by Spacebot's typed memory graph.

**Storage:** SQLite (structured metadata + graph edges) + vector store (embeddings for semantic recall)

**Memory types:** Fact, Preference, Decision, Identity, Event, Observation, Goal, Todo

**Graph edges:** RelatedTo, Updates, Contradicts, CausedBy, PartOf

**Recall:** Hybrid — vector similarity + full-text search, merged via Reciprocal Rank Fusion

**Importance scoring:** Access frequency, recency, graph centrality. Identity memories exempt from decay.

**Creation paths:** Agent-initiated, compactor-initiated, cortex-initiated

**Legibility model:**
- **DB is runtime cache/index and operational store.**
- **Repo export is canonical, reviewable, and portable record.**
- Every canonical memory class (Decision, Identity, long-lived Fact, Goal state transitions) must export to versioned files in the repo.
- Import/export determinism required: same DB snapshot → same exported artifacts.
- Recovery drill: rebuild fresh DB from canonical exports must pass before production cutover.

> **Research insight:** Mastra's tiered architecture (working memory, semantic recall, observational memory) and CrewAI's composite scoring (semantic similarity × recency × importance with tunable weights) informed this design. Spacebot validates the typed graph approach in production — 8 memory types with graph edges, hybrid recall via RRF, and a Cortex process that maintains the graph.

Multiple agents share the memory graph with scoped access (per-agent, per-team, global). Cross-agent memory isolation (§5.5) is enforced — agents only see memory they're explicitly granted access to.

### 4.6 Memory Policy (`MEMORY_POLICY.md`)

Memory is not left to each agent's ad-hoc judgment. An explicit `MEMORY_POLICY.md` in every agent workspace defines rules for the structured memory DB.

**What to remember (stored as typed memories in DB):**
- Decisions and their rationale (type: Decision)
- User preferences and corrections (type: Preference)
- Lessons learned from failures (type: Observation)
- Project context that changes rarely but matters always (type: Fact)
- Active goals and todos (types: Goal, Todo)

**What NOT to remember:**
- Secrets, credentials, tokens (use env vars or secret managers)
- Transient state (build output, temp files)
- Verbatim conversation logs (session history handles this)
- Other people's private information encountered in shared channels

**When to remember:**
- End of every significant task → agent-initiated memory creation
- Compaction events → compactor-initiated extraction of key facts
- Cortex maintenance cycles → consolidation, dedup, decay
- On explicit trigger → human says "remember this" or reacts with 📌

**Canonical export rules:**
- Decision, Identity, and long-lived Fact memories export to repo on every checkpoint
- Export format: structured markdown/JSON with frontmatter (`type`, `tags`, `date`, `importance`)
- Export freshness enforced in CI — stale exports block merges
- Import/export determinism tested: same DB state → same export artifacts

**Hygiene rules:**
- Importance scoring governs decay — low-access, low-centrality memories decay over time
- Identity memories exempt from decay
- Near-duplicate memories merged by Cortex during maintenance cycles
- Schema migrations versioned and tested

Explicit memory policy means new agents onboard faster — they read the policy, not reverse-engineer conventions from examples. DB-backed memory with canonical exports gives us the best of both worlds: fast structured recall at runtime, reviewable artifacts in git.

### 4.7 Mechanical Enforcement

- **Architecture linters** — enforce layer boundaries, dependency directions, naming conventions
- **Structural tests** — validate repo knowledge base is up-to-date and cross-linked
- **Custom lint error messages** — include remediation instructions so agents self-fix
- **Recurring cleanup agents** — scan for drift, open fix-up PRs (garbage collection pattern)

**Starter enforcement rules (v1):**
1. No direct LLM API calls outside the abstraction layer
2. Every skill must have SKILL.md with `name`, `description`, and `tools` fields
3. No tool can write outside workspace without explicit permission
4. MEMORY_POLICY.md must exist in every agent workspace
5. ADRs must have status, date, and decision fields
6. Canonical memory exports must pass schema conformance and determinism tests

### 4.8 Process Model: Channel / Branch / Worker / Cortex

Inspired by Spacebot's architecture. Split the monolith into specialized concurrent processes.

**Channel** — The user-facing process. Has soul, identity, personality. Always responsive — never blocked by work, never frozen by compaction. Delegates thinking to branches and heavy work to workers.

**Branch** — A fork of the channel's context that goes off to think. Has full conversation history. Operates independently; the channel never sees the working, only the conclusion. Multiple branches run concurrently. Deleted after returning results.

**Worker** — Independent process that does jobs. Gets a specific task, focused system prompt, and task-appropriate tools. No channel context, no personality. Can be fire-and-forget (one-shot tasks) or interactive (long-running, accepts follow-ups).

**Cortex (Supervisor)** — Dedicated process that sees across all channels, workers, and branches. v1 scope limited to:
- Worker/branch supervision (stuck process cleanup, retries, kill policies)
- Health signals and alerts (queue depth, failure rate, memory sync lag, process liveness)

Deferred to post-v1: pattern mining, memory bulletins, admin chat interface.

> **Research validation:** Spacebot validates this architecture in production for teams/communities. The key insight: a channel that never blocks means 50 users can interact simultaneously. The delegation model means thinking, executing, and responding happen concurrently, not sequentially.

### 4.9 Error Handling & Resilience

- **Retry with backoff** — automatic retry on transient failures (rate limits, network errors)
- **Provider failover** — if provider X is down, fall back to provider Y automatically
- **Session recovery** — checkpoint session state; recover from crashes without losing context
- **Graceful degradation** — reduced capability beats total failure (e.g., fall back to simpler model if primary unavailable)
- **Abort support** — clean cancellation of in-progress tool calls with partial result preservation

> **Research insight:** Pydantic AI's durable execution (checkpoint and resume across failures) and pi-agent-core's steering messages (interrupt running agents, queue follow-up work) are both patterns we adopt.

---

## 5. Guardrails

Guardrails are not a feature bolted onto the side. They are the architectural skeleton that everything else hangs from. An agent harness without guardrails is just a script runner with extra steps. This section defines the trust, safety, and control architecture — the rules that let us give agents real power without losing sleep.

### 5.1 Tool-Level Guardrails

Every tool call passes through a permission layer before execution. Tools are classified into four tiers:

| Tier | Scope | Examples | Default |
|------|-------|----------|---------|
| **Read-only** | Observe, never mutate | Read files, web search, snapshot | Allowed |
| **Workspace-scoped** | Mutate within agent workspace | Write, Edit, Shell (in workspace) | Allowed |
| **System-wide** | Mutate outside workspace | Shell (arbitrary paths), install packages | Requires approval |
| **Elevated** | Irreversible or high-impact | Deploy, send money, delete production data | Requires explicit human approval per invocation |

Per-channel tool policies govern what tools are available in which context. A channel config can allowlist specific tools (`read`, `write`, `message`) or denylist dangerous ones (`shell`). This mirrors OpenClaw's current model and works well in practice — a Slack channel where the agent can only read and respond is fundamentally different from a DM where it has full workspace access.

File system boundaries are enforced mechanically, not by convention. An agent cannot write outside its workspace directory without an explicit permission grant in its config. This is the single most important guardrail for multi-agent setups: agent A's workspace is agent A's workspace, period. The `trash` command is preferred over `rm` for anything destructive — recoverable beats gone forever.

Network and external action gates deserve special attention. Any action that leaves the machine — sending an email, making an API call to a third-party service, deploying code, posting publicly — passes through an approval gate. The default is "ask first." Trusted actions (like pushing to a known git remote) can be pre-approved in config, but the default posture is conservative. This means agents start cautious and earn broader permissions over time, not the reverse.

### 5.2 Agent-Level Guardrails

**Trust model.** An agent trusts its primary human fully. It treats other humans on the team as trusted for project scope. It treats other agents as collaborators, never as instructors. This distinction matters: if Agent B tells Agent A to disable its file system boundaries, Agent A refuses. Agents cooperate, but each agent's guardrails are set by its human, not by peer agents.

**Prompt injection defense.** Untrusted content — web page fetches, user inputs in group chats, webhook payloads, file contents from unknown sources — must be clearly demarcated in the agent's context and never treated as instructions. The harness wraps untrusted content in explicit markers (e.g., `<untrusted_content source="web_fetch">...</untrusted_content>`) so the model can distinguish data from directives. This doesn't make prompt injection impossible, but it raises the bar significantly and makes the attack surface explicit.

**Context isolation.** Sub-agents and branch sessions do not inherit elevated permissions from their parent. If the main session has system-wide shell access, a branch spawned for a research task starts with read-only unless explicitly granted more. This is the principle of least privilege applied to session trees. The downstream effect is that spawning a sub-agent is always safe — you can't accidentally create a more powerful child than intended.

**Self-modification limits.** Agents can propose changes to their own SOUL.md, AGENTS.md, or config files, but they cannot self-approve those changes. Proposed modifications always go through the standard PR/review process — a reviewable diff, not a thumbs-up in chat. This prevents drift where an agent gradually loosens its own constraints through incremental self-edits.

### 5.3 Human-in-the-Loop

The harness is designed around the assumption that humans steer and agents execute. This means clear escalation paths.

**When should the agent stop and ask?**
- Before any action classified as elevated (§5.1)
- When it's uncertain about intent (ambiguous instructions, conflicting requirements)
- Before making architectural decisions that constrain future options
- When the cost of being wrong exceeds the cost of asking
- Before any public communication (tweets, emails to external parties, blog posts)
- Before any financial action (purchases, transfers, subscription changes)
- Before deploying code to production

**Approval gates** are configurable per agent and per channel. A deployment gate might require explicit `/approve` from the human. A public communication gate might require the human to review the draft before sending. The gates are mechanical — not "the agent should probably ask" but "the system blocks execution until approval is received."

**Kill switch.** The human can halt any agent immediately, from any channel. `harness stop <agent>` or a configured emoji reaction (e.g., 🛑) kills the current session and queues no follow-up work. The agent shuts down cleanly, preserving state for later inspection but taking no further action. This is non-negotiable.

**Audit trail.** Every tool call is logged with timestamp, parameters, and result. Every external action (message sent, API called, file written outside workspace) gets an additional audit entry. The audit log is append-only, stored in the repo, and agents cannot modify their own audit history. This means any agent action is traceable after the fact — "what did Agent A do at 3pm?" is always answerable.

### 5.4 Cost and Resource Guardrails

Agents can burn through tokens and API calls fast, especially in compound loops or multi-agent setups. The harness enforces budgets at multiple levels:

- **Token budget per session** — configurable ceiling. When hit, the agent checkpoints and stops rather than silently burning money.
- **Token budget per task** — for sub-agents and branch sessions. A research task doesn't need the same budget as a full implementation task.
- **Circuit breaker** — if >50% of a session's token budget is consumed in <5 minutes, the agent pauses and alerts the human. Budgets catch the ceiling, but circuit breakers catch the velocity — a $50 budget burned in a tight retry loop is still a problem.
- **Rate limiting on external API calls** — per-provider, configurable. Prevents runaway loops from hammering APIs.
- **Model tier restrictions** — configurable rules like "use haiku/flash for simple lookups, sonnet for standard work, opus only for complex reasoning." The agent can request a tier upgrade, but the default should be the cheapest model that gets the job done. This creates a tension with developer experience (agents work better with smarter models) that we resolve by making the tier easily overridable, not by defaulting to the most expensive option.

### 5.5 Memory Guardrails

Memory is powerful and dangerous. The guardrails here complement the memory policy (§4.6) with hard enforcement:

- **No secrets in memory DB or exports.** The linter scans canonical exports for patterns that look like API keys, tokens, passwords, and connection strings. Violations block commits. The DB layer also rejects memory creation attempts that match secret patterns. Secrets belong in environment variables or secret managers, never in memory.
- **PII handling.** When agents encounter personally identifiable information (names, emails, phone numbers, addresses) in the course of their work, they don't persist it to memory unless explicitly instructed. The default is to use PII in-session and discard it. When persistence is needed, PII is flagged in the memory record so it can be audited and purged.
- **Cross-agent memory isolation.** Agent A cannot query Agent B's memory scope without explicit permission in both agents' configs. Shared memory spaces (like a team knowledge base) are opt-in, clearly delineated, and governed by the memory policy. This prevents accidental information leakage between agents with different trust levels or different humans.
- **Memory policy enforcement.** The rules in `MEMORY_POLICY.md` (§4.6) are not suggestions — they're enforced by the DB layer, linters on canonical exports, and CI gates. Schema conformance and export freshness are mechanically verified.

### 5.6 Channel Guardrails

Different channels have different trust levels, and the agent's behavior should reflect that.

- **Per-channel permission levels.** What the agent can do in `#general` (read-only, respond when mentioned) is different from what it can do in a DM with its human (full workspace access, proactive messaging). Channel configs define the permission ceiling.
- **Mention-gated channels.** In busy channels, agents should only respond when explicitly mentioned. The harness enforces this — messages in mention-gated channels that don't include the agent's name are not forwarded to the agent at all, saving tokens and preventing unwanted interjections.
- **Group chat behavior.** In group contexts, agents follow the principle of minimal intrusion: respond when asked, contribute when valuable, stay silent otherwise. The harness provides signals (mention detected, direct question, relevant topic) that help the agent decide, but the hard constraint is the mention gate.
- **No automatic cross-channel forwarding.** An agent cannot automatically forward messages between channels without explicit config. A message in a private DM stays in that DM unless the config says otherwise. Manual forwarding on human request is always allowed — if a human says "share this in #ai-collab," that's an explicit instruction, not a guardrail violation.

### 5.7 Guardrail Configuration

All guardrails are configured in a single, auditable file per agent: `GUARDRAILS.yaml` (or equivalent in the agent's workspace). This file is version-controlled, reviewable, and diffable. Changes to guardrails go through the same PR process as code changes.

The guardrails config supports inheritance: a base config defines org-wide defaults, agent-specific configs override where needed. This means you set sane defaults once and only customize per agent when there's a reason.

```yaml
# Example GUARDRAILS.yaml
tool_tiers:
  elevated:
    - deploy
    - send_email
    - financial_action
  system_wide:
    - shell_arbitrary
    - install_package
  
file_system:
  workspace_only: true
  allowed_external_paths: []

budgets:
  tokens_per_session: 500000
  tokens_per_task: 100000
  default_model_tier: standard  # haiku/flash

channels:
  "#general":
    tools: [read, message]
    mention_gated: true
  "dm":
    tools: [read, write, edit, shell, message]
    mention_gated: false

memory:
  cross_agent_access: deny
  pii_scan: true
  secret_scan: true

approval_gates:
  deploy: always          # always require human approval
  send_email: always
  financial_action: always
  public_comms: always    # tweets, posts, public PRs
  arch_decisions: always  # changes to ADRs, guardrails, soul

circuit_breaker:
  budget_velocity_pct: 50   # pause if >50% budget consumed...
  budget_velocity_window_s: 300  # ...in under 5 minutes
```

---

## 6. Security

Guardrails (§5) define operational constraints — what agents are *allowed* to do. Security addresses a different question: what happens when someone actively tries to make agents do things they shouldn't? The threat model for an agent harness is broad and largely novel. Agents sit at the intersection of untrusted input, powerful tools, and persistent state — an attacker who compromises an agent's reasoning effectively gains access to everything the agent can touch.

### 6.1 Threat Model

**Prompt injection** is the defining security challenge for agent systems. Unlike traditional software where code and data are cleanly separated, agents process natural language where instructions and content are the same medium. Every piece of untrusted content an agent encounters — a fetched web page, a message from a stranger in a group chat, a webhook payload, an email body, even the contents of a file someone shared — is a potential vector for injecting instructions that hijack the agent's behavior. Indirect prompt injection is particularly insidious: an attacker plants instructions in a web page they control, knowing an agent will eventually fetch and process it. The agent dutifully follows the injected instructions, believing them to be part of its task.

**Supply chain attacks on extensions** are the agent equivalent of malicious npm packages. A compromised SKILL.md, a malicious MCP server, or a tampered extension can execute arbitrary code with the agent's full permissions. The self-managing pattern we adopt from pi-mom — where agents install their own tools and configure their own credentials — amplifies this risk. An agent that fetches a skill URL (the nanobot pattern) and follows its setup instructions is executing an install script authored by an unknown party. Goose's approach of automatic malware scanning of external extensions before activation is a useful starting point, but scanning alone is insufficient for natural-language skill definitions that don't contain traditional malware signatures.

**Credential and data exfiltration** is a natural consequence of prompt injection. Once an attacker controls agent behavior (even partially), the obvious next step is extracting secrets: API keys from environment variables, tokens from config files, private data from the memory DB, conversation history from other channels. The exfiltration path is any tool that can send data outward — a web fetch to an attacker-controlled URL, a message to an external channel, even encoding secrets in seemingly innocuous output that the attacker reads later.

**Privilege escalation** occurs when an agent is tricked into using elevated tools it wouldn't normally invoke. A crafted message might convince an agent that a "routine task" requires shell access, or that an "urgent deployment" needs to bypass the approval gate. In multi-agent setups, there's an additional vector: agent impersonation, where one agent (or an attacker posing as an agent) issues instructions to another agent over the RPC layer (§7.1). Since agents treat other agents as collaborators, the trust boundary between agents needs cryptographic verification, not just convention.

**Model provider compromise** is the threat we can't fully control. If Anthropic/OpenAI's API is compromised or returns adversarial completions, the agent's reasoning itself is corrupted — every defense except output filtering, sandboxing, and egress controls is bypassed. The mitigation is layered: never send raw secrets in prompts (use tool-mediated access), filter outputs for credential patterns, sandbox tool execution, and restrict network egress. This won't stop a sophisticated provider-level attack, but it limits blast radius. Worth acknowledging honestly: we depend on our providers' security, and there's no way around that.

**Sandbox escape** is the last line of defense failing. If agents execute tools in Docker containers (as OpenHands demonstrates and pi-mom recommends), the container boundary is the security perimeter. Container escapes are well-studied but still occur — kernel exploits, mounted sockets, excessive capabilities. An agent that can manipulate its own container configuration, or that runs with elevated container privileges, has a wider escape surface.

### 6.2 Defense-in-Depth

No single defense stops a determined attacker. The harness layers multiple independent defenses so that any single failure doesn't cascade into full compromise.

**Input sanitization and content marking.** All content from external sources enters the agent's context wrapped in explicit untrusted markers. OpenClaw's `EXTERNAL_UNTRUSTED_CONTENT` pattern — where fetched web pages, incoming messages from non-trusted users, webhook payloads, and file contents are demarcated with source attribution — is the baseline we adopt. The key insight is that marking alone doesn't prevent injection (the model might still follow injected instructions), but it makes the attack visible in audit logs, enables post-hoc analysis, and gives the model a fighting chance at distinguishing data from directives. Content from the agent's own workspace and from its primary human bypasses untrusted marking; everything else is untrusted by default.

**Output filtering.** The harness scans agent output before delivery for patterns that look like credentials, API keys, connection strings, or private data. This is the same secret-scanning linter from §5.5, applied in real-time to outgoing messages and tool call parameters. If an agent is tricked into exfiltrating an API key via a web request, the output filter catches the key pattern in the URL and blocks the call. This is a probabilistic defense — novel encoding schemes can evade pattern matching — but it catches the low-hanging fruit that constitutes most real-world exfiltration attempts.

**Sandboxing.** v1 uses filesystem boundaries only — agents are restricted to their workspace directory, enforced by the tool layer. Container-level sandboxing (Docker isolation with minimal capabilities, no network unless required, non-root execution) is Phase 2. We're explicit about this because overpromising on security is worse than underpromising. OpenHands validates container sandboxing at scale, and pi-mom recommends it as the production model — we adopt it when the core is stable, not before. When we do, the default sandbox profile will drop all Linux capabilities except those explicitly needed, mount the workspace read-write but nothing else, and prevent agents from modifying their own sandbox configuration.

**Extension vetting.** Skills and MCP servers are the primary supply chain attack surface. The harness implements a three-tier trust model for extensions: *built-in* (shipped with the harness, fully trusted), *allowlisted* (reviewed and approved by the operator, hash-pinned), and *untrusted* (everything else, runs with reduced permissions and no access to secrets). Goose's malware scanning is adopted for the untrusted tier, but for allowlisted extensions, we go further: cryptographic signing of skill packages, hash verification on load, and a review process that mirrors code review. MCP servers from unknown sources run in their own sandboxed process with no access to the agent's memory or credentials — they can only communicate through the MCP protocol's defined message types.

**Network egress controls.** Agents should not be able to POST data to arbitrary endpoints. The sandbox's network policy defaults to deny-all-outbound, with an allowlist of permitted destinations (API providers, git remotes, configured messaging surfaces). This is the single most effective defense against data exfiltration: even if an attacker fully controls the agent's reasoning, they can't send data anywhere that isn't pre-approved. The allowlist is operator-configured and auditable, not agent-modifiable.

**Audit logging.** Every tool call, every external action, every permission escalation, and every approval gate decision is logged to an append-only audit trail. Agents cannot modify or delete their own audit logs (this is enforced by the logging subsystem writing to a location outside the agent's workspace, or by using append-only file attributes). The audit log captures: timestamp, tool name, full parameters, result summary, source of the instruction (which message triggered this action), and whether untrusted content was in the active context. This makes forensic analysis of security incidents tractable — "what did the agent do after processing that email?" is always answerable.

**Principle of least privilege.** Agents start with the minimal permission set for their configured role. A research agent gets read-only tools and web search. A development agent gets workspace write access and shell. An operations agent gets deployment tools. Permissions are additive and explicit — an agent never has a capability it wasn't specifically granted. Elevation requests go through the same approval gates defined in §5.1, but with an additional check: if the elevation request originated from processing untrusted content, the gate flags this to the human approver. This doesn't prevent all privilege escalation, but it ensures that the human making the approval decision knows the request's provenance.

### 6.3 Multi-Agent Security

When agents communicate over the RPC layer (§7.1), every message carries a cryptographic signature tied to the sending agent's identity. Agents verify signatures before processing RPC messages. This prevents impersonation — a compromised agent or an external attacker cannot forge messages from another agent. The coordination layer (messaging surfaces) is inherently human-visible and thus self-auditing, but RPC is agent-to-agent and needs this cryptographic accountability.

Cross-agent memory isolation (§5.5) is also a security boundary: even if Agent A is compromised, it cannot query Agent B's memory scope or credentials. The blast radius of a single compromised agent is limited to that agent's workspace, permissions, and network allowlist.

### 6.4 Security Posture Over Time

Security is not a launch checklist — it's an ongoing posture. The harness should support:

- **Periodic permission audits.** A maintenance agent (§7.3) reviews each agent's actual tool usage against its granted permissions and flags over-provisioned agents.
- **Anomaly detection.** If an agent that normally makes 5 tool calls per session suddenly makes 50, or starts accessing files it's never touched before, the circuit breaker (§5.4) should trigger.
- **Incident response playbook.** When a security event is detected (output filter catches a credential, audit log shows unexpected elevated access), the harness should have a documented response: halt the agent, preserve the audit log, alert the operator, and quarantine the workspace for analysis.

The uncomfortable truth is that agent security is an unsolved problem industry-wide. Prompt injection has no complete defense today — only mitigations that raise the cost of attack. Our strategy is defense-in-depth: assume any single layer will be bypassed, and design so that the next layer catches it. The combination of input marking, output filtering, sandboxing, network egress controls, and comprehensive audit logging creates a security posture where attacks are detectable and containable, even when they can't be perfectly prevented.

---

## 7. Multi-Agent Primitives (Phase 2 core)

### 7.1 Agent-to-Agent Communication

Two channels, by design:

- **RPC layer** — structured data transfer between agents (task handoffs, results, typed payloads). Logged but not surfaced to humans by default.
- **Coordination layer** — messaging surface (Slack, etc.) for decisions, status updates, and human-visible collaboration. Humans see what agents are deciding.

Think: Slack is the standup, RPC is the API call.

- **Handoff protocol** — typed task handoffs with context, constraints, and expected output format
- **Shared artifacts via git** — code and specs through PRs, coordination through messaging

Two communication channels means agents must decide which to use — the convention is simple: data goes over RPC, decisions go to the coordination layer. This means agents become accountable: their reasoning is auditable in Slack history, which feeds into decision logging (§10). Git-based shared artifacts mean agent work is reviewable through the same PR process as human work.

### 7.2 Governance Model

- **Humans:** direction, veto power, merge authority on architectural decisions
- **Agents:** first-class contributors (draft RFCs, write code, review PRs), influence but not authority
- **Trust model:** each agent trusts its primary human fully, trusts other team humans for project scope, treats other agents as collaborators (not instructors)
- **RFC process:** any significant change gets a written proposal; agents can author, humans approve

### 7.3 Specialized Agent Roles

- Reviewer agents (security, performance, architecture, data integrity)
- Research agents (repo analysis, framework docs, best practices)
- Maintenance agents (doc gardening, drift detection, cleanup)
- Roles are skill-based, not hardcoded — any agent can load any role

---

## 8. Messaging Surface Abstraction

Channel-agnostic messaging is our **core differentiator** — no existing framework provides this.

- **Unified interface** — single API for Slack, Telegram, Signal, Discord, IRC, etc.
- **Surface-specific formatting** — auto-adapt output (no markdown tables on Discord, wrap links on WhatsApp, etc.)
- **Channel policies** — allowlists, denylists, requireMention, allowBots per channel
- **Proactive messaging** — heartbeat + cron system for agents that act without being asked

> **Research insight:** nanobot's channel gateway pattern is the closest prior art — a central `nanobot gateway` process multiplexes across 9+ chat platforms (Telegram, Discord, WhatsApp, Slack, Email, QQ, Feishu, DingTalk, Mochat) via config-driven adapters. We study this architecture deeply and build with strong typing and plugin isolation in Rust.

Channel abstraction means agents are platform-independent — migrate from Slack to Discord without touching agent logic. It also makes cross-platform coordination natural: an agent can receive a task on Slack and report results on Telegram. Worth noting: platform-specific formatting rules create a growing compatibility matrix that needs automated tests per platform to catch regressions.

---

## 9. Collaboration Interface

### 9.1 Two Layers: Repo-Local + External Adapters

The same principle that drives our messaging abstraction (§8) applies to project management: **don't hardcode GitHub, Linear, or Jira — abstract them.**

| Layer | What | Examples |
|---|---|---|
| **Repo-local (always available)** | Task files, ADRs, backlog — works offline, no API needed | [Backlog.md](https://github.com/MrLesk/Backlog.md), `docs/adr/` |
| **PM adapter (pluggable)** | Syncs repo-local tasks to external trackers | GitHub Issues, Linear, Jira |

The harness depends on the repo-local layer only. PM adapters are optional plugins that sync outward.

### 9.2 Repo-Local Task Management (Backlog.md)

[Backlog.md](https://github.com/MrLesk/Backlog.md) is the agent-native task layer:

- **Each task = individual markdown file** in `backlog/` → zero merge conflicts by design
- **AI-native** — MCP + CLI integration, agents create/pick up/complete tasks naturally
- **Kanban board** in terminal (`backlog board`) or web (`backlog browser`)
- **100% repo-local, git-friendly** — no API keys, no external dependencies
- **Ownership rule:** the agent _assigned_ to a task updates its status. Unassigned items are updated by whoever picks them up.

This is the source of truth for day-to-day agent work. Always available, even offline.

### 9.3 PM Adapters (Phase 2)

Optional adapters sync Backlog.md tasks to external project management tools:

- **GitHub adapter** — sync to GitHub Issues/Projects. Agents participate in Discussions, author PRs.
- **Linear adapter** — sync to Linear. SHQ's current PM tool.
- **Jira adapter** — sync to Jira. Enterprise teams.

v1 adapters are **one-way (repo → external)**: task changes in `backlog/` push to the PM tool. Bidirectional sync (external → repo) is Phase 3 — it introduces conflict resolution complexity that isn't worth solving before the core is stable.

### 9.4 Conventions

- **Agents are first-class contributors** — they author PRs, comment on tasks, participate in discussions regardless of which PM tool is in use.
- **Labels/tags** distinguish human-created vs agent-created tasks (`source:human`, `source:agent`).
- **ADRs** (`docs/adr/`) are always repo-local, never synced to external tools (decisions live in git).

Repo-local as source of truth means the harness works without any external service — clone the repo, you have everything. PM adapters as plugins means teams aren't locked into our tool choices. This creates a consistent extension pattern that mirrors the messaging abstraction: adapters for everything.

---

## 10. Decision Capture Protocol

Decisions happen in conversations (Slack, GitHub Discussions, PRs). The record lives in the repo.

### 10.1 Architecture Decision Records (ADRs)

Location: `docs/adr/NNNN-title.md`

```markdown
# ADR-0001: Rust as primary language

**Status:** Accepted
**Date:** 2026-02-17
**Deciders:** Yao, Kani, Rem

## Context
We need a runtime language for the agent harness. Key requirements: single-binary deployment, high concurrency for the Channel/Branch/Worker/Cortex process model, and zero runtime dependencies. Spacebot validates Rust in the same domain.

## Decision
Rust. Single binary, zero deps, strong type system, native async for concurrent process model.

## Consequences
- Slower iteration speed vs TypeScript/Python — mitigated by constraining scope and reusing crates
- Team needs Rust proficiency — hiring/ramp-up consideration
- Excellent performance and reliability characteristics
- No Node.js ecosystem dependency

## Alternatives Considered
- TypeScript/Node.js — faster iteration, larger ecosystem, but runtime dependency and weaker concurrency model
- Python — strong AI/ML ecosystem but poor concurrency and deployment story
```

### 10.2 Explicit Capture Trigger

Decisions are captured via **explicit trigger**, not magic detection:

- **Slack:** React with 📋 (clipboard emoji) on a message containing a decision. The agent creates an ADR draft as a PR.
- **GitHub Discussion:** Label a comment with `decision`. The agent extracts and drafts an ADR.
- **CLI:** `harness decision "We chose X because Y"` — creates ADR directly.
- **Agent-initiated:** When an agent recognizes it's making a significant choice, it drafts an ADR and requests human approval before merging.

### 10.3 What Qualifies as a Decision

Not everything is an ADR. Capture when:
- An architectural or design choice is made that constrains future options
- A technology, library, or pattern is chosen over alternatives
- A convention or policy is established
- A significant tradeoff is accepted

Explicit triggers mean no false positives — decisions are captured intentionally, not guessed at. Git-based ADRs mean decisions are reviewable, revertible, and linkable, so "why did we do X?" is always answerable. And because agents draft the ADRs, the capture cost is near-zero: react with an emoji, get a PR.

---

## 11. Proactive Automation

- **Heartbeat system** — periodic check-ins, batched checks (email, calendar, mentions)
- **Cron scheduler** — exact timing, isolated sessions, different models per task
- **Compound loop integration** — recurring agents that run Plan → Work → Review → Compound on maintenance tasks

---

## 12. Deployment Model

- **Per-agent daemon** — each agent runs as its own process (like OpenClaw today). Keeps isolation simple.
- **Local-first** — runs on user's machine or a VPS. No mandatory cloud dependency.
- **Agent discovery** — agents register with a lightweight coordinator (config file or local service) that maps agent IDs to endpoints. For v1, this can be a shared JSON file in the repo.
- **Containerized deployment (optional)** — Docker Compose for multi-agent setups. Each agent = one container. Shared network for RPC, messaging surface for human-visible coordination.
- **Single-machine multi-agent** — v1 target. Distributed multi-machine is Phase 3.

---

## 13. Human Interaction & Team UX

Building an agent harness is half the problem. The other half is making it feel natural for humans to work alongside agents every day. This section addresses the full experience — from the first five minutes with a new agent to the daily rhythms of a team where agents are genuine collaborators.

### 13.1 Onboarding: The First Five Minutes

A new team member's first interaction with the harness should feel like meeting a colleague, not configuring a server. The agent introduces itself — its name, what it's good at, what it won't do, how it prefers to communicate. This identity lives in `SOUL.md` (personality, tone, boundaries) and `AGENTS.md` (capabilities, workspace conventions), following the pattern OpenClaw established and pi-mom independently converged on with its per-channel workspace model. The agent reads these files at session start and behaves accordingly, which means onboarding a new human is as simple as pointing them to the right channel. The agent already knows who it is.

Configuration during onboarding should be conversational where possible. "Hey, connect to my Slack workspace" beats editing a YAML file for most people. But the conversational setup generates the same config files that a power user would write by hand — there's one source of truth, and it's always a file in the repo. Goose's custom distributions pattern is instructive here: teams can create pre-configured agent profiles (a "research agent" distribution, a "dev agent" distribution) so new members pick a starting point rather than building from scratch.

The first task the agent completes should be small, visible, and useful — summarize a channel's recent history, set up a daily standup reminder, or review a PR. This builds trust through demonstration, not documentation.

### 13.2 Team Dynamics and Shared Agents

The default model is one primary agent per human, with shared agents for team-wide functions (a team standup bot, a shared research agent, a maintenance agent that gardens docs). Primary agents know their human deeply — preferences, communication style, work patterns. Shared agents are more formal, more conservative, and more explicit about what they're doing and why.

An agent behaves differently depending on context. In a 1:1 DM with its primary human, it's informal, proactive, and has full workspace access. In a group channel, it follows the minimal intrusion principle from §5.6 — respond when mentioned, contribute when valuable, stay silent otherwise. In a cross-team channel where it encounters unfamiliar humans, it defaults to read-only behavior until explicitly engaged. These aren't separate modes to configure; they emerge naturally from the channel policies already defined in `GUARDRAILS.yaml`.

Handoff is a real problem. When a human goes on vacation, their agent doesn't go dark — it continues background work (heartbeats, monitoring, cron tasks) and can be temporarily reassigned to a covering team member. The covering human gets a context briefing: what the agent has been working on, what's pending, what decisions are waiting. This is why canonical memory exports exist — they're not just for the agent's benefit, they're the handoff document. A new team member who wants to understand what an agent has been doing reads the exported memory artifacts in the repo, the same way they'd read a colleague's handover notes.

### 13.3 Daily Workflow

A typical day with the harness looks like this: the agent sends a morning summary (overnight activity, upcoming calendar events, pending items), the human reviews and prioritizes, the agent executes throughout the day with periodic check-ins, and an end-of-day summary captures what was accomplished. The heartbeat system (§11) and cron scheduler handle the timing; the messaging surface (§8) handles the delivery.

Reviewing agent work follows existing developer workflows. Code changes come as PRs that humans review normally. Task completions update Backlog.md entries with a summary of what was done. Decisions that need approval are surfaced explicitly through the approval gates in §5.3 — the agent doesn't bury important decisions in a wall of text, it flags them clearly and waits.

Knowing when to interrupt versus when to stay quiet is one of the hardest UX problems. The harness provides graduated urgency levels: silent (log it, don't message), normal (message in the relevant channel, no notification), urgent (direct message with notification), and critical (multi-channel alert). The default is conservative — most things are normal or silent. Agents learn over time which topics their human considers urgent, and those preferences are stored as Preference memories in the DB and exported to repo.

### 13.4 Error Communication

When an agent fails a task, how it communicates that failure is a UX problem, not a technical one. Agents that fail silently or retry in loops are the worst experience. The harness enforces a failure communication protocol:

1. **Explain what failed and why** — in plain language, not stack traces. "I couldn't deploy because the API token expired" not "Error: 401 Unauthorized at line 47."
2. **Say what was tried** — "I attempted to refresh the token, then tried the backup endpoint." The human needs to know the agent didn't just give up immediately.
3. **Suggest what the human can do** — "You can regenerate the token at Settings → API Keys, or I can try again with a different approach."
4. **Don't retry silently on ambiguous failures** — if the failure might be a misunderstanding of the task (not just a transient error), the agent stops and asks rather than burning tokens in a loop. Transient failures (rate limits, network blips) get automatic retry with backoff. Semantic failures (wrong file, unclear instructions, permission denied) get escalated to the human.

This is enforced at the harness level, not left to individual agent prompts. The tool execution layer wraps failures in a structured format that the agent must translate into human-readable communication before responding.

### 13.5 Configuration UX

The harness is CLI-first. `harness init` scaffolds a workspace, `harness agent create` sets up a new agent, `harness connect slack` wires up a messaging surface. Power users live in config files and the terminal. This is the primary interface and it needs to be excellent.

But not everyone on a team is a CLI person. A lightweight web dashboard — read-heavy, showing agent status, recent activity, cost summaries, and config — is Phase 2. It reads the same config files the CLI writes; it's a view layer, not a separate system. The hard constraint is that config files remain the source of truth. The dashboard can display and edit them, but it never maintains separate state.

For teams running multiple agents, shared configuration lives in the repo: base `GUARDRAILS.yaml` with per-agent overrides, shared `MEMORY_POLICY.md`, team-wide skill definitions. Git handles versioning and review. This is the same repo-native philosophy from §9 applied to agent configuration — if it's not in the repo, it doesn't exist.

### 13.5 Accessibility and Reach

Voice interaction is already partially solved — OpenClaw's TTS/STT capabilities mean agents can speak and listen, not just read and write. The harness should preserve this: voice notes in Telegram, spoken summaries, audio briefings. For some humans, talking to their agent while commuting is more natural than typing, and the messaging surfaces that support voice (Telegram, WhatsApp, Discord) are already in our adapter roadmap.

Mobile-friendly surfaces are a given, not a feature. Telegram, WhatsApp, Signal, and Discord are already mobile-native. The harness doesn't need a mobile app — it needs messaging adapters that work well on small screens. This means concise responses, progressive disclosure (summary first, details on request), and respecting platform conventions for formatting and interaction.

---

## 14. Observability & Monitoring

You can't manage what you can't see, and agent systems are particularly opaque. An agent that silently burns $200 on a retry loop, or one whose memory index has drifted out of sync, or one that's been down for three hours because a messaging adapter lost its connection — these are not hypothetical failures, they're Tuesday. Observability is how we catch them before they compound.

### 14.1 What to Observe

The basics are token usage and cost, tracked per session, per task, per agent, and per day. The budget system in §5.4 enforces ceilings, but observability tells you *where* the money is going. A research sub-agent that burns 80% of the daily budget on a single query is a pattern you need to see to fix.

Beyond cost: tool call frequency and latency (is the shell tool hanging? is the web fetch timing out consistently?), error rates by type (provider rate limits, tool failures, guardrail violations), memory DB health and vector index freshness (is the index stale? are canonical exports lagging?), and agent activity patterns (when is it active, what's it doing, how fast is it responding?). For multi-agent setups, add coordination metrics: RPC handoff success rates, messaging latency between agents, message volume in shared channels.

None of this requires a fancy dashboard on day one. It requires structured data that's queryable after the fact.

### 14.2 Structured Logging

Every significant action the harness takes gets a structured JSON log entry: every LLM call (model, tokens in/out, latency, cost), every tool invocation (tool name, parameters, result summary, duration), every message sent or received (channel, direction, length), every external action (API calls, git operations, file writes outside workspace). This is not optional and it's not debug logging you turn on when something breaks — it's the always-on foundation that everything else builds from.

The audit trail from §5.3 is a subset of this: the append-only, tamper-resistant log of actions that can't be modified by the agent itself. Observability extends the audit trail with performance data (latency, cost) and health signals (error counts, retry patterns) that aren't security-critical but are operationally essential.

Pydantic AI's tight integration with OpenTelemetry via Logfire validates the approach of instrumenting the agent loop at the framework level rather than bolting it on later. We adopt OpenTelemetry hooks in v1 — not the full distributed tracing stack, but the instrumentation points that make it trivial to connect later. Spans for LLM calls, spans for tool executions, span attributes for token counts and costs. The cost of adding these hooks early is near-zero; the cost of retrofitting them into a system that wasn't designed for them is significant.

### 14.3 CLI Observability

The primary interface for observability in v1 is the CLI:

- `harness status` — which agents are running, which adapters are connected, uptime, current session info
- `harness costs [--today|--week|--agent <name>]` — token usage and estimated cost, sliceable by time and agent
- `harness activity [--agent <name>]` — recent tool calls, messages, and actions in reverse chronological order
- `harness health` — checks agent process, messaging adapter connections, cron scheduler state, memory index freshness

Cost alerts deserve special mention. A simple threshold — "notify the human if total spend across all agents exceeds $X in a 24-hour period" — catches runaway loops and unexpected usage spikes. The circuit breaker in §5.4 catches velocity (burning budget too fast); cost alerts catch volume (spending too much overall). Together they cover the two main failure modes.

### 14.4 Audit Trail and Traceability

Every action should be traceable back to the conversation that triggered it. When an agent modifies a file, the log entry includes the session ID and message ID that led to that action. When something goes wrong — a bad deployment, an incorrect email, a file overwritten — the operator can trace from the action back to the instruction, back to the context the agent was working with. This is the same audit trail from §5.3 and §6.2, surfaced here as an observability concern because it serves both security and operational needs.

For compliance-sensitive environments, the structured logs are exportable: JSON lines that can be ingested by any log aggregation system, filtered by agent, time range, action type, or cost threshold. The harness doesn't need to build log aggregation — it needs to produce logs in a format that works with existing tools.

### 14.5 Phased Rollout

v1 ships with structured JSON logging, CLI status commands, and per-session cost tracking. This covers the "what just happened and how much did it cost" questions that dominate early operations.

Phase 2 adds OpenTelemetry integration (connecting the hooks from v1 to actual collectors), a lightweight web dashboard for teams who want visual monitoring, and configurable alerting (cost thresholds, error rate spikes, agent downtime). Mastra's built-in observability layer and CrewAI's AMP dashboard both validate that teams eventually want visual monitoring — but both also demonstrate that it's a product in itself, not a weekend project. We defer the dashboard until the data model is proven through CLI usage.

Phase 3 extends to cross-agent observability: how are agents interacting, where are the bottlenecks in multi-agent workflows, what's the team-wide cost profile? This only matters when we're running enough agents that individual monitoring doesn't scale. By then, the structured logging and OpenTelemetry foundations from earlier phases make this tractable rather than heroic.

---

## 15. What We Deliberately Omit (v1)

- **Plugin marketplace** — skills are git repos; discovery is manual until scale demands more
- **Visual agent builder** — no drag-and-drop; code-first
- **Notion/Jira/external trackers** — repo-native or nothing (PM adapters are Phase 2)

---

## 16. Success Criteria

1. Single agent running on new core, talking on one surface (end of week 3)
2. Two agents collaborating through new system (end of week 4)
3. Feature parity with OpenClaw features we actually use (end of week 6)
4. All SHQ agent operations migrated off OpenClaw (end of week 8)
5. MEMORY_POLICY.md enforced by linter (end of week 3)
6. First ADR captured via emoji trigger (end of week 4)
7. Hybrid recall over memory DB (vector + full-text) returning relevant results (end of week 5)

---

## 17. Critical Fork: Build vs. Wrap — RESOLVED

### Decision: Build own core in Rust, port patterns aggressively.

The pi-agent-core wrapping option (TypeScript) is no longer applicable — we've committed to Rust as the runtime language. Spacebot validates Rust for this domain: single binary, zero deps, high concurrency for the Channel/Branch/Worker/Cortex process model.

**Patterns to port from research (language-agnostic):**

From pi-agent-core:
1. Event streaming architecture (granular events for UI, logging, inter-agent coordination)
2. Message flow pipeline (app messages → transform → LLM messages)
3. Steering and follow-up queues
4. Context compaction strategy
5. Dynamic model/tool swapping

From Spacebot:
1. Channel/Branch/Worker/Cortex process separation
2. Message coalescing for group chats
3. Model routing by process type (expensive for channels, cheap for workers)
4. Non-blocking compaction as background process
5. Typed memory graph with hybrid recall (RRF)

From Mastra:
- Working memory as structured, persistent state
- Observational memory pattern

From CrewAI:
- Composite memory scoring (semantic × recency × importance with tunable weights)

### Decision Status: **ACCEPTED** (Yao, 2026-02-17)

---

## 18. Key Decisions & Open Questions

### Resolved:
- **Language/runtime:** Rust — single binary, zero deps, validated by Spacebot in same domain. (Accepted 2026-02-17)
- **Memory store:** Structured DB (SQLite + vector store) with typed memory graph. Repo exports as canonical record. (Accepted 2026-02-17)
- **Process model:** Channel/Branch/Worker/Cortex — concurrent specialized processes. (Accepted 2026-02-17)
- **Cortex:** In v1, scoped to process supervision + health signals only. (Accepted 2026-02-17)
- **Core loop:** Build own, port patterns from pi-agent-core, Spacebot, Mastra, CrewAI. (Accepted 2026-02-17)
- **License:** Apache 2.0 — permissive with patent protection
- **Governance:** SHQ-owned to start, foundation later if traction warrants
- **Task tracking:** Repo-native (backlog.md + GitHub Issues/Projects). No Notion.
- **Decision records:** Git-based ADRs with explicit capture triggers

### Open:
- **Name?** Working title TBD. Short, memorable.
- **Messaging adapter architecture?** Study nanobot's gateway pattern, then design. Port OpenClaw's adapter pattern or design fresh?
- **Vector store choice?** Benchmark needed for Rust-native options (lance, qdrant-client, usearch).

---

## 19. Testing Strategy

Agent frameworks are notoriously hard to test. Our approach:

1. **Unit tests** — tool implementations, message formatting, memory read/write, vector index operations. Rust's built-in test framework.
2. **Integration tests** — messaging adapters (mock Slack/Telegram APIs), LLM abstraction (mock provider responses), session lifecycle (create → branch → merge → prune).
3. **Golden path E2E** — send message → agent processes → tools execute → response delivered → memory updated. One test per messaging surface. Run on every PR.
4. **Replay tests** — record real agent sessions, replay against new code to catch regressions in behavior (not just API contracts).
5. **Memory integrity** — vector index matches file content after writes, concurrent agent writes don't corrupt state.

Tests are not Phase 2. The golden path E2E ships with the first prototype (Week 3).

---

## 20. Timeline

| Week | Milestone |
|------|-----------|
| 1 | Audit: what OpenClaw features we actually use + gap analysis. Set up repo with GitHub Projects, ADR template, MEMORY_POLICY.md. |
| 2 | Architecture decisions: Rust async model (ADR-0002), messaging adapter design (ADR-0003), vector backend selection (ADR-0004). |
| 3 | Prototype: single agent on new core, one messaging surface, memory policy enforced by linter. |
| 4 | Multi-agent: two agents collaborating. Decision capture via emoji trigger working. GitHub Issues/Projects integration. |
| 5 | Memory: vector index over memory files. Compound capture loop. Dogfood one real SHQ task end-to-end. |
| 6-7 | Feature parity with used OpenClaw features (informed by dogfood). Second messaging surface. |
| 8-9 | Migration: move SHQ operations to new system. Retrospective ADR on what worked/didn't. |

---

## 21. Uncaptured Research Patterns

These patterns emerged from our 9-framework audit but aren't yet woven into the architecture sections above. Each is validated by at least one production framework. They form a checklist for implementation — some are v1, some are later phases.

### 21.1 Context Compression (OpenHands, Pi/mom)

When sessions exceed the context window, automatically summarize older turns to preserve the most relevant information. Pi-mom uses a `log.jsonl` (full history, source of truth) + `context.jsonl` (what the LLM sees) split, with compaction when context grows too large. OpenHands applies aggressive context compression to keep agents functional across long tasks. Without this, agents degrade silently — they lose early context and start making mistakes they wouldn't make with full history. This belongs in §4.1 (Session Model) and should be a v1 feature, not an optimization. The compaction strategy matters: summarize tool outputs more aggressively than human instructions, preserve decisions and their rationale, and always keep the system prompt and recent turns intact.

### 21.2 Validation-Retry Loop (Pydantic AI)

When tool arguments fail schema validation, send the validation error back to the LLM for self-correction instead of failing hard. Pydantic AI does this with schema validation — if the agent produces malformed tool calls, the error message becomes part of the next prompt, and the agent fixes its own mistake. This is a small implementation detail with outsized impact on reliability. In Rust, serde + the type system handle validation; the retry behavior should be default in the tool execution layer — configurable max retries (default: 3), with the validation error formatted to help the LLM understand what went wrong.

### 21.3 Structured Output Guarantee (Pydantic AI)

Beyond validating tool *inputs*, the harness should support validating agent *outputs*. When a task requires structured data (JSON, typed objects), the agent retries until its output matches the expected schema. This is the "harness-first" principle (§3.1) applied to outputs — the scaffolding guarantees correctness, not the LLM's good behavior. Useful for: API responses, data extraction, structured reports, and any agent-to-agent communication where the receiving agent expects a typed payload.

### 21.4 Deferred Approval Pattern (Pydantic AI)

The current approval gates (§5.3) block the agent until a human responds. Pydantic AI has a better pattern: tools can return "pending human approval" as a first-class result type. The agent acknowledges the pending action and continues working on non-blocked tasks. When the human approves, the deferred action executes and results flow back into the agent's context. This is strictly better than blocking — the agent stays productive while waiting for approval, and humans don't feel pressured to respond immediately.

### 21.5 Action/Observation Formalism (OpenHands)

OpenHands separates agent behavior into Actions (what the agent wants to do) and Observations (what the environment reports back). This is more formal than "tools return results" and creates a clean audit trail: every agent turn produces a list of Actions, each Action produces an Observation, and the full Action→Observation chain is logged, replayable, and analyzable. This maps naturally to our audit logging (§14) and replay tests (§19). Worth adopting as the internal execution model even if the surface API stays simple.

### 21.6 Typed Dependency Injection (Pydantic AI)

Tools need runtime context — database connections, API clients, user identity, configuration. Pydantic AI's `RunContext[Deps]` pattern passes these as typed dependencies rather than global state or environment variables. In Rust, this maps naturally to generic tool definitions with typed context parameters, leveraging the trait system for dependency injection. This matters for testing (inject mocks instead of real services), for multi-tenant setups (different deps per user), and for security (tools only see the deps they're given, not everything in the environment).

### 21.7 Custom Agent Distributions (Goose)

Goose supports "custom distributions" — pre-configured agent profiles with specific tools, extensions, and system prompts baked in. This maps to our Kani family pattern (personas) and specialized roles (§7.3), but Goose packages them as shareable, installable configurations. A `harness create --template research` or `harness create --template dev-lead` would dramatically improve onboarding. Templates are just config bundles (SOUL.md + AGENTS.md + GUARDRAILS.yaml + default skills) in a git repo or registry. The harness ships with a few starter templates; teams create their own.

### 21.8 Remote Skill Discovery (nanobot)

nanobot agents can read a skill from a URL and self-configure — no manual installation. The agent is told "read https://example.com/skill.md and follow the instructions," and it does. This is lighter than a marketplace and more flexible than bundled skills. For our harness, a curated skill registry (even just a JSON file mapping skill names to git URLs) gives agents discoverability without infrastructure. The agent can `harness skill add <url>`, which clones the skill repo into the workspace and registers it. Remote skills are untrusted by default (§6.2 extension vetting applies).

### 21.9 Workflow Engine with Suspend/Resume (Mastra)

Mastra's workflow system is graph-based: define steps, connect them with edges, add conditional branching and human checkpoints. Crucially, workflows can suspend (waiting for human input, external event, or timer) and resume without losing state. The PRD mentions proactive automation (§11) but doesn't address complex multi-step workflows that span hours or days. For v1, simple cron + heartbeat is enough. But the architecture should anticipate durable workflows — a research task that runs overnight, pauses for human review in the morning, then continues based on feedback. The session tree model (§4.1) naturally supports this if branches can be suspended and resumed.

### 21.10 Benchmark-Driven Development (OpenHands)

OpenHands tracks performance against SWE-bench, giving them a quantitative regression safety net. We should define our equivalent from day one. Not a massive benchmark suite — just 10-15 standard scenarios that cover the critical paths:

1. Receive message → process → respond (basic round-trip)
2. Branch session → complete task → merge results back
3. Recover from crash mid-task (session recovery)
4. Capture decision via emoji trigger (ADR flow)
5. Respect guardrail boundary (refuse elevated action without permission)
6. Compress context without losing critical information
7. Multi-agent handoff (agent A delegates to agent B, gets result)
8. Cost tracking accuracy (reported cost matches actual API spend)
9. Memory write → vector index → semantic recall
10. Error communication (fail gracefully, explain clearly)

Run these on every PR. If a scenario regresses, the PR doesn't merge. This is the "mechanical enforcement" principle (§3.3) applied to the harness itself.

---

## References

- [OpenAI Harness Engineering](https://openai.com/index/harness-engineering/)
- [Compound Engineering (Kieran Klaassen / Cora)](https://every.to/guides/compound-engineering)
- [Spacebot (Spacedrive)](https://github.com/spacedriveapp/spacebot) — Rust agent harness, Channel/Branch/Worker/Cortex architecture, typed memory graph
- [Pi-AI architecture (pi-mono)](https://github.com/badlogic/pi-mono)
- [Mastra framework](https://mastra.ai) — TypeScript agent framework, strongest architectural match
- [CrewAI](https://docs.crewai.com) — Multi-agent orchestration, unified memory with composite scoring
- [nanobot](https://github.com/HKUDS/nanobot) — Channel gateway pattern across 9+ platforms
- [Goose (Block)](https://github.com/block/goose) — MCP-native extension architecture
- [Pydantic AI](https://github.com/pydantic/pydantic-ai) — Typed dependency injection, durable execution
- [OpenHands](https://github.com/All-Hands-AI/OpenHands) — Sandbox execution, context compression
- Framework comparison matrix: `docs/research/2026-02-16-framework-comparison.md`
- OpenClaw feature audit (to be completed Week 1)
