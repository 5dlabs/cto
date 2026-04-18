# Testing the CTO Sidebar ↔ OpenClaw gateway wiring

The sidebar talks to the per-pod OpenClaw gateway over its OpenAI-compatible
HTTP API. Inside a controller-launched code-server pod the gateway runs in the
`agent` sidecar on `http://localhost:18789`. On a dev host you reach the same
gateway via `kubectl port-forward`.

## Gateway facts (verified against a live `cto` pod)

- Bind: the `agent` container of the agent pod, port `18789`.
- Chat: `POST /v1/chat/completions` (OpenAI-compat, streaming SSE with
  `data: {…}` lines terminated by `data: [DONE]`).
- Models: `GET /v1/models` returns ids like `openclaw`, `openclaw/default`,
  `openclaw/coder`. The sidebar maps an agent id to `openclaw/<agent>`.
- Auth: `Authorization: Bearer openclaw-internal` (static shared token from
  `/workspace/.openclaw/openclaw.json` → `gateway.auth.token`). Override with
  the `cto.apiToken` VS Code setting if a deployment rotates it.
- Health: `GET /health` → `{"ok":true,"status":"live"}` (no auth).

## Settings

| Setting | Default | Purpose |
| --- | --- | --- |
| `cto.apiBase` | `http://localhost:18789` | Gateway base URL. |
| `cto.apiToken` | `openclaw-internal` | Bearer token. |
| `cto.defaultAgent` | `""` | Selected agent id (→ model `openclaw/<id>`). |
| `cto.requestTimeoutMs` | `30000` | First-byte timeout for chat. |

## In-pod smoke test

```bash
POD=$(kubectl -n cto get pods -l app=openclaw-coder -o jsonpath='{.items[0].metadata.name}')

kubectl -n cto exec "$POD" -c agent -- \
  curl -sS http://localhost:18789/health
# {"ok":true,"status":"live"}

kubectl -n cto exec "$POD" -c agent -- \
  curl -sS -H "Authorization: Bearer openclaw-internal" \
    http://localhost:18789/v1/models
# {"object":"list","data":[{"id":"openclaw",...},{"id":"openclaw/default",...},...]}

kubectl -n cto exec "$POD" -c agent -- sh -c 'curl -sS -N \
  -H "Authorization: Bearer openclaw-internal" \
  -H "Content-Type: application/json" \
  -X POST http://localhost:18789/v1/chat/completions \
  -d "{\"model\":\"openclaw\",\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}],\"stream\":true}"'
# data: {"id":"chatcmpl_…","choices":[{"delta":{"role":"assistant"}}]}
# data: {"id":"chatcmpl_…","choices":[{"delta":{"content":"Hi"}}]}
# …
# data: [DONE]
```

## Dev-host smoke test (port-forward)

```bash
kubectl -n cto port-forward svc/openclaw-coder 18789:18789
# separate terminal
curl -sS http://localhost:18789/health
curl -sS -H "Authorization: Bearer openclaw-internal" http://localhost:18789/v1/models
```

Then in VS Code `settings.json`:

```json
{
  "cto.apiBase": "http://localhost:18789",
  "cto.apiToken": "openclaw-internal"
}
```

## Error-banner matrix

| Condition | Expected banner |
| --- | --- |
| Gateway down (`ECONNREFUSED` / fetch failed) | *"Cannot reach OpenClaw gateway at …. The sidecar may not be running …"* |
| First-byte timeout (`cto.requestTimeoutMs` elapsed) | *"Gateway did not respond within Ns at …. Is the OpenClaw sidecar running?"* |
| 401/403 | *"Gateway rejected the bearer token …. Check the `cto.apiToken` setting."* |
| 5xx | *"Gateway returned 5xx … at …. Check the agent pod logs."* |
| Other 4xx | *"Gateway error NNN … at …."* |
| Mid-stream failure after first byte | *"Stream interrupted: …"* |

## Manual UI test

1. Install `cto-sidebar-0.1.1.vsix` into a code-server instance (either a
   port-forwarded host VS Code or a running agent pod).
2. Open the CTO activity-bar view; the agent list populates from
   `GET /v1/models` (falls back to the built-in roster when the gateway is
   unreachable — marked `offline`).
3. Type a prompt and send. You should see deltas stream in word-by-word
   rather than the entire cumulative buffer being replaced on every chunk.
4. Break the connection (`cto.apiToken` = `"bogus"`) and retry — you should
   see the 401 banner, not the generic "Connection failed".
