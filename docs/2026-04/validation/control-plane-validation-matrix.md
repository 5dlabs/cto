# CTO Control Plane Validation Matrix

> Finish-line validation matrix for the CTO control plane. Unknown or unverified rows are intentionally marked `NOT_STARTED` or `BLOCKED`, not done.

## Status vocabulary

- `PASS` — validated with evidence linked in this matrix.
- `UNIT_PASS` — validated by local unit/contract tests; live/cluster evidence is still required before final acceptance.
- `FAIL` — attempted and failed; evidence explains failure.
- `NOT_STARTED` — not yet implemented or not yet validated.
- `BLOCKED` — cannot be validated until an upstream decision, artifact, environment, or implementation exists.

## Evidence standards

Each completed row should include:

- Redacted command/script or manual procedure used.
- Expected result and observed result.
- Timestamp, route ID, pod/app identifiers, and target surface identifiers where safe.
- Evidence path under `docs/2026-04/validation/` or a linked PR/check/log artifact.
- Confirmation that Discord tokens, presence shared tokens, and other secrets were not printed.

## Summary by area

| Area | Current status | Notes |
|---|---|---|
| Discord surfaces | `UNIT_PASS` | Bridge unit coverage now validates normalization, rich attachment metadata preservation (`id`, filename, content type, size, spoiler), mention selection, shared-channel fail-closed behavior, thread parent/thread preservation, and thread-vs-parent home-route precedence. Live Discord DM/guild/private/thread/attachment evidence remains. |
| Hermes ingress/outbound/session | `UNIT_PASS` | Hermes adapter unit coverage now validates authenticated inbound delivery, metadata validation, deterministic session/home/route metadata forwarding, and outbound status intent posting. The Hermes CodeRun smoke harness now has local Python syntax coverage and dry-run manifest/payload rendering evidence; live CodeRun E2E and live session/home validation remain. |
| Morgan sidecar/MCP | `BLOCKED` | Design exists; sidecar/MCP implementation and repo/image decision are missing. |
| Agent coordination | `UNIT_PASS` | Wave 2A runtime-neutral envelope/addressing contract and pure TypeScript helper skeleton are documented and unit-tested; validators now reject unknown message/runtime/address enums before transport adapters trust payloads. Durable registry/inbox/service implementation remains. |
| Memory/skills lifecycle | `UNIT_PASS` | Morgan memory/skills lifecycle policy is now documented with scoped runtime/workspace/OpenMemory layers, retention guardrails, remote skill-source rules, lifecycle states, and pinned-core protections. Implementation and live policy enforcement remain. |
| OpenClaw/hosted | `NOT_STARTED` | Runtime type exists; adapters/examples and contract tests remain. |
| Ops hardening | `UNIT_PASS` | Workflow audit now confirms the existing Discord bridge publish workflow and a local Hermes adapter publish workflow candidate are syntax-valid; Hermes workflow still must land on `main` and publish successfully before final `PASS`. Operator runbook/rollback skeleton and branch reconciliation handoff now exist; builder RBAC now has a self-contained GitOps fix candidate to unblock live smoke prerequisites after merge/sync. Live scale/failure validation, RBAC-restored smoke access, and redacted diagnostics remain. |

## Discord surfaces and input coverage

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| DS-01 | DM text message normalizes to `cto.presence.v1` | `NOT_STARTED` | TBD | Send controlled DM to Discord bridge bot; inspect normalized event delivery. | Event includes DM chat type, user ID, message ID, text, no Discord credentials. | TBD |
| DS-02 | Guild channel text message normalizes and routes only when addressed | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `normalized mention messages route to non-default mentioned agents`, `fanout ignores routes for unmentioned agents when the Discord event has mentions`. | Matching route receives event; unrelated routes do not. | Live Discord guild message evidence still required. |
| DS-03 | Guild/shared-channel ambient message fails closed | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `unaddressed shared-channel Discord events do not fan out`. | No unrelated agent delivery; unit router returns zero deliveries. | Live Discord guild ambient evidence still required. |
| DS-04 | Private channel message normalizes and routes | `NOT_STARTED` | TBD | Send message in canonical private channel. | Event preserves channel/user metadata and routes to intended worker only. | TBD |
| DS-05 | Thread message preserves parent channel and thread IDs | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `normalizes thread messages with parent channel id`. | Event includes parent channel and actual thread ID separately. | Live Discord thread evidence still required. |
| DS-06 | Thread-specific route beats parent-channel route | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `thread-specific home route beats parent-channel home route`. | Thread-specific route selected according to precedence. | Live Discord thread route evidence still required. |
| DS-07 | Parent-channel route receives thread traffic only when configured | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `parent-channel home route handles thread traffic when only parent channel is registered`. | Parent route can receive thread traffic when it is the selected home route. | Needs a stricter live/documented opt-in policy decision before final `PASS`. |
| DS-08 | Empty-text message is represented | `UNIT_PASS` | control-plane loop | `2026-05-04T01:25Z npm test` in `apps/discord-bridge` (`28/28` pass): `preserves empty text for attachment-only Discord messages` and `accepts empty text on attachment-only normalized Discord events`. `npm run build` passed. | Event preserves explicit empty text plus non-text attachment metadata instead of dropping message. | Live Discord attachment/embed-only message evidence still required. |
| DS-09 | Attachment metadata is represented | `UNIT_PASS` | control-plane loop | `2026-05-03T23:15:44Z npm test` in `apps/discord-bridge` (`25/25` pass): `normalizes guild messages into cto.presence.v1 without Discord credentials`, `rejects malformed attachments on normalized Discord events`, and `accepts rich attachment metadata on worker inbound events`. `npm run build` passed. | Event preserves attachment URL plus ID, filename, content type, size, and spoiler where available; malformed attachment shape is rejected before worker delivery. | Live Discord attachment evidence still required. |
| DS-10 | Reply/reference metadata is represented | `UNIT_PASS` | control-plane loop | `2026-05-04T00:22Z npm test` in `apps/discord-bridge` (`26/26` pass): `normalizes Discord reply reference metadata`. `npm run build` passed. | Normalized event carries reply/source message, channel, and guild IDs without Discord credentials. | Live Discord reply evidence still required. |
| DS-11 | Mention/addressing metadata selects intended agent | `UNIT_PASS` | control-plane loop | `2026-05-04T01:39Z npm test` in `apps/discord-bridge` (`28/28` pass): `normalized mention messages route to non-default mentioned agents` now asserts selected route delivery carries `metadata.selected_agent_id=rex`, `metadata.selection_reason=discord_mention`, and normalized `metadata.mentioned_agent_ids` while the existing unmentioned-agent fanout regression stays green. `npm run build` passed. | Selected agent metadata is present on worker inbound payloads; unrelated routes do not receive mentioned events. | Live Discord mention/addressing evidence still required before `PASS`. |
| DS-12 | Slash command or interaction path is represented | `NOT_STARTED` | TBD | Invoke canonical slash/interaction flow. | Event/command shape is normalized or explicitly documented as unsupported. | TBD |
| DS-13 | `/sethome` works in DM context | `NOT_STARTED` | TBD | Invoke `/sethome` from bot DM. | Home binding succeeds or produces controlled error; command context accepted by Discord. | TBD |
| DS-14 | `/sethome` works in guild/private-channel context | `NOT_STARTED` | TBD | Invoke `/sethome` from configured guild/private channel. | Home binding succeeds or produces controlled error; command context accepted by Discord. | TBD |

## Hermes ingress, outbound, and session behavior

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| H-01 | Hermes CodeRun receives synthetic bridge event | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass) and `apps/hermes-presence-adapter` (`5/5` pass): synthetic `/presence/discord-events` fanout plus authenticated adapter inbound/inbox fallback. `2026-05-03T22:47:33Z python3 -m py_compile scripts/presence-smoke-hermes-coderun.py scripts/presence-morgan-task-smoke.py` passed, and `python3 scripts/presence-smoke-hermes-coderun.py --mode dry-run` rendered a Hermes `CodeRun` manifest plus synthetic `cto.presence.v1` payload without cluster mutation or secret output. | Adapter receives authenticated inbound and writes Hermes input or fallback inbox; dry-run harness can now be used as the repeatable live-smoke entry point. | Real CodeRun smoke still required via `scripts/presence-smoke-hermes-coderun.py --mode live`. |
| H-02 | Hermes CodeRun receives live Discord DM | `NOT_STARTED` | TBD | Send live DM to canonical validation bot/user. | Message reaches active Hermes CodeRun with session metadata. | TBD |
| H-03 | Hermes CodeRun receives live guild/channel event | `NOT_STARTED` | TBD | Send addressed guild/channel message. | Message reaches intended Hermes CodeRun only. | TBD |
| H-04 | Hermes CodeRun receives live thread event | `NOT_STARTED` | TBD | Send addressed thread message. | Message reaches intended Hermes CodeRun with thread metadata. | TBD |
| H-05 | Hermes adapter route registration | `NOT_STARTED` | TBD | Inspect bridge route registry after CodeRun starts. | Route includes runtime `hermes`, agent, coderun, project/task, worker URL, filters, session metadata. | TBD |
| H-06 | Hermes adapter route deletion/cleanup | `NOT_STARTED` | TBD | Stop CodeRun or adapter; inspect route cleanup/expiry. | Route is deleted or expires without stale delivery. | TBD |
| H-07 | Presence shared token is secret-ref only | `NOT_STARTED` | TBD | Inspect rendered pod/env and validation logs redacted. | Token value is not printed; pod uses secret reference. | TBD |
| H-08 | Hermes pod has no Discord token | `NOT_STARTED` | TBD | Inspect redacted pod env for Hermes worker and sidecars. | No Discord bot token or Discord credential appears outside bridge. | TBD |
| H-09 | Hermes outbound `typing` intent | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `applies outbound Discord intents through the bridge-owned client`. | Bridge applies typing indicator through bridge-owned client. | Live Discord typing evidence still required. |
| H-10 | Hermes outbound `status` intent | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/hermes-presence-adapter` (`5/5` pass): adapter posts non-fatal status intents; `apps/discord-bridge` outbound-intent test maps status to reaction/log behavior. | Status intent uses approved bridge path and does not require worker Discord credentials. | Live Discord status evidence still required. |
| H-11 | Hermes outbound `send` intent | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `applies outbound Discord intents through the bridge-owned client`. | Bridge sends message using bridge-owned client. | Live Discord send evidence still required. |
| H-12 | Hermes outbound `edit` intent | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `applies outbound Discord intents through the bridge-owned client`. | Bridge edits the correct message via bridge-owned client. | Live Discord edit evidence still required. |
| H-13 | Hermes outbound `react` intent | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `applies outbound Discord intents through the bridge-owned client`. | Bridge applies reaction via bridge-owned client. | Live Discord react evidence still required. |
| H-14 | DM home route precedence | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `channel-specific DM home route beats ambient home fallback`. | DM home route wins for that user/surface. | Live DM home-route evidence still required. |
| H-15 | Mention/direct selection overrides ambient home | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `explicit mention/direct agent selection overrides ambient home`. | Explicit selection wins. | Live Discord addressed-message evidence still required. |
| H-16 | Stable session key within a Discord surface | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass) and `apps/hermes-presence-adapter` (`5/5` pass): deterministic session key generated by bridge and forwarded into Hermes metadata/session. | Same session key/conversation metadata unless explicitly rerouted. | Real CodeRun/live repeat-message evidence still required. |
| H-17 | Ambiguous same-score direct routes fail closed | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): `rejects ambiguous equal-score route matches`. | No arbitrary delivery; diagnostic explains ambiguity. | Live/stale-route collision smoke still required. |
| H-18 | Attachments reach Hermes input/inbox | `UNIT_PASS` | control-plane loop | `2026-05-04T01:25Z npm test` in `apps/hermes-presence-adapter` (`8/8` pass): adapter rejects malformed attachment metadata before Hermes input and forwards attachment-only Discord messages with the explicit no-text placeholder plus attachment URL list. `npm run build` passed. Bridge package `npm test` (`28/28`) also validates empty-text and rich attachment contract shape. | Hermes adapter accepts only well-formed normalized attachments and forwards attachment context through the credential-free adapter path, including attachment-only messages. | Live CodeRun/Discord attachment evidence still required. |
| H-19 | Replies reach Hermes input/inbox | `UNIT_PASS` | control-plane loop | `2026-05-04T01:39Z npm test` in `apps/hermes-presence-adapter` (`8/8` pass): `forwards Discord reply reference metadata to Hermes input` now also proves bridge-selected mention metadata (`selected_agent_id`, `selection_reason`, `mentioned_agent_ids`) reaches Hermes input metadata. `npm run build` passed. | Hermes input metadata includes Discord reply IDs plus selected-agent/addressing provenance from normalized inbound events. | Live CodeRun/Discord reply and addressed-message evidence still required. |
| H-20 | Slash/interaction route to Hermes | `NOT_STARTED` | TBD | Invoke canonical slash/interaction for Hermes. | Hermes receives normalized interaction or row is explicitly scoped out. | TBD |

## Morgan sidecar, MCP, and meeting/avatar path

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| M-01 | Decide Morgan sidecar source location | `BLOCKED` | TBD | Architecture decision record/PR. | Source repo/package is selected: `morgan-meet`, `cto`, or shared package. | TBD |
| M-02 | Morgan sidecar image/package exists | `BLOCKED` | TBD | Build/package sidecar after M-01. | Durable image/package exists with health endpoint. | TBD |
| M-03 | Morgan sidecar attaches to Hermes CodeRun | `BLOCKED` | TBD | Render/launch CodeRun with Morgan enabled. | Pod includes Morgan sidecar, shared `/workspace`, env, health/readiness. | TBD |
| M-04 | ACPX discovers Morgan MCP tools | `BLOCKED` | TBD | Inspect generated ACPX config and call tool listing. | `meet_join`, `meet_leave`, `meet_get_status`, `meet_stream_audio` visible. | TBD |
| M-05 | `meet-init` gates readiness | `BLOCKED` | TBD | Launch Morgan-enabled Hermes CodeRun. | Lobster waits for sidecar readiness/status before ACPX workflow proceeds. | TBD |
| M-06 | `/workspace/meet-events.jsonl` is written | `BLOCKED` | TBD | Trigger meeting event/status. | JSONL event stream records redacted event. | TBD |
| M-07 | `/workspace/meet-commands.jsonl` is consumed | `BLOCKED` | TBD | Write controlled command or call MCP tool that writes command. | Sidecar observes/handles command. | TBD |
| M-08 | `/workspace/meet-status.json` reflects status | `BLOCKED` | TBD | Call `meet_get_status` and inspect file. | Status file updates with safe current meeting/session state. | TBD |
| M-09 | Morgan controlled meeting join | `BLOCKED` | TBD | Run controlled LiveKit/meeting smoke. | Morgan joins and announces/records status safely. | TBD |
| M-10 | Morgan controlled meeting leave | `BLOCKED` | TBD | Run leave flow after join. | Morgan leaves and updates status safely. | TBD |
| M-11 | Morgan consent/entry messaging | `BLOCKED` | TBD | Join meeting with consent policy enabled. | Entry/status language meets policy. | TBD |
| M-12 | Avatar/provider fallback path | `BLOCKED` | TBD | Simulate unavailable primary avatar/meeting provider. | Fallback behavior is safe and visible, or explicitly scoped out. | TBD |

## Agent coordination plane

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| C-01 | Coordination contract documented | `UNIT_PASS` | control-plane loop | `2026-05-03T22:00Z npm test` in `apps/agent-coordination-plane` (`6/6` pass) plus `npm run build`; `2026-05-03T22:24Z npm test` (`7/7` pass) plus `npm run build` after hardening enum validators; reviewed `docs/2026-04/design/agent-coordination-plane.md`. | Agent identity, routes, groups, envelopes, delivery targets, and human contact request are specified; malformed unknown message kinds, priorities, runtimes, and address kinds fail validation before transport adapters trust payloads. | Wave 2A spec and helper skeleton cover identity/address/message/envelope and subject mapping; human-contact policy endpoint still requires the service/MCP wave before final `PASS`. |
| C-02 | Agent registration API | `NOT_STARTED` | TBD | Register two disposable Hermes agents. | Directory stores agent identity, project/task/coderun/role metadata, expiry. | TBD |
| C-03 | Project group routing | `NOT_STARTED` | TBD | Broadcast to project group. | All project members receive durable message. | TBD |
| C-04 | Task group routing | `NOT_STARTED` | TBD | Broadcast to task group. | All task members receive durable message. | TBD |
| C-05 | Role group routing | `NOT_STARTED` | TBD | Send to role group. | Agents with matching role receive durable message. | TBD |
| C-06 | Per-agent durable inbox | `NOT_STARTED` | TBD | Send message, restart worker, read inbox. | Message survives until ack or configured expiry. | TBD |
| C-07 | Ack/retry/dead-letter basics | `NOT_STARTED` | TBD | Force delivery failure/retry. | Retry and dead-letter behavior is deterministic. | TBD |
| C-08 | Human contact request policy gate | `NOT_STARTED` | TBD | Agent requests human contact. | Request routes through approved adapter and policy, not direct Discord credentials. | TBD |
| C-09 | Hermes coordination MCP tools | `NOT_STARTED` | TBD | Call `agent_lookup`, `agent_send`, `agent_broadcast`, `agent_read_inbox`, `agent_update_status`, `contact_human`. | Tools work through coordination service and preserve runtime-neutral envelope. | TBD |
| C-10 | Discord bridge is not internal agent bus | `NOT_STARTED` | TBD | Architecture/code review. | Internal agent messages do not use Discord payloads or bridge as primary bus. | TBD |

## Memory and skills lifecycle

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| MS-01 | Morgan memory/skills policy documented | `UNIT_PASS` | control-plane loop | `2026-05-04T00:00:49Z` reviewed `docs/2026-04/design/morgan-memory-skills-policy.md`; policy defines runtime streams vs workspace session memory vs OpenMemory, startup/during-run/completion retrieval rules, remote skills/persona source-of-truth, lifecycle states (`active`, `stale`, `archived`, `pinned`), and pinned-core protection. `git diff --check` passed for this docs-only evidence update. | Memory scopes and skill lifecycle rules are explicit enough to unblock implementation design. | Tooling/enforcement for project-scoped retrieve, lifecycle transitions, and pinned protection still required before final `PASS`. |
| MS-02 | Project-scoped memory retrieve | `NOT_STARTED` | TBD | Store Project A memory; query from Project A. | Relevant memory is retrievable. | TBD |
| MS-03 | Cross-project memory bleed prevention | `NOT_STARTED` | TBD | Store Project A memory; query from Project B. | Memory is not returned unless explicit policy elevation allows. | TBD |
| MS-04 | Meeting transcript summarization before durable write | `NOT_STARTED` | TBD | Process controlled transcript. | Only summarized/provenanced facts are written durably. | TBD |
| MS-05 | User/org preference promotion | `NOT_STARTED` | TBD | Promote controlled preference with provenance. | Preference stored at correct scope with confidence/provenance. | TBD |
| MS-06 | Skill scopes implemented | `NOT_STARTED` | TBD | Create core/project/meeting/ephemeral skills. | Scope metadata controls visibility and mutation. | TBD |
| MS-07 | Skill `active` to `stale` transition | `NOT_STARTED` | TBD | Simulate unused skill beyond policy threshold. | Skill marks stale. | TBD |
| MS-08 | Skill `stale` to `archived` transition | `NOT_STARTED` | TBD | Simulate unused stale skill beyond archive threshold. | Skill archives safely. | TBD |
| MS-09 | Pinned core Morgan skills cannot be auto-rewritten | `NOT_STARTED` | TBD | Attempt automatic rewrite of pinned skill. | Rewrite is blocked and audited. | TBD |
| MS-10 | Weekly/idle Curator-style review | `NOT_STARTED` | TBD | Run curator review job/tool. | Review proposes safe state changes with audit trail. | TBD |

## OpenCloud/OpenClaw and hosted compatibility

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| OCH-01 | Shared presence contract tests exist | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/discord-bridge` (`23/23` pass): runtime-neutral fanout test covers Hermes/OpenClaw/hosted routes for one normalized event; bridge/auth/outbound tests cover core contract boundaries. | Contract covers route registration, inbound event, outbound intents, auth failure at unit level. | Dedicated reusable cross-runtime contract suite still required. |
| OCH-02 | Hermes passes shared contract tests | `UNIT_PASS` | control-plane loop | `2026-05-03T21:14:12Z npm test` in `apps/hermes-presence-adapter` (`5/5` pass): auth, inbound metadata validation, status intent posting, Hermes input/inbox fallback, session/home metadata forwarding. | Hermes adapter passes current local contract checks. | Dedicated shared suite and live CodeRun evidence still required. |
| OCH-03 | OpenCloud/OpenClaw adapter exists | `NOT_STARTED` | TBD | Build/run adapter. | Adapter exposes authenticated inbound endpoint and route registration. | TBD |
| OCH-04 | OpenCloud/OpenClaw route registration | `NOT_STARTED` | TBD | Register runtime `openclaw` route. | Bridge accepts route with expected metadata. | TBD |
| OCH-05 | OpenCloud/OpenClaw inbound delivery | `NOT_STARTED` | TBD | Send normalized event through bridge fanout. | Adapter receives event without Discord credentials. | TBD |
| OCH-06 | OpenCloud/OpenClaw outbound `status/send` | `NOT_STARTED` | TBD | Adapter calls `/presence/outbound`. | Bridge applies effect through bridge credentials. | TBD |
| OCH-07 | Hermes and OpenCloud/OpenClaw same-event fanout | `NOT_STARTED` | TBD | Register matching routes for both runtimes; send one event. | Both eligible adapters receive normalized event as allowed by routing rules. | TBD |
| OCH-08 | Hosted/generic worker example exists | `NOT_STARTED` | TBD | Build/run hosted example. | Example registers route and exposes inbound webhook. | TBD |
| OCH-09 | Hosted worker receives inbound event | `NOT_STARTED` | TBD | Send normalized event to hosted route. | Worker receives event with auth and metadata. | TBD |
| OCH-10 | Hosted worker sends outbound status/send | `NOT_STARTED` | TBD | Worker calls `/presence/outbound`. | Bridge applies status/send effect. | TBD |

## Operations, hardening, and production validation

| ID | Use case | Status | Owner | Validation command/procedure | Expected result | Evidence |
|---|---|---|---|---|---|---|
| OPS-01 | Discord bridge image publish workflow verified | `PASS` | control-plane loop | `2026-05-03T21:35Z` inspected `.github/workflows/discord-bridge-publish.yml`, `.github/actions/docker-build-push/action.yaml`, ran YAML parse check, and `gh run list --workflow discord-bridge-publish.yml --limit 5 --json ...`. | Workflow is on `main`; latest push run for `85ee0d503f8c5a88f525b5310b55c0550a16bdfa` completed `success` on 2026-04-30. Tags include `latest`, `v<package.version>`, and commit SHA through the shared Docker action. | GitHub Actions run `25163643005`; no secrets printed. |
| OPS-02 | Hermes presence adapter image publish workflow verified | `UNIT_PASS` | control-plane loop | `2026-05-03T21:35Z` inspected local `.github/workflows/hermes-presence-adapter-publish.yml`, `.github/actions/docker-build-push/action.yaml`, and ran YAML parse check. `gh run list --workflow hermes-presence-adapter-publish.yml` returned HTTP 404 because the workflow is not yet present on `origin/main`. | Workflow candidate mirrors the Discord bridge publish path: PR/push test+build, main-only GHCR publish, and `latest`/`v0.1.0`/SHA tags via the shared Docker action. | Needs commit/merge and first successful Actions run before `PASS`; no secrets printed. |
| OPS-03 | Branch reconciliation handoff exists before image/GitOps promotion | `PASS` | control-plane loop | `2026-05-04T03:40Z` confirmed PR #4925 from `control-plane-presence-hardening-2026-05-04` to `main` is open and `MERGEABLE`, then watched `gh pr checks 4925 --watch --interval 10` to completion. CI passed for CodeQL Analyze (rust), Discord Bridge Test & Build, Hermes Presence Adapter Test & Build, Controller CI changes/lint-rust/test-rust/integration-tests, code-quality scans, skills scan, and mirror; PR-only publish jobs skipped as expected. Refreshed path-overlap analysis still shows local 30 paths, remote 48 paths, overlap 0 from merge base `85ee0d503f8c`. | Evidence handoff identifies local commit groups, remote stack, remote safety branch, PR URL/check state, and safe post-merge validation sequence without blind force-push/rebase surgery from heartbeat. | PR #4925: https://github.com/5dlabs/cto/pull/4925; production image pin audit remains separate follow-up. |
| OPS-04 | ArgoCD apps healthy after rollout | `NOT_STARTED` | TBD | Inspect ArgoCD/app health. | Apps are synced and healthy. | TBD |
| OPS-17 | Builder RBAC prerequisite is self-contained for live smoke handoff | `UNIT_PASS` | control-plane loop | `2026-05-04T04:57Z` inspected live prerequisite failures and found the builder `ClusterRoleBinding` referenced missing `ClusterRole/cto-hermes-gateway`. Updated `infra/manifests/hermes-control-plane-builder/rbac.yaml` to define a matching self-contained `ClusterRole/cto-hermes-coder-control` and bind the builder service account to it; validated with `kubectl kustomize infra/manifests/hermes-control-plane-builder`, `git diff --check`, smoke script `py_compile`, and Hermes CodeRun dry-run. | After PR merge/GitOps sync, the builder heartbeat should regain the Kubernetes read/create surface needed to retrieve redacted smoke prerequisites and create temporary CodeRun/pod smoke resources without relying on a missing external role. | Local GitOps manifest evidence only; live RBAC cannot change until the branch is merged and ArgoCD syncs, so final `PASS` requires `kubectl auth can-i` checks returning yes and a live smoke run without secret output. |
| OPS-05 | Redacted route/register/delete/delivery logs | `NOT_STARTED` | TBD | Exercise route lifecycle and inspect logs. | Logs include useful route/runtime metadata and no secrets. | TBD |
| OPS-06 | Redacted outbound intent audit logs | `NOT_STARTED` | TBD | Exercise outbound intents and inspect logs. | Logs include effect type/status and no secret/token/message leakage beyond policy. | TBD |
| OPS-07 | Operator runbook exists | `UNIT_PASS` | `2026-05-03T23:37:57Z` reviewed `docs/2026-04/validation/control-plane-operator-runbook.md`; `git diff --check`, `python3 -m py_compile scripts/presence-smoke-hermes-coderun.py scripts/presence-morgan-task-smoke.py`, and dry-run Hermes CodeRun smoke passed. | Fresh operator has local/package validation commands, safe dry-run and live-smoke procedures, no-secret rules, and troubleshooting paths. | Runbook exists locally; needs operator dry-run/live rehearsal before final `PASS`. |
| OPS-08 | Rollback instructions exist | `UNIT_PASS` | `2026-05-03T23:37:57Z` reviewed rollback section in `docs/2026-04/validation/control-plane-operator-runbook.md`; no cluster mutation performed. | Rollback path prefers GitOps revert/previous image pin, documents emergency drift recording, and identifies post-rollback checks. | Needs a real rollback rehearsal or incident validation before final `PASS`. |
| OPS-09 | 10-Hermes-pod registration scale smoke | `NOT_STARTED` | TBD | Run scale script with 10 disposable registrations. | Registry/fanout remains correct. | TBD |
| OPS-10 | 50/100-Hermes-pod registration scale smoke | `NOT_STARTED` | TBD | Run scale script with 50/100 disposable registrations. | System stays within latency/error thresholds. | TBD |
| OPS-11 | Route collision/stale route expiry failure test | `NOT_STARTED` | TBD | Force duplicate/colliding/stale routes. | Collision and expiry behavior is deterministic and safe. | TBD |
| OPS-12 | Worker unavailable retry/dead-letter test | `NOT_STARTED` | TBD | Stop worker and deliver event/message. | Retry/dead-letter behavior is documented and observable. | TBD |
| OPS-13 | NATS unavailable/degraded-mode test | `BLOCKED` | TBD | Disable NATS/dependency after coordination plane exists. | Degraded behavior follows documented policy. | TBD |
| OPS-14 | Discord rate-limit/backoff behavior | `NOT_STARTED` | TBD | Exercise controlled outbound burst or mock rate limit. | Backoff/retry respects Discord limits and reports safe status. | TBD |
| OPS-15 | Morgan sidecar crash/restart behavior | `BLOCKED` | TBD | Kill/restart sidecar after Morgan exists. | CodeRun sees safe status and can recover or fail visibly. | TBD |
| OPS-16 | Final acceptance review | `NOT_STARTED` | TBD | Review this matrix and linked evidence. | All current-scope rows are `PASS` or explicitly moved out of scope. | TBD |
