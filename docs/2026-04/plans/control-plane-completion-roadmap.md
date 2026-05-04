# CTO Control Plane Completion Roadmap

> Source-of-truth roadmap promoted from `.hermes/plans/2026-05-02_190837-control-plane-completion.md` into repo docs. This is a docs-only plan: it records the target state and validation expectations but does not mark unverified work as complete.

## Goal

Bring the CTO control plane from the current initial Discord/Hermes presence slice to a production-validated, runtime-neutral control plane covering:

- Hermes-first Discord workflows.
- Morgan Hermes meetings/avatar via sidecar + MCP.
- Durable many-Hermes-pod coordination.
- OpenCloud/OpenClaw compatibility.
- Hosted/generic worker compatibility.
- Memory and skills lifecycle management.
- Operator validation, observability, rollback, and hardening.

## Architecture direction

Keep the platform split into clear planes:

1. **Discord bridge** — the credentialed human-IO boundary. It owns Discord login, message/interaction normalization, route registry, inbound fanout, and outbound Discord effects.
2. **Hermes presence adapter** — the Hermes CodeRun ingress/outbound sidecar. It registers routes, receives authenticated normalized events, writes to Hermes input or `/workspace` inbox, and requests outbound effects through the bridge.
3. **Agent coordination plane** — durable project/task/role/group messaging for many pods. It must not use Discord-shaped payloads or direct pod networking as the product contract.
4. **Morgan meeting/avatar sidecars** — sidecars exposed through MCP and `/workspace` event/command/status streams.
5. **Memory/skills lifecycle plane** — scoped memory plus Curator-style skill hygiene for Morgan and related Hermes agents.
6. **Runtime adapters** — Hermes, OpenCloud/OpenClaw, and hosted/generic workers all pass the same normalized route/inbound/outbound contract tests.

Discord credentials and Discord API effects must remain inside `apps/discord-bridge` only.

## Current state summary

### Implemented or committed foundation

- `apps/discord-bridge` exists with Dockerfile and GitHub publish workflow.
- `apps/hermes-presence-adapter` exists with Dockerfile.
- `crates/controller/src/tasks/code/resources.rs` can inject a `hermes-presence-adapter` sidecar when presence is enabled for Hermes CodeRuns.
- Discord bridge normalizes inbound Discord messages to `cto.presence.v1`.
- Presence route runtime type supports `hermes`, `openclaw`, and `hosted`.
- Route matching supports Discord filters plus project/task/coderun specificity and fail-closed shared-channel behavior.
- Outbound intents exist for `send`, `edit`, `react`, `typing`, and `status`.
- Hermes adapter can register/delete routes, receive authenticated inbound events, post status outbound, and queue input to Hermes input/API or a JSONL inbox.
- `/sethome` install-context fix exists for the Hermes control-plane-builder runtime patch.
- Existing repo research identifies Hermes as a CTO CodeRun harness mode using Lobster + ACPX, not a separate external Hermes framework.

### Local artifacts to reconcile separately

At the time this roadmap was promoted, local status still showed untracked planning/research artifacts and a smoke script:

```text
?? .hermes/
?? docs/2026-04/plans/hermes-parity-centralized-discord-control-plane.md
?? docs/2026-04/research/
?? scripts/presence-morgan-task-smoke.py
```

Those artifacts should be reviewed in a follow-up PR or intentionally discarded. This roadmap does not commit or validate them by itself.

### Known gaps

- No confirmed successful publish workflow for `ghcr.io/5dlabs/hermes-presence-adapter`; main-branch test/build passes, but image publish is blocked by GHCR `write_package` permission.
- Live Hermes CodeRun synthetic route-registration/inbound/pod-discovery smoke is passing, but live Discord ingress/outbound and semantic worker-response evidence remain.
- Session/home/crown semantics are incomplete; `/sethome` install-context behavior is fixed, but control-plane route/session rules still need live Discord validation.
- Morgan Hermes sidecar/MCP source location is decided for the first stub: implement it in `5dlabs/cto` as `apps/morgan-agent-sidecar`, with `5dlabs/morgan-meet` remaining the OpenClaw-first demo/product plan. The sidecar image/package and live CodeRun attachment are not implemented yet.
- Durable many-Hermes-pod coordination plane is not implemented.
- OpenCloud/OpenClaw adapter remains to implement/validate.
- Hosted/generic adapter remains to implement/validate.
- Memory/skills lifecycle policy and tooling for Morgan are not implemented.
- Full operator validation matrix is not passing.

## Definition of 100% completion

The current product scope is complete only when every item below is documented with passing evidence in `docs/2026-04/validation/control-plane-validation-matrix.md` or linked validation docs.

1. **Discord credential boundary is clean**
   - Only `apps/discord-bridge` has Discord credentials.
   - Hermes/OpenCloud/OpenClaw/hosted workers request Discord effects only through normalized control-plane APIs.

2. **Hermes Discord parity is live**
   - A real Hermes CodeRun receives live DM/guild/thread/private-channel events through the bridge.
   - Session/home routing is deterministic.
   - Attachments, replies, empty text, mentions, and slash/interaction paths are represented.
   - Hermes can send/status/type/react/edit through bridge-owned credentials.

3. **Morgan Hermes agent path is live**
   - Morgan sidecar attaches to Hermes CodeRuns.
   - ACPX sees Morgan MCP tools.
   - `meet-init` gates readiness.
   - Meeting event/command/status streams exist in `/workspace`.
   - Morgan can join/leave/status a controlled meeting and publish user-visible status safely.

4. **Many-Hermes-pod coordination is live**
   - Agent directory, groups, durable inboxes, and project/task/role routing work.
   - Agent-to-agent messaging is runtime-neutral and not Discord-shaped.
   - Human contact is policy-gated and routed through approved adapters.

5. **Memory and skills lifecycle is live for Morgan**
   - Memory scopes prevent cross-project bleed.
   - Skill scopes distinguish core/project/meeting/ephemeral skills.
   - Curator-style states exist: `active`, `stale`, `archived`, `pinned`.
   - Pinned core Morgan skills cannot be auto-rewritten.

6. **Multi-runtime compatibility is proven**
   - Hermes and OpenCloud/OpenClaw pass the same route/inbound/outbound contract tests.
   - A hosted/generic worker example passes the same smoke.

7. **Operator confidence exists**
   - GitOps manifests point at durable branches/images.
   - Images publish reliably.
   - ArgoCD apps are healthy.
   - Validation docs show pass/fail evidence for all current-scope rows.
   - Rollback and redacted troubleshooting docs exist.

## Phases and estimates

Assumption: one primary engineer plus Copilot/Hermes subagents/reviewers. Estimates include planning, implementation, review, tests, and GitOps validation. Parallelization can shorten calendar time, but several workstreams depend on Hermes E2E and contract validation.

| Phase | Name | Estimate | Completion outcome |
|---:|---|---:|---|
| 0 | Reconcile repo, publish plan, image pipeline audit | 1-2 days | Clean committed baseline and source-of-truth plan docs |
| 1 | Hermes Discord MVP green | 1-2 weeks | Real Hermes CodeRun ingress/outbound/session validation |
| 2 | Validation matrix + observability | 3-5 days | Repeatable operator evidence harness and redacted diagnostics |
| 3 | Agent coordination plane MVP | 1-2 weeks | Many Hermes pods organized by project/task/group |
| 4 | Morgan Hermes sidecar/MCP MVP | 2-3 weeks | Morgan can run as Hermes sidecar with MCP + workspace streams |
| 5 | Morgan memory/skills lifecycle | 1-2 weeks | Scoped memory + Curator-style skill hygiene |
| 6 | OpenCloud/OpenClaw + hosted adapters | 1-2 weeks | Multi-runtime contract coverage |
| 7 | Production hardening/cutover | 1-2 weeks | GitOps, dashboards, docs, rollback, scale tests |

**Total estimate:** 8-13 calendar weeks for the known current-scope platform. A narrower Hermes Discord MVP is expected to take about 1-2 weeks.

## Phase details

### Phase 0: Reconcile baseline and source of truth

- Protect and review existing untracked plan/research docs.
- Promote this roadmap and the validation matrix into repo docs.
- Audit image/package publishing, especially whether `hermes-presence-adapter` has durable GHCR publishing.
- Verify the branch is based on current `origin/main` before implementation PRs.

**Exit criteria:** roadmap and matrix are committed in docs; repo baseline is explicit; missing image workflow status is known.

### Phase 1: Hermes Discord MVP green

- Add a real Hermes CodeRun smoke harness.
- Validate sidecar injection, route registration, authenticated inbound, and CodeRun inbox/input delivery.
- Implement deterministic session/home route rules:
  - DM home route beats ambient fallback for that user/surface.
  - Explicit mention/direct agent selection beats ambient home.
  - Thread route beats parent-channel route.
  - Parent-channel route handles thread messages only when designed.
  - Shared-channel messages with no recognized selector fail closed.
  - Session key remains stable within a Discord surface unless explicitly rerouted.
- Validate outbound effects from Hermes: `typing`, `status`, `send`, `edit`, `react`.
- Extend/validate attachments, replies, empty text, mentions, and slash/interaction representation.

**Exit criteria:** a real Hermes CodeRun receives live Discord input, maintains deterministic session metadata, and produces outbound Discord effects through bridge-owned credentials.

### Phase 2: Validation matrix and observability

- Maintain `docs/2026-04/validation/control-plane-validation-matrix.md` as the finish-line artifact.
- Add structured redacted diagnostics for route register/delete/deliver/outbound effect.
- Add operator runbook covering smoke tests, rollout, rollback, and troubleshooting.

**Exit criteria:** every current-scope row has status, owner, command/script, expected result, and evidence path; logs are useful without leaking secrets.

### Phase 3: Agent coordination plane MVP

- Specify runtime-neutral agent identity, groups, inboxes, routes, and message envelope.
- Implement service endpoints for registration, group management, message send, inbox read, ack, and health.
- Use NATS JetStream/KV for durable coordination, or a clearly marked local fallback behind an interface only for MVP.
- Add Hermes coordination sidecar/MCP tools: `agent_lookup`, `agent_send`, `agent_broadcast`, `agent_read_inbox`, `agent_update_status`, `contact_human`.

**Exit criteria:** multiple Hermes pods can register, discover peers/groups, exchange durable messages, and contact humans only through policy-gated adapters.

### Phase 4: Morgan Hermes sidecar/MCP MVP

- Reconcile Morgan Meet contracts against the accepted source decision: first Hermes stub sidecar in `5dlabs/cto` as `apps/morgan-agent-sidecar`; `5dlabs/morgan-meet` remains the OpenClaw-first demo/product plan and compatibility reference.
- Implement a Morgan sidecar exposing MCP tools and workspace streams:
  - Tools: `meet_join`, `meet_leave`, `meet_get_status`, `meet_stream_audio`.
  - Streams/files: `/workspace/meet-events.jsonl`, `/workspace/meet-commands.jsonl`, `/workspace/meet-status.json`.
- Wire sidecar injection into Hermes CodeRuns.
- Generate ACPX MCP config before ACPX starts.
- Add `meet-init` Lobster readiness gating.
- Run a controlled meeting/avatar smoke.

**Exit criteria:** Morgan can run as a Hermes sidecar with MCP tools and workspace streams; controlled join/leave/status flow passes.

### Phase 5: Morgan memory and skills lifecycle

- Define memory scopes: user, organization, project, meeting, task, ephemeral session.
- Implement scoped memory add/search/update for Morgan.
- Implement Curator-style skill lifecycle states: `active`, `stale`, `archived`, `pinned`.
- Protect pinned core Morgan skills from auto-rewrite.

**Exit criteria:** Morgan cannot retrieve cross-project memory unless policy allows it; skill lifecycle transitions and pinned protection are tested.

### Phase 6: OpenCloud/OpenClaw and hosted adapters

- Extract shared presence contract tests.
- Implement OpenCloud/OpenClaw adapter route registration, inbound handling, outbound intents, and gateway mapping.
- Add hosted/generic worker example.

**Exit criteria:** Hermes, OpenCloud/OpenClaw, and hosted/generic worker all pass shared route/inbound/outbound contract smokes.

### Phase 7: Production hardening and cutover

- Pin images and make GitOps rollback-safe.
- Add manifests/values for bridge, Hermes adapter, coordination plane, and Morgan sidecar as appropriate.
- Validate scale and failure modes:
  - 10/50/100 Hermes pod registrations.
  - Route collision and stale route expiry.
  - Worker unavailable/retry/dead-letter.
  - NATS unavailable/degraded mode.
  - Discord rate limit/backoff.
  - Morgan sidecar crash/restart.
- Complete final 100% acceptance review.

**Exit criteria:** ArgoCD/GitOps paths are healthy, rollback instructions exist, and all current-scope validation rows are `PASS` or explicitly moved out of current scope.

## Immediate next PR sequence

1. `docs: add control-plane completion roadmap and validation matrix`
2. `ci: publish hermes presence adapter image`
3. `test: add hermes coderun presence e2e smoke`
4. `feat: implement hermes session home routing parity`
5. `test: validate hermes outbound discord intents`
6. `feat: add agent coordination plane contract and service`
7. `feat: add hermes coordination mcp tools`
8. `feat: add morgan hermes sidecar mcp skeleton`
9. `feat: wire morgan sidecar into hermes coderuns`
10. `feat: add morgan memory and skill lifecycle policy/tools`
11. `feat: add openclaw presence adapter contract support`
12. `feat: add hosted presence worker example`
13. `ops: pin images add runbooks and scale validation`

## Risks and tradeoffs

- **Scope creep:** keep the validation matrix current-scope focused and move future work out explicitly.
- **Discord bridge overload:** do not turn the bridge into the internal agent bus; add the coordination plane beside it.
- **Image drift:** avoid unpinned `latest` for production where possible.
- **Secret leakage:** never print Discord or presence shared tokens; validation docs must redact.
- **Morgan dependency uncertainty:** sidecar/API location must be decided before implementation estimates can be tightened.
- **NATS durability:** local fallback may be acceptable for MVP, but production needs durable coordination.
- **Live validation friction:** some rows require controlled Discord/meeting surfaces and may need manual observation.

## Open questions

1. Is current-scope 100% allowed to exclude advanced avatar vendor bakeoffs beyond LemonSlice/LiveKit MVP?
2. Which provider secrets and meeting identity should the later Morgan sidecar use after the stub path is green?
3. What are canonical live validation Discord surfaces for DM/guild/thread/private channel?
4. Which agent IDs are canonical for live route matching: `morgan`, `coder`, `rex`, `metal`, others?
5. Should the first coordination plane use NATS JetStream immediately or a simpler store behind an interface?
6. Which auxiliary model should run Curator-style Morgan skill reviews?
7. What is the operational meaning of Hermes “crown,” if any, beyond home/session selection?
