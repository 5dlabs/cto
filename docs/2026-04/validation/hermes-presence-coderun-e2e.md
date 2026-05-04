# Hermes Presence CodeRun E2E Smoke

Date: 2026-04

## Purpose

Validate the Hermes-first, runtime-neutral Discord control-plane path for a `CodeRun`:

1. A Hermes `CodeRun` is rendered with `spec.harnessAgent: hermes`.
2. The controller injects the `hermes-presence-adapter` sidecar from `crates/controller/src/tasks/code/resources.rs`.
3. The adapter registers a central presence route with `apps/discord-bridge`.
4. A synthetic Discord-shaped event sent to `/presence/inbound` is routed to the adapter.
5. The adapter converts the event into a Hermes input request, falling back to `presence-inbox.jsonl` if the local Hermes input endpoint is unavailable.

The runnable harness is `scripts/presence-smoke-hermes-coderun.py`.

## Safety model

- Default mode is `dry-run`; it renders the `CodeRun` manifest and synthetic inbound payload only.
- Live mode is opt-in via `SMOKE_MODE=live` or `--mode live`.
- The harness never prints `PRESENCE_SHARED_TOKEN`; command output and logs are redacted.
- The shared token is supplied by environment for router calls. For created namespaces only, the script can create a disposable Kubernetes Secret from that env value.
- By default live mode targets the existing `cto` namespace and does **not** create namespaces or secrets.
- Cleanup deletes the smoke `CodeRun` and its presence route unless `SMOKE_KEEP_CODERUN=1` is set.
- No live Discord message is required; this uses the bridge presence API directly.

## Script modes

### Dry run (no cluster mutation)

```bash
scripts/presence-smoke-hermes-coderun.py
# or
SMOKE_MODE=dry-run scripts/presence-smoke-hermes-coderun.py
```

This prints:

- the `agents.platform/v1` `CodeRun` manifest,
- the synthetic `cto.presence.v1` inbound payload,
- the exact defaults that would be used for a live run.

### Live smoke

Prerequisites:

- kube context can access the target namespace and `CodeRun` CRD,
- controller is running with presence enabled,
- `discord-bridge` presence router is reachable from where the script runs,
- the controller's presence config points at the same router and token secret,
- `PRESENCE_SHARED_TOKEN` is available in the shell for the script's router calls.

Typical in-cluster/default service URL:

```bash
export PRESENCE_SHARED_TOKEN='<redacted>'
SMOKE_MODE=live scripts/presence-smoke-hermes-coderun.py
```

When running from a laptop or builder that cannot resolve `*.svc`, port-forward the bridge and override the URL:

```bash
kubectl -n bots port-forward svc/discord-bridge-http 3200:3200
export PRESENCE_ROUTER_URL='http://127.0.0.1:3200'
export PRESENCE_SHARED_TOKEN='<redacted>'
SMOKE_MODE=live scripts/presence-smoke-hermes-coderun.py
```

For a disposable namespace, opt in explicitly:

```bash
export SMOKE_NAMESPACE='cto-presence-hermes-smoke'
export SMOKE_CREATE_NAMESPACE=1
export PRESENCE_SHARED_TOKEN='<redacted>'
SMOKE_MODE=live scripts/presence-smoke-hermes-coderun.py
```

## Environment variables

| Variable | Default | Notes |
| --- | --- | --- |
| `SMOKE_MODE` | `dry-run` | `dry-run` or `live`. |
| `PRESENCE_ROUTER_URL` | `http://discord-bridge-http.bots.svc:3200` | Use `http://127.0.0.1:3200` with port-forwarding. |
| `PRESENCE_SHARED_TOKEN` | unset | Required for live mode; never printed. |
| `PRESENCE_TOKEN_SECRET_NAME` | `openclaw-discord-tokens` | Used only when `SMOKE_CREATE_NAMESPACE=1`. Must match controller presence config for real CodeRun pods. |
| `PRESENCE_TOKEN_SECRET_KEY` | `PRESENCE_SHARED_TOKEN` | Used only when `SMOKE_CREATE_NAMESPACE=1`. |
| `SMOKE_NAMESPACE` | `cto` | Namespace for the smoke `CodeRun`. |
| `SMOKE_CREATE_NAMESPACE` | `0` | If `1`, create/delete the namespace and a token Secret. |
| `SMOKE_KEEP_NAMESPACE` | `0` | Preserve created namespace. |
| `SMOKE_KEEP_CODERUN` | `0` | Preserve the smoke `CodeRun` for debugging. |
| `SMOKE_RUN_ID` | random `hermes-coderun-*` | Also used in labels and synthetic message text. |
| `SMOKE_CODERUN_NAME` | `SMOKE_RUN_ID` | Presence route id should match this name. |
| `SMOKE_AGENT_ID` | `rex` | Populates `implementationAgent`, route `agent_id`, and event `agent_id`. |
| `SMOKE_PROJECT_ID` | `presence-smoke` | Presence route/event project id. |
| `SMOKE_TASK_ID` | `1` | Presence route/event task id. |
| `SMOKE_DISCORD_ACCOUNT_ID` | `control-plane-smoke-account` | Synthetic Discord account id. |
| `SMOKE_DISCORD_CHANNEL_ID` | generated | Synthetic Discord channel id. |
| `SMOKE_DISCORD_THREAD_ID` | unset | Optional synthetic thread id. |
| `SMOKE_REPOSITORY_URL` | `https://github.com/5dlabs/cto` | CodeRun target repository. |
| `SMOKE_DOCS_REPOSITORY_URL` | `https://github.com/5dlabs/cto` | CodeRun docs repository. |
| `SMOKE_CLI_TYPE` | `codex` | Rendered into `spec.cliConfig.cliType`. |
| `SMOKE_MODEL` | `gpt-5-codex` | Rendered into `spec.model` and `spec.cliConfig.model`. |
| `SMOKE_PROVIDER` | unset | Optional `spec.cliConfig.provider`. |
| `SMOKE_PROMPT` | synthetic no-op prompt | Rendered into `spec.promptModification`. |
| `SMOKE_WAIT_TIMEOUT` | `240` | Seconds to wait for route/pod observations. |

## Expected live output

A passing run should show:

```text
[smoke] mode=live
[smoke] route registered: hermes-coderun-...
[smoke] posting synthetic Discord event through /presence/inbound
[smoke] adapter pod observed: ...
[smoke] passed
```

The critical assertion is the HTTP `202` from `/presence/inbound` after the adapter-created route appears. Pod log inspection is best-effort because image/log wording may change.

## Manual validation and debugging

List the smoke resources:

```bash
kubectl -n "$SMOKE_NAMESPACE" get coderun,pod -l smoke.5dlabs.ai/type=hermes-presence-coderun-e2e
```

Check the registered route without leaking the token in shell history by using the environment variable:

```bash
curl -fsS \
  -H "Authorization: Bearer ${PRESENCE_SHARED_TOKEN}" \
  "${PRESENCE_ROUTER_URL:-http://127.0.0.1:3200}/presence/routes" \
  | jq '.routes[] | select(.metadata.source == "hermes-presence-adapter" or .route_id == env.SMOKE_CODERUN_NAME)'
```

Inspect adapter logs:

```bash
POD=$(kubectl -n "$SMOKE_NAMESPACE" get pod -l smoke.5dlabs.ai/run-id="$SMOKE_RUN_ID" -o jsonpath='{.items[0].metadata.name}')
kubectl -n "$SMOKE_NAMESPACE" logs "pod/$POD" -c hermes-presence-adapter --tail=200
```

Inspect inbox fallback inside the pod if Hermes input was unavailable:

```bash
kubectl -n "$SMOKE_NAMESPACE" exec "pod/$POD" -c hermes-presence-adapter -- \
  sh -c 'find /workspace -name presence-inbox.jsonl -maxdepth 4 -type f -print -exec tail -n 5 {} \\;'
```

## Redaction requirements

When attaching output to issues or docs:

- replace `PRESENCE_SHARED_TOKEN`, bearer values, API keys, and any Discord bot tokens with `<redacted>`,
- do not paste full Kubernetes Secret manifests,
- include route ids, CodeRun names, status codes, pod names, and non-secret synthetic channel ids only.

The harness performs best-effort redaction, but reviewers should still inspect logs before sharing externally.

## Relationship to existing smokes

- `scripts/presence-smoke.sh` validates direct route registration to a disposable worker.
- `scripts/presence-morgan-task-smoke.py` validates Morgan/task-channel routing behavior without creating a real CodeRun.
- `scripts/presence-smoke-hermes-coderun.py` validates the controller-rendered Hermes CodeRun path and the adapter-created route.

## Known limitations

- Live mode may start a real CodeRun pod. Keep the prompt no-op and use disposable names.
- If the deployed controller image predates Hermes presence injection, route registration will time out.
- If `discord-bridge` is reachable only inside the cluster, use port-forwarding for the script or run it from an in-cluster debug pod.
- If the local Hermes `/input` endpoint is absent or not ready, the adapter should still accept the event and append to `presence-inbox.jsonl`; this still validates central routing.
