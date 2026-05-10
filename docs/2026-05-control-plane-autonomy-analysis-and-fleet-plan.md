# CTO Control Plane Autonomy Analysis and Fleet Plan

Date: 2026-05-09

## Why progress was slow

Based on the preserved cron transcripts, current docs, validation matrix, and code state, the control-plane loop slowed down for several overlapping reasons:

1. **The cron heartbeat optimized for reporting, not shipping.** The heartbeat prompts repeatedly loaded context, counted matrix rows, ran smoke/status checks, and summarized blockers. That kept the project visible, but it was single-threaded and did not reliably create implementation branches/PRs or assign independent workstreams.
2. **The validation ladder was intentionally strict.** The matrix only moves rows to `PASS` with live/cluster/operator evidence. Many implemented pieces stayed at `UNIT_PASS` because live Discord ingress/outbound, sidecar attachment, GHCR publish, and multi-runtime evidence were not yet collected.
3. **The work got stuck in low-leverage unit/doc slices.** The docs themselves warned not to keep adding small local unit tests/docs if the feature-complete percentage would not move. Several cron runs appear to have repeated H-01 synthetic smoke or refreshed documentation instead of unblocking larger rows.
4. **Branch reconciliation and live environment friction burned cycles.** Earlier sessions had local `main` diverged from `origin/main`, image publish permission issues, RBAC prerequisites, and live-smoke prerequisites. Those were real blockers and made cron jobs conservative.
5. **Cron jobs are the wrong execution model for parallel implementation.** Cron runs start cold, have no user present, and should not recursively schedule jobs. They are good for watchdog/status/antenna checks, but bad for multi-agent implementation because they tend to re-discover context each tick and cannot coordinate a fleet well.
6. **A large portion of the matrix remained genuinely unimplemented.** Current matrix count is 93 rows: 5 `PASS`, 32 `UNIT_PASS`, 47 `NOT_STARTED`, 9 `BLOCKED`. The biggest remaining gaps are not status-reporting gaps; they are service/runtime gaps: agent coordination service, Morgan CodeRun sidecar wiring, OpenClaw/hosted adapters, live Discord evidence, and ops hardening.

## Current code baseline

Existing packages:

- `apps/discord-bridge`: normalized Discord boundary, route matching, outbound intent unit coverage.
- `apps/hermes-presence-adapter`: Hermes sidecar adapter with route registration/inbound/outbound local coverage.
- `apps/agent-coordination-plane`: currently only TypeScript envelope/addressing helpers and tests; no service, durable/in-memory store, registration API, inbox, ack/retry, or MCP tools.
- `apps/morgan-agent-sidecar`: local provider-free sidecar skeleton with Morgan/Meet tool aliases and workspace event/command/status streams; not yet attached to Hermes CodeRuns.

## Highest-impact execution order

The fastest way to restore autonomous progress is to avoid cron for implementation and run parallel fleet slices against independent files/packages:

1. **Coordination plane MVP service**
   - Add an in-memory implementation behind a store interface.
   - Add registration, lookup, send/broadcast, inbox read, ack, health APIs.
   - This directly starts moving C-02 through C-07 from `NOT_STARTED` toward `UNIT_PASS`.

2. **Shared presence contract + hosted worker example**
   - Extract reusable route/inbound/outbound contract helpers or add a generic hosted worker example.
   - This starts moving OCH-01/OCH-08/OCH-09/OCH-10 beyond placeholders.

3. **Morgan CodeRun sidecar wiring design/first render slice**
   - Wire controller/resource-generation config for optional Morgan sidecar injection and MCP/workspace env.
   - If full CRD wiring is too large, land a render helper + tests/docs first.
   - This unblocks M-03/M-04/M-05.

4. **Live validation harness improvements**
   - Improve scripts to collect redacted evidence for route lifecycle, no Discord token in worker pods, adapter route registration/deletion, and outbound effects.
   - This can move H-05/H-06/H-07/H-08/OPS-05/OPS-06.

## Execution model

- Keep cron disabled for implementation; use it later only as a quiet watchdog if needed.
- Use subagent fleet workers for independent slices, each constrained to distinct paths.
- Integrate in small PRs, but because edge_kase authorized merge authority, green/mergeable PRs can be merged by this control-plane builder after verification.
- Update the validation matrix only when a row materially changes status with evidence.

## First fleet wave

Wave A will run now:

- Worker A: implement coordination-plane MVP service + unit tests in `apps/agent-coordination-plane`.
- Worker B: implement hosted/generic worker contract/example + tests, avoiding Discord credentials.
- Worker C: inspect controller/Morgan sidecar wiring and implement the smallest safe render/test slice or produce a patch-ready plan if the CRD schema makes it unsafe in one wave.
- Parent integrator: validate package tests/builds, resolve conflicts, update matrix/plan evidence, commit and push.
