# Hermes CodeRun Presence Live Smoke — 2026-05-04

Timestamp: 2026-05-04T07:28Z–07:36Z

## Scope

Live/semi-live validation for the runtime-neutral Discord control plane after PR #4925 merged and the `hermes-control-plane-builder` ArgoCD application synced.

This smoke did not send a real Discord message. It validated the safer integration path that creates a temporary Hermes `CodeRun`, waits for its presence route, posts a synthetic normalized `cto.presence.v1` event through the central bridge `/presence/inbound` endpoint, and then deletes the temporary CodeRun/route.

## Preconditions checked

Commands used only resource names and Kubernetes authorization checks; no secret values were printed.

```bash
gh pr view 4925 --json state,mergedAt,mergeCommit,url
kubectl -n argocd get application hermes-control-plane-builder -o jsonpath='{.spec.source.targetRevision}{"\n"}{.status.sync.status}{"\n"}{.status.health.status}{"\n"}{.status.sync.revision}{"\n"}'
kubectl auth can-i get secrets/openclaw-discord-tokens -n cto
kubectl auth can-i get pods -n cto
kubectl auth can-i get services -n bots
kubectl auth can-i create pods -n cto
kubectl auth can-i create coderuns -n cto
kubectl auth can-i delete coderuns -n cto
kubectl get secret openclaw-discord-tokens -n cto -o jsonpath='{.metadata.name}{"\n"}'
kubectl get svc -n bots discord-bridge-http -o jsonpath='{.metadata.name}{"\n"}'
```

Observed:

- PR #4925 merged at `2026-05-04T06:52:47Z` with merge commit `67c415654fa22ee27fc62dd72c649a518b280ca7`.
- Local `main` was reset to `origin/main` at `4a08272fe030a499c05b250d87f1f145942d522f` after creating backup branch `local-main-before-pr4925-sync-20260504T072823Z`.
- ArgoCD application `hermes-control-plane-builder` target revision: `main`; sync: `Synced`; health: `Healthy`; revision: `4a08272fe030a499c05b250d87f1f145942d522f`.
- RBAC prerequisites returned `yes` for reading the presence token secret name, pods, and bridge service, and for creating/deleting temporary CodeRun resources.
- Safe resource-name reads confirmed `openclaw-discord-tokens` in namespace `cto` and service `discord-bridge-http` in namespace `bots` exist.

## Live smoke command

The token was sourced from Kubernetes Secret key `PRESENCE_SHARED_TOKEN` and passed only as an environment variable to the smoke script. The token value was not printed.

```bash
# Token sourced from Kubernetes Secret key PRESENCE_SHARED_TOKEN and held only in-process; value omitted.
PRESENCE_SHARED_TOKEN="<redacted>" python3 scripts/presence-smoke-hermes-coderun.py --mode live
```

Observed safe output excerpts:

```text
[smoke] mode=live
[smoke] $ kubectl apply -f /tmp/tmp9mlpte6k.yaml
[smoke] route registered: hermes-coderun-7fwzkv6n
[smoke] posting synthetic Discord event through /presence/inbound
[smoke] route delivered, but adapter pod was not found by smoke label before timeout
[smoke] passed
[smoke] $ kubectl -n cto delete coderun hermes-coderun-7fwzkv6n --ignore-not-found
```

Cleanup verification:

```bash
kubectl -n cto get coderun hermes-coderun-7fwzkv6n -o name
# Error from server (NotFound): coderuns.agents.platform "hermes-coderun-7fwzkv6n" not found

# Authenticated route registry inspection reported route_count 4 and no route_id/coderun_id matching hermes-coderun-7fwzkv6n.
```

## Result

PASS for the live route-registration, authenticated synthetic inbound, HTTP `202` bridge delivery, and cleanup/deletion slice:

- Temporary Hermes CodeRun manifest applied.
- Presence route `hermes-coderun-7fwzkv6n` registered.
- Synthetic normalized `cto.presence.v1` event posted through bridge `/presence/inbound` and accepted with the smoke script's expected success path.
- Temporary CodeRun was deleted and the route was absent afterward.
- No secret values were printed.

Remaining gap before strongest end-to-end acceptance: the harness did not find the adapter pod by smoke label before timeout, so this run lacks adapter-log or worker-consumption proof. A stronger follow-up should either fix pod-label discovery or add a redacted inbox/adapter acknowledgement check, then run live Discord ingress/outbound evidence.

## Follow-up live smoke — 2026-05-04T09:33Z

A follow-up heartbeat hardened `scripts/presence-smoke-hermes-coderun.py` to discover the controller-rendered adapter pod by fallback labels when the CR-level `smoke.5dlabs.ai/run-id` label is not preserved on Jobs/Pods. The new lookup order is:

1. `smoke.5dlabs.ai/run-id=<run-id>`
2. `cleanup.5dlabs.ai/run=<coderun-name>`
3. `app=controller,component=code-runner,service=<service>`

Live command used the same no-secret pattern: source Kubernetes Secret key `PRESENCE_SHARED_TOKEN` into process environment only, then run `python3 scripts/presence-smoke-hermes-coderun.py --mode live`. No token value was printed or written.

Observed safe output excerpts:

```text
[smoke] mode=live
[smoke] route registered: hermes-coderun-zc45za2s
[smoke] posting synthetic Discord event through /presence/inbound
[smoke] adapter pod discovered with selector app=controller,component=code-runner,service=presence-smoke: t1-codex-gpt-5-codex-default-65c71de1-v1-fnrvx
[smoke] adapter pod observed: t1-codex-gpt-5-codex-default-65c71de1-v1-fnrvx
[smoke] passed
[smoke] $ kubectl -n cto delete coderun hermes-coderun-zc45za2s --ignore-not-found
```

Cleanup verification after the run:

```text
kubectl -n cto get coderun hermes-coderun-zc45za2s -o name
# Error from server (NotFound): coderuns.agents.platform "hermes-coderun-zc45za2s" not found

# Authenticated route registry inspection reported route_count 4 and matching_route_count 0 for hermes-coderun-zc45za2s.
```

Result: this upgrades the previous synthetic Hermes CodeRun live smoke from route-delivery-only to adapter-pod-observed evidence for the same route registration, authenticated `/presence/inbound` HTTP `202`, adapter container discovery/log-tail, and cleanup slice. It still does not prove a real Discord ingress event or an end-to-end Hermes worker semantic response.

## Follow-up live smoke — 2026-05-04T10:40Z

A second post-merge heartbeat reran the live Hermes CodeRun smoke against current `origin/main`/ArgoCD revision `b6dd2e33cf5ba937152abf3767b0262f24542dd6` after PR #4928 merged. The token was again sourced from Kubernetes Secret key `PRESENCE_SHARED_TOKEN` into process environment only and was not printed or written.

Preconditions immediately before the run:

```text
kubectl auth can-i get secrets/openclaw-discord-tokens -n cto -> yes
kubectl auth can-i get pods -n cto -> yes
kubectl auth can-i get services -n bots -> yes
kubectl auth can-i create pods -n cto -> yes
kubectl auth can-i create coderuns -n cto -> yes
kubectl auth can-i delete coderuns -n cto -> yes
hermes-control-plane-builder targetRevision=main, sync=Synced, health=Healthy, revision=b6dd2e33cf5ba937152abf3767b0262f24542dd6
```

Observed safe output excerpts:

```text
[smoke] mode=live
[smoke] route registered: hermes-coderun-x9j4jq97
[smoke] route summary: {"agent_id": "rex", "coderun_id": "hermes-coderun-x9j4jq97", "project_id": "presence-smoke", "route_id": "hermes-coderun-x9j4jq97", "runtime": "hermes", "task_id": "1", "worker_url_present": true}
[smoke] posting synthetic Discord event through /presence/inbound
[smoke] adapter pod discovered with selector app=controller,component=code-runner,service=presence-smoke: t1-codex-gpt-5-codex-default-3995f4d1-v1-g7bt7
[smoke] adapter pod observed: t1-codex-gpt-5-codex-default-3995f4d1-v1-g7bt7
[smoke] passed
[smoke] $ kubectl -n cto delete coderun hermes-coderun-x9j4jq97 --ignore-not-found
```

Cleanup verification after the run:

```text
kubectl -n cto get coderun hermes-coderun-x9j4jq97 -o name
# Error from server (NotFound): coderuns.agents.platform "hermes-coderun-x9j4jq97" not found

# Authenticated route registry inspection reported route_count 4 and matching_route_count 0 for hermes-coderun-x9j4jq97.
```

Result: repeated PASS evidence for the live synthetic Hermes CodeRun slice on current `main`: route registration, authenticated `/presence/inbound` delivery, adapter pod discovery/log-tail, and cleanup. This still does not prove real Discord ingress/outbound or semantic Hermes worker response, so H-02/H-03/H-04/H-20 remain `NOT_STARTED`.

## Adjacent CI/image evidence

After PR #4925 merged, the Discord bridge publish workflow succeeded on push run `25305273050` for commit `67c415654fa22ee27fc62dd72c649a518b280ca7`.

The new Hermes presence adapter publish workflow ran on push run `25305273066`; its test/build job succeeded, but the publish job failed with GHCR `write_package` permission denial while pushing `ghcr.io/5dlabs/hermes-presence-adapter:latest`. This blocks image-publish `PASS` for the adapter until package permissions or workflow registry permissions are corrected. A 2026-05-04T10:40Z recheck still showed run `25305273066` as the latest `main` push run for `hermes-presence-adapter-publish.yml` and still `failure`; no newer successful publish run exists yet.
