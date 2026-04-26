# ACP + NATS Integration Research

*Compiled: 2026-03-05*

## Executive Summary

There are **two distinct protocols** both abbreviated "ACP" in the agent ecosystem. This
research evaluates both against the CTO platform's existing NATS messaging architecture.

The conclusion is that **NATS and ACP serve different, complementary layers** and should
coexist. NATS remains the right transport for the CTO platform's multi-agent pub-sub
backbone. The Agent Client Protocol (Zed/JetBrains) is relevant for editor integration
but not for inter-agent messaging. The Agent Communication Protocol (IBM/BeeAI) has
merged into A2A and is no longer independently maintained, but its design patterns
(task lifecycle states, session persistence) can inform improvements to the NATS layer.

---

## 1. Key Findings: The Two ACPs

### 1a. Agent Client Protocol (Zed/JetBrains/Block)

| Attribute | Detail |
|-----------|--------|
| **Full name** | Agent Client Protocol |
| **Creators** | Zed Industries, JetBrains, Block (Goose) |
| **Purpose** | Editor-to-agent communication |
| **Transport** | JSON-RPC 2.0 over stdio (subprocess) or HTTP |
| **Status** | Active development (v0.11.0, March 2026) |
| **Spec repo** | https://github.com/agentclientprotocol/agent-client-protocol |
| **SDKs** | TypeScript, Rust, Kotlin, Python |

**Core lifecycle methods:**

```
session/initialize  -- capability negotiation
session/new         -- create a session with working directory + MCP config
session/load        -- resume an existing session
session/prompt      -- send user input to the agent
session/update      -- agent streams back results (notifications)
session/cancel      -- cancel in-progress work
```

**Update event types streamed via `session/update`:**

- `agent_message_chunk` -- incremental response text
- `agent_thought_chunk` -- reasoning/chain-of-thought
- `tool_call` -- tool invocation announcement
- `tool_call_update` -- tool execution result
- `plan` -- multi-step plan

**Key characteristics:**
- Designed for a 1:1 relationship: one editor, one agent subprocess
- Subprocess model -- agent runs as child process of the editor
- Session state lives in the agent process or filesystem (`~/.acpx/sessions/`)
- Not designed for multi-agent fan-out or pub-sub patterns
- Complements MCP: "MCP handles the *what* (tools/data), ACP handles the *where* (agent in your workflow)"

**acpx (OpenClaw implementation):**

acpx is a headless CLI client for the Agent Client Protocol. It wraps coding agents
(Codex, Claude Code, Gemini, OpenCode, Pi) behind a uniform ACP interface and
communicates via ndjson over stdio.

- Session persistence in `~/.acpx/sessions/*.json`
- Queue-aware prompt submission per session
- Cooperative cancellation preserving session state
- Named sessions for parallel workstreams
- NDJSON event envelope: `{ eventVersion, sessionId, requestId, seq, stream, type }`

### 1b. Agent Communication Protocol (IBM/BeeAI -- now merged into A2A)

| Attribute | Detail |
|-----------|--------|
| **Full name** | Agent Communication Protocol |
| **Creator** | IBM Research (BeeAI platform) |
| **Purpose** | Agent-to-agent communication |
| **Transport** | REST over HTTP |
| **Status** | **Archived** -- merged into Google A2A under Linux Foundation (Aug 2025) |
| **Spec repo** | https://github.com/i-am-bee/acp (read-only) |

**Run lifecycle states:**

```
created --> in-progress --> completed
                       --> failed
                       --> awaiting (paused for human input)
                       --> cancelling --> cancelled
```

**Key characteristics:**
- REST-based, no SDK required (curl works)
- Peer-to-peer: agents interact directly, no central manager needed
- Supports sync, async, and streaming execution modes
- Session-level persistence for multi-turn conversations
- MIME-type extensible (text, images, audio, video, binary)
- Offline discovery via embedded metadata in distribution packages

**Why it was archived:**
IBM and Google unified ACP and A2A under the Linux Foundation's LF AI & Data umbrella in
September 2025. The merger preserved ACP's RESTful simplicity while incorporating A2A's
enterprise features (Agent Cards, task lifecycle). Active development continues in the A2A
repo.

---

## 2. Architecture Comparison: ACP vs NATS Layers

### Layer Model

```
                   CTO Platform Stack
  ============================================================

  [Application]    Agent logic (bolt, rex, nova, etc.)
                   Lobster workflows, deliberation, voting

  [Session/        ACP-style session lifecycle            <-- NEW (proposed)
   Lifecycle]      Run states, delivery guarantees,
                   idempotency, event projection

  [Routing]        NATS subject hierarchy                 <-- EXISTING
                   agent.>, elicitation.>, agent.all.broadcast
                   Bridges: Discord, Linear

  [Transport]      NATS TCP connections                   <-- EXISTING
                   Reconnect, drain, cluster-aware

  ============================================================
```

### What NATS provides today

Based on the existing codebase (`apps/nats-messenger/`, `apps/discord-bridge/`,
`apps/linear-bridge/`):

| Capability | Implementation | Files |
|------------|---------------|-------|
| **Pub-sub routing** | `agent.>` wildcard subscriptions | `nats-tap.ts`, `client.ts` |
| **Request-reply** | `nc.request()` with timeout | `client.ts:request()` |
| **Discovery** | Ping-pong on `agent.all.broadcast` | `client.ts:discoverPeers()` |
| **Ping-pong guard** | Rate limiting per peer pair | `service.ts`, `tool.ts` |
| **Conversation tracking** | In-memory maps (RoomManager, IssueManager) | `room-manager.ts`, `issue-manager.ts` |
| **Elicitation** | Cross-platform HITL with cancel coordination | `elicitation-handler.ts`, `elicitation-types.ts` |
| **Multi-bridge fan-out** | Discord + Linear both subscribe to same subjects | `index.ts` in both bridges |
| **Session delivery** | `enqueueSystemEvent()` into OpenClaw runtime | `actions.ts` |

### What NATS does NOT provide (gaps)

| Gap | Description |
|-----|-------------|
| **Session lifecycle states** | No formal state machine (created/in-progress/awaiting/completed/failed/cancelled) |
| **Persistence** | All conversation state is in-memory; lost on bridge restart |
| **Delivery guarantees** | At-most-once delivery; no checkpointing or idempotency tokens |
| **Run tracking** | No concept of a "run" with unique ID and terminal state |
| **Event log** | No durable event stream for replay or audit |
| **Structured events** | Wire format is flat `AgentMessage`; no typed event envelope (text_delta, tool_call, done, error) |

### What ACP (Agent Client Protocol) provides that is relevant

| ACP Feature | CTO Relevance |
|-------------|---------------|
| Session lifecycle (`new`, `load`, `cancel`) | Could formalize conversation lifecycle |
| Typed event streaming (`agent_message_chunk`, `tool_call`, etc.) | Would improve Discord/Linear rendering |
| Session persistence (`~/.acpx/sessions/`) | Model for SQLite/filesystem-based durability |
| Cooperative cancellation | Already partially implemented via `elicitation.cancel` |
| Queue-aware prompt submission | Could prevent message loss during agent busy periods |

### What ACP (Agent Client Protocol) does NOT provide

| Missing for CTO | Why |
|-----------------|-----|
| Multi-agent fan-out | ACP is 1:1 (editor:agent), not 1:N or N:N |
| Pub-sub routing | No subject hierarchy, no wildcards |
| Cross-pod messaging | Designed for subprocess (stdio), not network |
| Bridge pattern | No concept of relaying to Discord/Linear/Slack |
| Agent discovery | No peer discovery mechanism |

---

## 3. Proposed Integration Approach

### Recommendation: ACP Patterns on Top of NATS (not replacement)

NATS remains the transport and routing layer. We adopt **ACP design patterns** to fill
the session lifecycle and durability gaps.

### Phase 1: Session Lifecycle State Machine

Add formal run states to the NATS messaging layer:

```typescript
// Proposed: packages/types/src/run-states.ts

export type RunState =
  | "created"       // Run submitted, not yet picked up
  | "in_progress"   // Agent actively processing
  | "awaiting"      // Paused for human input (elicitation)
  | "completed"     // Terminal: success
  | "failed"        // Terminal: error
  | "cancelled";    // Terminal: cancelled

export interface AgentRun {
  run_id: string;          // UUID
  session_id: string;      // Conversation/session ID
  agent: string;           // Agent name
  state: RunState;
  created_at: string;      // ISO timestamp
  updated_at: string;
  parent_run_id?: string;  // For delegated work
  metadata?: Record<string, string>;
}
```

Publish state transitions on a new NATS subject: `run.<agent>.<run_id>.state`

### Phase 2: Typed Event Envelope

Replace the flat `AgentMessage.message` string with structured events:

```typescript
// Proposed: packages/types/src/events.ts

export type AgentEventType =
  | "text_delta"      // Incremental text output
  | "tool_call"       // Tool invocation
  | "tool_result"     // Tool output
  | "thinking"        // Chain-of-thought
  | "plan"            // Multi-step plan
  | "state_change"    // Run state transition
  | "error"           // Error event
  | "done";           // Run completed

export interface AgentEvent {
  event_version: 1;
  session_id: string;
  run_id: string;
  seq: number;           // Monotonic sequence for ordering
  type: AgentEventType;
  data: unknown;         // Type-specific payload
  timestamp: string;
}
```

Bridges can use `type` to render events differently:
- `text_delta` -> streaming message update in Discord
- `tool_call` -> collapsed embed with tool name and args
- `state_change` -> status emoji update
- `error` -> red-highlighted embed

### Phase 3: Persistence Layer

Add optional SQLite persistence for conversation state:

```
~/.cto/sessions/
  <session-id>.sqlite
    - runs: (run_id, state, agent, created_at, updated_at)
    - events: (seq, run_id, type, data, timestamp)
    - delivery: (event_seq, bridge, delivered_at)  -- checkpoint for idempotency
```

This replaces the in-memory `Map<string, ConversationState>` in both bridges, surviving
restarts. Based on the acpx pattern of `~/.acpx/sessions/*.json` but with SQLite for
relational queries and atomic writes.

### Phase 4: ACP Relay Proxy (optional, deferred)

If we adopt acpx-compatible agents in the future, a thin relay proxy could translate:

```
NATS agent.<name>.inbox  <-->  ACP session/prompt (stdio/HTTP)
NATS run.<name>.*.state  <-->  ACP session/update (notifications)
```

This is **not recommended for Phase 1**. The CTO agents run as OpenClaw instances with
the NATS messenger plugin, not as ACP subprocess agents. An ACP relay only becomes
relevant if we want to integrate external ACP-native agents (e.g., Claude Code via acpx,
Codex CLI, Gemini CLI) into the NATS mesh.

---

## 4. Trade-offs

### Approach A: ACP Patterns on NATS (recommended)

| Pro | Con |
|-----|-----|
| Keeps existing NATS infrastructure untouched | Must implement lifecycle state machine ourselves |
| Multi-agent fan-out and pub-sub preserved | No off-the-shelf SDK (build from ACP patterns) |
| Bridges continue working with additive changes | Typed events require migration from flat `AgentMessage` |
| SQLite persistence is incremental (opt-in per bridge) | Additional storage and GC complexity |
| No new runtime dependency | -- |

### Approach B: Replace NATS with ACP-over-HTTP

| Pro | Con |
|-----|-----|
| Standard protocol with existing SDKs | Loses pub-sub: no wildcard subscriptions |
| Session lifecycle built-in | Loses multi-bridge fan-out (ACP is 1:1) |
| Community tooling (acpx, SDKs) | Must build custom routing layer on top |
| -- | Discovery protocol must be reimplemented |
| -- | Elicitation cross-cancel pattern breaks |
| -- | REST polling instead of push-based streaming |
| -- | Massive rewrite of all three bridges |

### Approach C: Dual Stack (NATS + ACP HTTP gateway)

| Pro | Con |
|-----|-----|
| Can integrate external ACP agents | Operational complexity (two protocols) |
| Gradual migration path | Consistency between NATS and ACP sessions |
| -- | Translation layer must handle impedance mismatch |

### Verdict

**Approach A** is the clear winner. ACP (Agent Client Protocol) was designed for editor-agent
integration, not multi-agent systems. ACP (Agent Communication Protocol / IBM) is archived
and merged into A2A. Neither provides the pub-sub, fan-out, and bridge patterns that NATS
gives us today.

The value is in adopting **ACP's design patterns** -- session lifecycle states, typed event
streams, persistence, cooperative cancellation -- without adopting ACP as a transport.

---

## 5. Comparison with Existing Research

The prior research note (`docs/2026-02/research-notes/agent-protocols.md)`, 2026-01-31) recommended
adopting ACP/A2A patterns for Morgan's agent coordination. This research **confirms and
refines** that recommendation:

| Prior Recommendation | Updated Position |
|---------------------|-----------------|
| "Adopt ACP/A2A patterns" | Confirmed -- adopt patterns, not the protocol itself |
| "Define agent metadata schema (discovery)" | Already implemented via NATS ping-pong discovery |
| "Implement task lifecycle" | **Highest priority** -- add `RunState` to NATS layer |
| "Add status streaming" | Implement as typed `AgentEvent` on NATS subjects |
| "Build Morgan as coordinator" | Morgan can use NATS + lifecycle states natively |

---

## 6. Next Steps if Adopted

### Immediate (can start now)

1. **Define `RunState` and `AgentEvent` types** in the shared types package
   (currently `apps/nats-messenger/types.ts`, moving to `@openclaw/nats-types`)
2. **Add `run_id` to `AgentMessage`** as an optional field for backward compatibility
3. **Publish state transitions** on `run.<agent>.<run_id>.state` NATS subjects
4. **Update bridge embeds** to render based on event type, not just flat message text

### Short-term (next sprint)

5. **Add SQLite session persistence** to Discord bridge as a prototype
6. **Implement delivery checkpointing** (idempotency for bridge restarts)
7. **Add `awaiting` state** integration with the existing elicitation protocol

### Medium-term (when openclaw-nats repo is extracted)

8. **Migrate types** to `@openclaw/nats-types` package per the extraction plan
9. **Add event replay** from SQLite for missed-message recovery
10. **Evaluate ACP relay proxy** if external agent integration is needed

### Deferred (monitor A2A evolution)

11. **Track A2A protocol** development (the ACP+A2A merger under Linux Foundation)
12. **Evaluate A2A Agent Cards** for agent discovery (may complement NATS ping-pong)
13. **Consider A2A task lifecycle** alignment if cross-org agent interop becomes a requirement

---

## 7. Reference Architecture

```
                              CTO Platform
  ====================================================================

                         +-----------------+
                         |  NATS Cluster   |
                         |  (transport)    |
                         +--------+--------+
                                  |
              +-------------------+-------------------+
              |                   |                   |
      agent.<name>.inbox   elicitation.>      run.<name>.*.state
              |                   |                   |
  +-----------+-----------+  +---+---+  +------------+------------+
  |                       |  |       |  |                         |
  |  OpenClaw Agents      |  |  HQ   |  |    Session Manager     |
  |  (bolt, rex, nova..)  |  | (web) |  |  (lifecycle + SQLite)  |
  |  nats-messenger plugin|  |       |  |                         |
  |                       |  +---+---+  +-----+----------+-------+
  +-----------+-----------+      |            |          |
              |                  |            |          |
              |         +--------+---+  +-----+-----+ +-+--------+
              |         |  Discord   |  |  Linear   | |  Future  |
              |         |  Bridge    |  |  Bridge   | |  Bridges |
              |         +------------+  +-----------+ +----------+
              |
      agent.all.broadcast
              |
      +-------+-------+
      | Discovery      |
      | (ping/pong)    |
      +----------------+
```

---

## Sources

- [Agent Client Protocol spec](https://agentclientprotocol.com/protocol/overview)
- [Agent Client Protocol GitHub](https://github.com/agentclientprotocol/agent-client-protocol)
- [acpx (OpenClaw headless CLI)](https://github.com/openclaw/acpx)
- [Intro to ACP: Agent-Editor Integration (Block/Goose)](https://block.github.io/goose/blog/2025/10/24/intro-to-agent-client-protocol-acp/)
- [JetBrains ACP documentation](https://www.jetbrains.com/help/ai-assistant/acp.html)
- [Zed ACP page](https://zed.dev/acp)
- [Agent Communication Protocol (IBM)](https://www.ibm.com/think/topics/agent-communication-protocol)
- [ACP agent run lifecycle](https://agentcommunicationprotocol.dev/core-concepts/agent-run-lifecycle)
- [ACP joins A2A under Linux Foundation](https://lfaidata.foundation/communityblog/2025/08/29/acp-joins-forces-with-a2a-under-the-linux-foundations-lf-ai-data/)
- [Agent protocol survey (arXiv)](https://arxiv.org/html/2505.02279v1)
- [ACPex protocol overview (Elixir)](https://hexdocs.pm/acpex/protocol_overview.html)
- [Top AI Agent Protocols 2026](https://getstream.io/blog/ai-agent-protocols/)
- [CTO prior research: agent-protocols.md](docs/2026-02/research-notes/agent-protocols.md))
- [CTO NATS extraction plan: openclaw-nats-extraction.md](docs/2026-03/openclaw-nats-extraction.md))
