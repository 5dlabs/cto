# Agent Coordination Plane MVP

Date: 2026-04
Status: Wave 2A design + isolated TypeScript skeleton
Scope: specification and testable type/addressing helpers only; no cluster mutation and no production bridge changes.

## Goal

Provide a runtime-neutral coordination plane for many Hermes pods and compatible adapters to exchange structured work messages without coupling agent UX, Discord presence, or OpenClaw-specific NATS tooling to a single bridge implementation.

The MVP defines:

- stable identities for agents and ephemeral agent groups;
- structured messages with explicit project/task/role/agent addressing;
- envelopes for transport metadata, correlation, delivery attempts, and tracing;
- deterministic NATS subject mapping that can coexist with existing `apps/nats-messenger` conventions.

## Current repository patterns observed

- `apps/nats-messenger` is an OpenClaw plugin using `nats` directly with JSON payloads.
  - Direct inbox subject convention: `agent.<agentName>.inbox`.
  - Broadcast convention: `agent.all.broadcast`.
  - Message shape includes `from`, optional `to`, `subject`, `message`, `priority`, `timestamp`, optional `replyTo`, `type`, `role`, and string metadata.
  - It includes request/reply, discovery ping/pong, static roster display, and a ping-pong guard.
- `apps/discord-bridge` treats agent messages as transport-agnostic HTTP POST payloads while optionally publishing presence fabric events to NATS.
  - Presence subjects use the `cto.presence.*` prefix, e.g. `cto.presence.in.<runtime>.<coderun>` and `cto.presence.route.<runtime>.<agent>`.
- Recent app conventions for small TypeScript packages:
  - private package under `apps/<name>`;
  - `type: module`, `main: ./dist/index.js`;
  - scripts: `build: tsc`, `test: node --test --import tsx src/*.test.ts`;
  - strict TypeScript, `moduleResolution: bundler`, declarations and source maps.

## Non-goals for Wave 2A

- Do not replace the production Discord bridge or OpenClaw `nats-messenger` plugin.
- Do not deploy NATS streams, JetStream consumers, Kubernetes CRDs, or controllers.
- Do not introduce persistence requirements beyond envelope fields that make persistence possible later.
- Do not define scheduling, task assignment, or agent lifecycle reconciliation policy beyond the message contract.

## Core domain model

### AgentIdentity

`AgentIdentity` identifies one running or desired agent capability. It should be stable enough for routing while allowing ephemeral pod/session metadata.

Required fields:

- `agentId`: stable logical name or instance id, e.g. `forge`, `planner-7`.
- `role`: coordination role, e.g. `planner`, `coder`, `reviewer`, `operator`.
- `runtime`: implementation/runtime, initially `hermes`, `openclaw`, `external`, or `unknown`.

Optional scoping fields:

- `projectId`
- `taskId`
- `sessionId`
- `podName`
- `model`
- `metadata: Record<string, string>`

### AgentGroup

`AgentGroup` is an addressable collection of agents. MVP groups are descriptive, not authoritative membership controllers.

Required fields:

- `groupId`
- `name`
- `members: AgentIdentity[]`

Optional fields:

- `projectId`
- `taskId`
- `role`
- `metadata`

Groups support common fanout patterns:

- all agents on a project;
- all agents on a project task;
- all agents with a role;
- explicit named group members.

### AgentAddress

An `AgentAddress` can target exactly one addressing mode:

- `{ kind: "agent", agentId, projectId?, taskId? }`
- `{ kind: "role", role, projectId?, taskId? }`
- `{ kind: "task", projectId, taskId }`
- `{ kind: "project", projectId }`
- `{ kind: "group", groupId }`
- `{ kind: "broadcast" }`

The skeleton enforces this as a discriminated union rather than a bag of optional fields.

### AgentMessage

`AgentMessage` carries semantic intent independent of the transport envelope.

Required fields:

- `messageId`
- `kind`: `command`, `request`, `response`, `event`, `status`, or `discovery`
- `from: AgentIdentity`
- `to: AgentAddress`
- `body`
- `createdAt` ISO timestamp

Optional fields:

- `priority`: `low`, `normal`, `high`, `urgent`
- `replyToMessageId`
- `correlationId`
- `metadata`

### AgentEnvelope

`AgentEnvelope` wraps the message for transport.

Required fields:

- `schema: "cto.agent.envelope.v1"`
- `envelopeId`
- `subject`
- `message`
- `sentAt`

Optional fields:

- `replyTo`
- `ttlMs`
- `attempt`
- `trace`

## Subject strategy

Use a new `cto.agent.v1.*` subject family for the coordination plane. Keep legacy `agent.<name>.inbox` and `agent.all.broadcast` for `apps/nats-messenger` compatibility adapters.

MVP subject mapping:

| Address kind | Subject |
| --- | --- |
| agent | `cto.agent.v1.agent.<agentId>.inbox` |
| role | `cto.agent.v1.role.<role>.inbox` |
| task | `cto.agent.v1.project.<projectId>.task.<taskId>.inbox` |
| project | `cto.agent.v1.project.<projectId>.inbox` |
| group | `cto.agent.v1.group.<groupId>.inbox` |
| broadcast | `cto.agent.v1.broadcast` |

Subject segments are normalized to lowercase `[a-z0-9_-]`, replacing other characters with `-`, trimming surrounding dashes, and falling back to `unknown` if empty. This mirrors the safe segment style already used by `apps/discord-bridge/src/presence-fabric.ts`.

## MVP API surface

The isolated skeleton exposes only pure helpers:

- `subjectForAddress(address)`
- `addressKey(address)`
- `createEnvelope(message, options)`
- Type definitions for `AgentIdentity`, `AgentGroup`, `AgentAddress`, `AgentMessage`, and `AgentEnvelope`

No network client is included in Wave 2A. A later wave can add a NATS adapter with the same subject mapping and a bridge adapter that translates between legacy `AgentMessage` and the new envelope.

## Compatibility path

1. Keep `apps/nats-messenger` unchanged.
2. Add an adapter that maps legacy direct messages:
   - legacy `agent.<to>.inbox` -> new `{ kind: "agent", agentId: to }`
   - legacy `agent.all.broadcast` -> new `{ kind: "broadcast" }`
   - legacy `message` -> new `body: { text: message }`
3. For Hermes control-plane workers, publish coordination messages on `cto.agent.v1.*` while continuing to emit presence on `cto.presence.*`.
4. Once consumers are verified, optionally teach `nats-messenger` to subscribe to both legacy and `cto.agent.v1.agent.<self>.inbox` subjects.

## MVP validation

The skeleton package includes node test-runner coverage for:

- deterministic direct agent subject mapping;
- explicit project/task/role addressing;
- broadcast addressing;
- envelope creation with schema, subject, timestamps, and correlation preservation;
- envelope runtime validation before future transport adapters trust messages from NATS/HTTP/worker boundaries;
- invalid/empty subject segment normalization fallback and malformed metadata rejection.

## Open questions for the next wave

- Should durable delivery use JetStream streams by project/task, or should this remain transient initially?
- What is the authoritative source for live agent membership: Kubernetes pods, Hermes CodeRuns, a CRD, or a presence registry?
- Should role fanout be queue-group load-balanced, broadcast fanout, or both via separate subjects?
- How should human-facing presence messages reference coordination envelopes for auditability?
