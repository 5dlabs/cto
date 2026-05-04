# Hermes-Parity Centralized Discord Control Plane Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Bring the CTO centralized Discord control plane to parity with the Hermes experience and make Hermes the reference runtime for Discord/Morgan workflows, while keeping OpenCloud/OpenClaw support in scope through the same runtime-neutral contracts.

**Architecture:** Discord remains centralized only at the credential boundary: `apps/discord-bridge` owns Discord login, slash-command registration, message/interaction normalization, route registry, inbound fanout, and outbound Discord effects. Hermes/OpenCloud/OpenClaw/hosted workers never receive Discord credentials; they register routes and exchange normalized control-plane events. Hermes is the gold-standard behavior source: sidecars, MCP tools, `/workspace` JSONL streams, lobster readiness steps, and CodeRun metadata define the target agent experience. OpenCloud/OpenClaw adapters should conform to the same contract, but should not force the centralized control plane to inherit gateway limitations.

**Tech Stack:** TypeScript/Node 20 (`discord.js`, `node:test`, `tsx`) for the bridge/adapters, Rust controller/CodeRun resources, Kubernetes GitOps, Hermes `lobster + ACPX`, MCP servers, `/workspace` JSONL event streams, and optional NATS fabric.

---

## Current baseline

Already merged in PR #4918:

- Discord inbound messages normalize once to `cto.presence.v1` at the bridge boundary.
- Presence routes support `runtime: "hermes" | "openclaw" | "hosted"`.
- Route matching includes Discord channel/thread/project/task/coderun specificity and fail-closed shared-channel behavior.
- Outbound intents (`send`, `edit`, `react`, `typing`, `status`) are executed only by the bridge-owned Discord client.
- `apps/hermes-presence-adapter` can register/delete routes, accept authenticated inbound events, post status outbound, and queue messages to Hermes API/input or JSONL inbox fallback.
- Controller injects Hermes presence adapter env/sidecar wiring when presence is enabled.
- In-cluster smoke verified bridge health, auth, route registration, inbound delivery to a disposable worker, and `401` on unauthenticated routes.

Morgan Meet docs establish the Hermes-native pattern:

- Runtime is a Kubernetes CodeRun pod with `lobster + ACPX`, shared `/workspace`, MCP tools, and sidecar containers.
- Morgan sidecar exposes MCP tools such as `meet_join`, `meet_leave`, `meet_get_status`, `meet_stream_audio` on localhost.
- Event/command/status streams live in `/workspace/meet-events.jsonl`, `/workspace/meet-commands.jsonl`, and `/workspace/meet-status.json`.
- Lobster `meet-init` step waits for sidecar readiness and writes status before ACPX starts.
- OpenCloud/OpenClaw equivalents should be thin adapters over the same bridge/session core.

---

## Reference implementation principle

Hermes is the product-quality reference implementation. Parity means the centralized control plane should support the same useful end-user behaviors Hermes provides today, not merely the smaller OpenCloud/OpenClaw gateway feature set.

OpenCloud/OpenClaw remains in scope because CTO has promised it and it is strategically important, but it is not the north star. The order of precedence is:

1. Preserve Discord credential isolation and runtime-neutral contracts.
2. Match Hermes behavior and quality for real Discord/Morgan workflows.
3. Provide OpenCloud/OpenClaw compatibility through adapters and contract tests.
4. Add hosted/generic runtime support after Hermes paths are solid.

---

## Intended use cases to validate

### 1. Discord command install/home flow

- `/sethome` works in Coder Control Plane DM, guild channel, and private channel contexts as applicable.
- User-installed and guild-installed command metadata includes Discord DM/user-install contexts.
- Safe slash-command sync detects context/integration type drift and recreates stale commands.
- Home/session binding becomes route metadata for future Discord events.

### 2. Live Discord ingress to Hermes CodeRun

- A real Discord DM/channel/thread message addressed to an agent normalizes to `cto.presence.v1`.
- Bridge selects the correct Hermes route without leaking Discord credentials.
- Hermes adapter receives the event, preserves Discord session metadata, and queues it to the active CodeRun.
- Attachments and empty-text messages are represented faithfully.

### 3. Hermes conversational/session parity

- Discord channel/thread/user identity maps to a stable Hermes session key.
- Message replies preserve enough context for Hermes to continue the same conversation.
- Multiple messages in the same surface reach the same CodeRun/session unless explicitly rerouted.
- Ambiguous shared-channel messages fail closed unless agent mention or direct route metadata selects a target.

### 4. Hermes sidecar/MCP parity for Morgan

- Morgan sidecar can be attached to Hermes CodeRuns.
- ACPX discovers Morgan MCP tools through harness-injected MCP server config.
- `meet-init` lobster step gates startup on sidecar readiness.
- Meeting events/status/commands flow through `/workspace/*.jsonl` files.
- Discord can trigger Morgan meeting/session actions through normalized events and Hermes MCP/tool calls.

### 5. Outbound Discord effects

- Hermes/OpenCloud/hosted workers can request `typing`, `status`, `send`, `edit`, and `react` effects only through `/presence/outbound`.
- Bridge applies effects to the correct DM/channel/thread with bridge-owned credentials.
- Status updates are visible enough for the user to know whether Hermes is started/running/blocked/done/failed.

### 6. Thread/channel/guild routing

- Parent channel and thread IDs are preserved separately.
- Parent-channel routes can opt into thread traffic.
- Thread-specific routes can narrow delivery to a single thread.
- Guild/shared-channel traffic does not fan out to unrelated agents.

### 7. Multi-runtime contract compatibility

- The same centralized bridge-normalized event can fan out to Hermes and OpenCloud/OpenClaw routes when both match.
- OpenCloud/OpenClaw route registration, inbound endpoint, and outbound intent calls pass the same contract tests as Hermes.
- OpenCloud/OpenClaw adapter can be initially thinner than Hermes but must not require Discord credentials inside runtime workers.

### 8. Morgan-specific safety and identity

- Morgan is clearly identified in meeting/Discord surfaces.
- Consent/entry message language is available for meeting flows.
- Recording/transcription/event logs follow customer policy and avoid exposing secrets.
- `morgan@5dlabs.ai` identity flow is documented and kept separate from Discord bot credentials.

---

## Gap map against current implementation

| Area | Current state | Gap to Hermes parity |
|---|---|---|
| Discord bridge | Normalizes messages and routes presence events | Need live `/sethome`, live message, slash/interaction, attachment, and thread E2E matrix |
| Hermes adapter | Can receive event and queue to Hermes API/input or JSONL inbox | Need real CodeRun validation and richer session continuation semantics |
| Hermes sidecar model | Presence sidecar injection exists for Hermes adapter | Need Morgan sidecar/MCP injection, `meet-init`, workspace JSONL command/event/status streams |
| MCP/tool surface | Mentioned in docs for Morgan | Need implemented MCP server/registration/discovery path for Morgan tools |
| Outbound intents | Contract and bridge side exist | Need live Discord side-effect validation from real Hermes worker |
| Session/home/crown behavior | `/sethome` fix landed; route metadata exists | Need inspect Hermes code/behavior for crown/home/session semantics and map to control-plane concepts |
| OpenCloud/OpenClaw | Runtime type and router contract exists | Need adapter implementation/contract tests; keep in scope but secondary |
| Hosted/generic | Runtime type exists | Need generic webhook/MCP adapter after Hermes/OpenCloud basics |
| Safety/observability | Auth and secret boundaries tested | Need route audit logs, redacted diagnostics, and operator validation docs |

---

## Implementation tasks

### Task 1: Inventory Hermes behavior as the source of truth

**Objective:** Pull/inspect the actual Hermes codebase or deployed runtime patch and produce a feature inventory for Discord commands, crown/home/session handling, event dispatch, message handling, attachments, and outbound effects.

**Files:**
- Create: `docs/2026-04/research/hermes-discord-feature-inventory.md`
- Read/inspect: Hermes source repo or deployed runtime image/patch sources
- Read/inspect: `apps/discord-bridge/src/*`, `apps/hermes-presence-adapter/src/*`, `templates/harness-agents/hermes.sh.hbs`

**Steps:**
1. Locate authoritative Hermes source/image used by `hermes-control-plane-builder`.
2. Search for Discord command registration, `sethome`, crown/session/home concepts, message handlers, attachment handling, and outbound Discord helpers.
3. Document each capability as: behavior, input surface, state dependency, output side effect, security boundary, and parity status in CTO.
4. Do not print or store secret values from configs/manifests.

**Verification:** The inventory names the concrete files/commit/image inspected and lists at least command, message, session/home/crown, attachment, and outbound behavior categories.

### Task 2: Define `cto.presence.v2` or extend `v1` for Hermes parity

**Objective:** Add only the fields/events needed for Hermes parity while preserving backward compatibility with `cto.presence.v1` workers.

**Files:**
- Modify: `apps/discord-bridge/src/presence-types.ts`
- Modify: `apps/hermes-presence-adapter/src/types.ts`
- Add/modify tests in both apps
- Document: `docs/2026-04/plans/hermes-parity-centralized-discord-control-plane.md`

**Candidate additions:**
- Event types for slash command, component/modal interaction, lifecycle, and tool/action request if Hermes needs them.
- Stable `session_key`, `home_id`/home route metadata, and optional `conversation_id`.
- Rich attachment metadata including content type, size, Discord attachment ID, and spoiler flag if available.
- Reply/reference metadata for threaded conversation context.
- Explicit `addressing` metadata: mentioned agents, selected agent, broadcast/fail-closed reason.

**Verification:** Existing `cto.presence.v1` tests continue to pass; new validation rejects malformed new fields; route matching remains fail-closed.

### Task 3: Real Hermes CodeRun E2E validation

**Objective:** Prove a live Discord event reaches an actual Hermes CodeRun through the centralized bridge.

**Files:**
- Add/modify: `scripts/presence-smoke-hermes-coderun.sh` or equivalent Python-based script if local `curl` remains unavailable
- Document results in: `docs/2026-04/validation/hermes-presence-coderun-e2e.md`

**Steps:**
1. Launch a small presence-enabled Hermes CodeRun using existing controller path.
2. Confirm pod includes `hermes-presence-adapter` sidecar and shared token env via secret reference only.
3. Confirm adapter registers a route with the bridge.
4. Send synthetic `/presence/discord-events` event and confirm CodeRun inbox/input receives it.
5. Trigger a live Discord message from the canonical DM/channel/thread and confirm same path.
6. Cleanup test CodeRun/route.

**Verification:** Evidence includes route ID, pod names, timestamps, redacted logs, and no secret values.

### Task 4: Implement Hermes session/home parity

**Objective:** Map Hermes home/crown/session concepts into centralized control-plane route/session metadata.

**Files:**
- Modify: `apps/discord-bridge/src/presence-router.ts`
- Modify: `apps/discord-bridge/src/discord-normalizer.ts`
- Modify: `apps/hermes-presence-adapter/src/hermes-client.ts`
- Tests: corresponding `*.test.ts`

**Steps:**
1. Use Task 1 inventory to define the exact semantics.
2. Add route/session key derivation rules.
3. Preserve `/sethome` binding as a high-priority route selector where applicable.
4. Ensure mentions/direct agent selection override ambient home only when explicit.
5. Add tests for DM home, guild home, thread route, mention override, and no-fanout ambient messages.

**Verification:** Deterministic tests cover every precedence rule and ambiguity mode.

### Task 5: Morgan Hermes sidecar/MCP implementation

**Objective:** Make Morgan work through Hermes as the primary runtime.

**Files:**
- Modify: `templates/harness-agents/hermes.sh.hbs`
- Modify: `crates/controller/src/tasks/code/resources.rs`
- Modify/Add: Morgan sidecar image/config references once image exists
- Add: tests for resource generation if Rust toolchain/CI available
- Cross-repo: `5dlabs/morgan-meet` sidecar/MCP implementation

**Steps:**
1. Add CodeRun fields or config gates for Morgan sidecar enablement.
2. Inject `MORGAN_MEET_MCP_URL`, `MORGAN_MEET_SESSION_ID`, and MCP server registration before ACPX starts.
3. Add `meet-init` lobster step to wait for sidecar health.
4. Ensure sidecar writes `/workspace/meet-events.jsonl`, watches `/workspace/meet-commands.jsonl`, and updates `/workspace/meet-status.json`.
5. Expose MCP tools: `meet_join`, `meet_leave`, `meet_get_status`, `meet_stream_audio`.

**Verification:** A Hermes CodeRun can list/call Morgan MCP tools and receive meeting status events without OpenCloud/OpenClaw gateway involvement.

### Task 6: Live outbound Discord intent validation from Hermes

**Objective:** Prove Hermes can respond/status/react in Discord through bridge-owned credentials.

**Files:**
- Add: `docs/2026-04/validation/hermes-discord-outbound-intents.md`
- Add/modify smoke script if helpful

**Steps:**
1. From a Hermes worker/adapter, call `/presence/outbound` with `typing` and `status`.
2. Call `send` to a controlled test DM/channel/thread.
3. Call `react`/`edit` against the test message if permissions allow.
4. Confirm all effects appear in Discord and logs redact credentials.

**Verification:** Live Discord observations plus bridge logs show success; no worker has Discord token env.

### Task 7: OpenCloud/OpenClaw compatibility adapter

**Objective:** Keep OpenCloud/OpenClaw in scope by making it pass the same contract suite, without letting it define the core feature ceiling.

**Files:**
- Add: `apps/openclaw-presence-adapter/` or equivalent existing OpenCloud adapter location
- Add: shared contract tests for route registration/inbound/outbound
- Modify docs: OpenCloud/OpenClaw support status

**Steps:**
1. Implement adapter route registration to `/presence/routes` with `runtime: "openclaw"`.
2. Implement authenticated inbound endpoint accepting `cto.presence.v1`/extended events.
3. Bridge OpenCloud/OpenClaw gateway methods/events to the same normalized contract.
4. Implement outbound effects by calling `/presence/outbound`; no Discord credentials in OpenCloud/OpenClaw runtime.
5. Run the same contract smoke tests used by Hermes.

**Verification:** Synthetic fanout can deliver one event to Hermes and OpenCloud/OpenClaw routes; both can issue outbound status/send intents through the bridge.

### Task 8: Hosted/generic worker adapter

**Objective:** Provide a simple integration path for future hosted agents after Hermes/OpenCloud basics work.

**Files:**
- Add docs/examples under `docs/2026-04/examples/hosted-presence-worker.md`
- Optional: `apps/hosted-presence-example/`

**Steps:**
1. Define minimal route registration and inbound webhook contract.
2. Provide sample worker receiving `cto.presence.v1` and posting `/presence/outbound`.
3. Add auth guidance and secret-boundary notes.

**Verification:** Sample worker passes route/inbound/outbound smoke test.

### Task 9: Operator validation matrix

**Objective:** Make finish-line validation concrete and repeatable.

**Files:**
- Create: `docs/2026-04/validation/centralized-discord-control-plane-matrix.md`

**Matrix dimensions:**
- Surface: DM, guild channel, private channel, thread.
- Input: text, empty text, attachment, mention, slash command, reply/thread reference.
- Runtime: Hermes, OpenCloud/OpenClaw, hosted.
- Output: typing, status, send, edit, react.
- Routing: home route, direct agent, mention override, ambiguous/fail-closed.
- Morgan: meeting join, status, provider fallback, consent/entry messaging.

**Verification:** Every row has owner, command/script, expected result, and pass/fail evidence location.

---

## Open questions

1. What is the authoritative Hermes source repo or image tag for full feature inventory beyond the runtime patch already inspected?
2. What exactly does the Hermes "crown" system mean operationally: leader election, permission/ownership, active conversation selection, or something else?
3. Which Discord surface should be the canonical live validation target: Coder Control Plane DM only, a guild channel, or a thread?
4. Which agent IDs are canonical for live route matching now: `coder`, `rex`, `metal`, `morgan`, others?
5. Should Morgan Hermes sidecar implementation live first in `5dlabs/morgan-meet`, `5dlabs/cto`, or a shared Aperture repo/package?
6. Should the next PR be inventory/docs-only, or should it directly implement Hermes session/home parity after the inventory is complete?

---

## Recommended next PR sequence

1. **Inventory PR:** Hermes feature inventory + validation matrix + contract gap map.
2. **Hermes E2E PR:** real CodeRun smoke script/docs + fixes found while validating.
3. **Session/home parity PR:** route/session/home/crown semantics mapped into bridge and adapter.
4. **Morgan Hermes PR:** sidecar/MCP/meet-init/workspace stream implementation.
5. **OpenCloud/OpenClaw adapter PR:** contract-compatible adapter and tests.
6. **Hosted example PR:** generic worker sample and docs.
