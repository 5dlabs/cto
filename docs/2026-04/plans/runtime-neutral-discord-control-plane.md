# Runtime-Neutral Discord Control Plane Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Complete the CTO Discord control plane so Discord credentials live only in the bridge boundary while normalized `cto.presence.v1` events route to Hermes, OpenClaw, and future hosted agents through contract-compatible adapters.

**Architecture:** `apps/discord-bridge` owns Discord login, message normalization, runtime route registry, inbound fanout, outbound Discord intents, and optional NATS fabric publication. Worker runtimes register routes and receive normalized events over `/presence/inbound`; workers respond only through `/presence/outbound`. `apps/hermes-presence-adapter` is the first runtime worker adapter and `crates/controller` injects it for Hermes CodeRuns when presence is enabled.

**Tech Stack:** TypeScript/Node 20 (`discord.js`, `node:test`, `tsx`), Rust controller manifests where available, Kubernetes GitOps manifests.

---

## Acceptance criteria

1. Discord inbound messages are normalized exactly once into `cto.presence.v1` event shape at the bridge boundary.
2. The bridge can fan out a single Discord event to matching Hermes, OpenClaw, and hosted routes without exposing Discord credentials to workers.
3. Route matching is deterministic, rejects ambiguous same-score direct routing, supports channel/thread/project/task/coderun specificity, and supports multi-route Discord fanout.
4. Outbound worker intents (`send`, `edit`, `react`, `typing`, `status`) are applied only by the bridge-owned Discord client.
5. Hermes adapter accepts authenticated bridge-to-worker events, queues to Hermes input or inbox, posts status intents back to the bridge, registers and deletes its route, and preserves Discord session metadata.
6. GitOps manifests configure the isolated control-plane builder and do not migrate the production bridge path by accident.
7. Node unit tests and TypeScript builds pass for `apps/discord-bridge` and `apps/hermes-presence-adapter`; Rust/controller checks are run when the toolchain is available, otherwise the missing toolchain is documented.

## Tasks

### Task 1: Repair Node test scripts

**Objective:** Make existing tests actually execute under npm instead of passing a quoted glob to Node.

**Files:**
- Modify: `apps/discord-bridge/package.json`
- Modify: `apps/hermes-presence-adapter/package.json`

**Steps:**
1. Change each `test` script from `node --test --import tsx "src/**/*.test.ts"` to `node --test --import tsx src/*.test.ts`.
2. Run both app test commands and confirm existing tests execute.

### Task 2: Add pure Discord event normalizer

**Objective:** Convert Discord message-like objects into normalized control-plane events without depending on live Discord APIs.

**Files:**
- Create: `apps/discord-bridge/src/discord-normalizer.ts`
- Create: `apps/discord-bridge/src/discord-normalizer.test.ts`
- Modify: `apps/discord-bridge/src/presence-types.ts`

**Behavior:**
- Ignore bot-authored messages.
- Preserve guild/channel/thread/parent/user/message IDs, text, attachments, and mentioned agent IDs.
- Infer `chat_type` as `dm`, `thread`, or `group`.
- Produce schema `cto.presence.v1`, event_type `message`, and no runtime-specific credentials.

### Task 3: Add route fanout for normalized Discord events

**Objective:** Route one normalized Discord event to every matching registered runtime route.

**Files:**
- Modify: `apps/discord-bridge/src/presence-types.ts`
- Modify: `apps/discord-bridge/src/presence-router.ts`
- Modify: `apps/discord-bridge/src/presence-router.test.ts`

**Behavior:**
- Add `routeDiscordEvent(payload)` to `PresenceRouter`.
- Match routes by Discord filters, mentioned agent IDs, and route specificity.
- Generate runtime-specific `PresenceInbound` per selected route.
- Avoid cross-talk: routes with no Discord/project/task/coderun specificity are not eligible.
- Return delivery results for all selected routes.

### Task 4: Wire Discord client ingress to the presence router

**Objective:** Subscribe to Discord message creation only when the presence router is available and feed normalized events into route fanout.

**Files:**
- Modify: `apps/discord-bridge/src/discord-client.ts`
- Modify: `apps/discord-bridge/src/index.ts`

**Behavior:**
- Add required Gateway intents for message ingress.
- Add `onMessage` handler on `DiscordHandle`.
- Log and continue on route errors; never leak token values.

### Task 5: Finish HTTP/control-plane API contract polish

**Objective:** Expose contract-compatible route management, inbound direct route, fanout ingress, outbound intents, and health details.

**Files:**
- Modify: `apps/discord-bridge/src/http-server.ts`
- Add/extend tests if needed.

**Behavior:**
- Keep `/notify` and elicitation endpoints backwards-compatible.
- Add `/presence/discord-events` for synthetic/control-plane ingress tests.
- Keep all `/presence/*` APIs protected by `PRESENCE_SHARED_TOKEN`.

### Task 6: Complete adapter compatibility and configuration

**Objective:** Ensure Hermes adapter and controller-generated sidecar routes match the bridge contract and support future runtimes.

**Files:**
- Modify: `apps/hermes-presence-adapter/src/*` as needed.
- Modify: `crates/controller/src/tasks/code/resources.rs` only if contract env is missing.

**Behavior:**
- Hermes adapter route registration includes runtime `hermes`, agent, coderun, worker URL, Discord filters, and session key.
- Adapter posts status through `/presence/outbound` only.

### Task 7: Verify and review

**Objective:** Prove the solution is tested and ready for PR.

**Commands:**
- `npm test` and `npm run build` in both Node apps.
- `cargo test -p controller presence` if Cargo is available.
- Static secret scan of diff.
- Independent code review.
