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

## Adjacent CI/image evidence

After PR #4925 merged, the Discord bridge publish workflow succeeded on push run `25305273050` for commit `67c415654fa22ee27fc62dd72c649a518b280ca7`.

The new Hermes presence adapter publish workflow ran on push run `25305273066`; its test/build job succeeded, but the publish job failed with GHCR `write_package` permission denial while pushing `ghcr.io/5dlabs/hermes-presence-adapter:latest`. This blocks image-publish `PASS` for the adapter until package permissions or workflow registry permissions are corrected.
