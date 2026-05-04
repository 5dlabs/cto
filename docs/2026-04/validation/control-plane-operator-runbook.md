# CTO Discord Control Plane Operator Runbook

Date: 2026-05-03

## Purpose

Give an operator a repeatable, no-secret procedure to validate and troubleshoot the runtime-neutral Discord control plane without giving Hermes, OpenCloud/OpenClaw, or hosted workers Discord credentials.

The control-plane boundary is:

- `apps/discord-bridge` owns Discord credentials, inbound normalization, route registry/fanout, and outbound Discord API effects.
- Runtime adapters such as `apps/hermes-presence-adapter` receive authenticated `cto.presence.v1` events and request outbound effects through the bridge.
- Workers must not print or store Discord bot tokens or the presence shared token.

## Safety rules

1. Prefer repo/GitOps inspection before live mutation.
2. Do not paste Kubernetes Secret manifests, bearer values, Discord tokens, or provider API keys into issues, commits, or chat.
3. Live smoke tests may create temporary routes, pods, or CodeRuns; use unique `SMOKE_RUN_ID`/`route_id` values and clean them up.
4. If production is degraded, capture redacted evidence first, then restore by GitOps revert/image rollback rather than ad-hoc manifest drift whenever possible.
5. Do not mutate the legacy Coder Discord path while validating the centralized control plane.

## Quick local validation

Run these from `/opt/data/workspace/cto` after touching control-plane code or scripts:

```bash
git status --short --branch
git diff --stat
git diff --check
python3 -m py_compile scripts/presence-smoke-hermes-coderun.py scripts/presence-morgan-task-smoke.py
python3 scripts/presence-smoke-hermes-coderun.py --mode dry-run
```

When bridge code changes:

```bash
cd apps/discord-bridge
npm test
npm run build
```

When Hermes adapter code changes:

```bash
cd apps/hermes-presence-adapter
npm test
npm run build
```

When agent coordination code changes:

```bash
cd apps/agent-coordination-plane
npm test
npm run build
```

The repo root does not need to define `npm test`; package-scoped commands are the product checks.

## GitOps and image publication checks

```bash
git status --short --branch
git log --oneline -8
gh run list --workflow discord-bridge-publish.yml --limit 5 --json databaseId,status,conclusion,headBranch,headSha,createdAt,event
gh run list --workflow hermes-presence-adapter-publish.yml --limit 5 --json databaseId,status,conclusion,headBranch,headSha,createdAt,event
```

Interpretation:

- A workflow is final `PASS` only after it exists on the remote/default branch and has a successful relevant run.
- A local workflow candidate with valid YAML but no remote Actions history remains `UNIT_PASS`.
- Record run IDs and commit SHAs in `docs/2026-04/validation/control-plane-validation-matrix.md`; never record secret values.

## Dry-run Hermes CodeRun smoke

This is safe for cron/heartbeat runs because it does not mutate the cluster and does not require a live token:

```bash
python3 scripts/presence-smoke-hermes-coderun.py --mode dry-run
```

Expected evidence:

- a rendered `agents.platform/v1` `CodeRun` manifest with `spec.harnessAgent: hermes`,
- a synthetic `cto.presence.v1` inbound payload,
- no bearer token or Kubernetes Secret value printed.

## Live Hermes CodeRun smoke

Only run live mode when a human or incident procedure authorizes temporary cluster resources.

Prerequisites:

- kube context can access the target namespace and `CodeRun` CRD,
- controller image has Hermes presence sidecar injection enabled,
- bridge router is reachable,
- the presence shared token is available as an environment variable but will not be printed.

If running from outside the cluster, port-forward the bridge first:

```bash
kubectl -n bots port-forward svc/discord-bridge-http 3200:3200
export PRESENCE_ROUTER_URL='http://127.0.0.1:3200'
```

Then run the smoke with a unique run id:

```bash
export PRESENCE_SHARED_TOKEN='<redacted value from approved secret source>'
export SMOKE_RUN_ID="hermes-coderun-$(date -u +%Y%m%d%H%M%S)"
SMOKE_MODE=live python3 scripts/presence-smoke-hermes-coderun.py
```

Passing output should include:

```text
[smoke] mode=live
[smoke] route registered: hermes-coderun-...
[smoke] posting synthetic Discord event through /presence/inbound
[smoke] passed
```

If the script exits early, confirm whether cleanup ran. If needed, delete only the disposable smoke route/CodeRun named by `SMOKE_RUN_ID`.

## Morgan/task routing smoke

Use `scripts/presence-morgan-task-smoke.py` for Morgan DM/home and Sigma One task-channel route semantics before the full Morgan sidecar exists. Prefer dry/synthetic behavior unless explicitly validating a live deployment.

The expected route behavior is:

- Morgan DM/home route returns `202`,
- Sigma One task-channel route to Rex returns `202`,
- ambient or wrong task-channel traffic fails closed with `404`,
- no shared token is printed.

## Troubleshooting matrix

| Symptom | Likely cause | Next safe check |
| --- | --- | --- |
| `/presence/routes` returns `401` without auth | Expected auth boundary | Retry with bearer env only; do not paste token. |
| `/presence/inbound` returns `404` | No matching route, stale route, wrong channel/thread/task/coderun selectors | Inspect redacted route list and synthetic payload selectors. |
| `/presence/inbound` returns `400 fetch failed` | Worker pod/service not reachable yet | Wait for pod Ready plus endpoint propagation; retry once with same smoke id. |
| Ambiguous presence route | Route IDs or selectors collide | Use unique `SMOKE_RUN_ID`/`coderun_id`; delete stale debug routes. |
| `/presence/discord-events` returns `404` in cluster | Deployed bridge image may predate endpoint | Validate direct `/presence/inbound` and report image/revision mismatch. |
| Hermes adapter accepts event but no Hermes input is visible | Local Hermes input endpoint unavailable | Check adapter fallback `presence-inbox.jsonl` path/logs with secrets redacted. |
| `gh run list` returns workflow 404 | Workflow is local-only or not merged to default branch | Keep validation row `UNIT_PASS` until merged and green. |

## Rollback guidance

1. Identify the exact deployed revision/image from GitOps, Actions, and ArgoCD before changing anything.
2. Prefer reverting the offending Git commit or pinning the previous known-good image tag in Git, then letting GitOps reconcile.
3. If an emergency live patch is unavoidable, record the redacted command, reason, timestamp, and follow-up Git commit needed to remove drift.
4. After rollback, rerun the smallest relevant validation:
   - bridge package tests/build for bridge code,
   - adapter package tests/build for adapter code,
   - dry-run Hermes CodeRun smoke,
   - live smoke only if authorized and needed.
5. Update `control-plane-validation-matrix.md` with observed pass/fail evidence.

## Evidence to record

For each validation, record:

- timestamp in UTC,
- branch and commit SHA,
- command/procedure,
- pass/fail/skip result,
- route ID, run ID, pod/app identifiers where safe,
- confirmation that no secrets were printed.
