# Morgan/task routing semi-live smoke — 2026-05-04

Timestamp: 2026-05-04T15:xxZ

## Scope

Semi-live validation for the centralized Discord control-plane routing shape expected by the Morgan + task-channel workflow. This is not a live Discord ingress test and does not prove the Morgan sidecar/MCP implementation exists. It exercises the deployed bridge contract with temporary in-cluster worker infrastructure and synthetic normalized `cto.presence.v1` events.

The smoke validates:

- A DM/home-style direct inbound event routes to the Morgan route.
- A Sigma One task-channel direct inbound event routes to the Rex task worker route.
- An ambient task-channel direct event fails closed instead of reaching the Morgan DM route.
- The currently deployed bridge image still does not expose `/presence/discord-events`, so live Discord-style fanout remains skipped rather than passed.

## Preconditions checked

Commands used Kubernetes authorization checks and resource names only. Secret values were not printed.

```text
kubectl auth can-i create namespaces -> yes
kubectl auth can-i delete namespaces -> yes
kubectl auth can-i create secrets -n cto -> yes
kubectl auth can-i create pods -n cto -> yes
kubectl auth can-i create services -n cto -> yes
kubectl auth can-i get pods -n cto -> yes
kubectl auth can-i get services/discord-bridge-http -n bots -> yes
kubectl get secret openclaw-discord-tokens -n cto -o jsonpath='{.metadata.name}' -> openclaw-discord-tokens
```

The presence token was sourced from Kubernetes Secret key `PRESENCE_SHARED_TOKEN` into process environment only and was not printed or written.

## Command

```bash
PRESENCE_SHARED_TOKEN="***" python3 scripts/presence-morgan-task-smoke.py
```

## Observed safe output

```text
[smoke] run_id=morgan-task-wohej430
[smoke] router=http://discord-bridge-http.bots.svc:3200
[smoke] namespace=cto-presence-smoke-morgan-task-wohej430
[smoke] project=sigma-1 task=1
[smoke] dm_to_morgan: HTTP 202
[smoke] task_chan_to_rex: HTTP 202
[smoke] ambient_task_chan_fails_closed_direct_path: HTTP 404
[smoke] discord-events endpoint is not in the currently deployed bridge image; skipped live ambient fanout check
[smoke] passed
[smoke] DM channel modeled as dm-morgan-task-wohej430; task chan modeled as task-chan-sigma-1-task-1-morgan-task-wohej430
```

## Cleanup verification

```text
kubectl get namespace cto-presence-smoke-morgan-task-wohej430 -o name
# Error from server (NotFound): namespaces "cto-presence-smoke-morgan-task-wohej430" not found

# Authenticated route registry inspection after cleanup:
{"matching_route_count": 0, "matching_route_ids": [], "route_count": 4}
```

## Result

This is useful semi-live integration evidence for the centralized routing contract that Morgan will need, but it does not move Morgan sidecar/MCP rows out of `BLOCKED`: there is still no durable Morgan sidecar image/package, CodeRun sidecar attachment, MCP tool discovery, `meet-init` readiness gate, or `/workspace/meet-*.json*` stream implementation.

The skipped `/presence/discord-events` check is the clearest next integration blocker for promoting Discord-style fanout rows: the currently deployed bridge image did not expose that endpoint, so the smoke could validate only direct `/presence/inbound` synthetic delivery.
