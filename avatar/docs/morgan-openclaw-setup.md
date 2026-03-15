# Morgan and OpenClaw Agent Config

This doc summarizes how OpenClaw agents are set up in **openclaw-platform** so Morgan’s gateway config can be aligned. The avatar PoC sends requests to `morgan.5dlabs.ai` with `x-openclaw-agent-id: morgan`; the OpenClaw side must have an agent with that id and a gateway that accepts chat-completion traffic.

## Reference: openclaw-platform

Clone (or use existing):

```bash
git clone git@github.com:5dlabs/openclaw-platform.git
```

### How an agent is defined

1. **Helm values** — One values file per agent under `charts/openclaw/agents/<id>/values.yaml` (e.g. `agents/metal/values.yaml`, `agents/intake/values.yaml`).

2. **Required agent fields** (from `values.yaml` and chart defaults):
   - `agent.id` — Unique id (e.g. `"metal"`, `"intake"`). This is what the gateway uses (and what we send as `x-openclaw-agent-id: morgan`).
   - `agent.name` — Display name.
   - `agent.model` — Model key (e.g. `anthropic/claude-sonnet-4-6`, `openai-api/gpt-5.4-pro`).
   - Optional: `agent.heartbeat`, `agent.sandbox`, `agent.tools`, `messaging`, `secrets`, etc.

3. **ConfigMap** — The chart renders `configmap-openclaw.yaml` into an `openclaw.json` that includes:
   - `agents.list[]` with one entry per agent: `id`, `name`, `model`, `agentDir`, `heartbeat`, `sandbox`, `tools`, `workspace`.
   - Gateway config (e.g. `gateway.http.endpoints.chatCompletions.enabled: true`, port 18789).

4. **Deployment** — Each agent gets its own StatefulSet/Deployment and Pod (e.g. `openclaw-metal-openclaw-0`, `openclaw-intake-openclaw-0`). The main container is `agent`; it runs the OpenClaw binary and listens on 18789.

### Current cluster (openclaw namespace)

From a recent check there are **no** pods named `openclaw-morgan-*`. Existing agents include conductor, current, forge, healer, holt, infra, intake, keeper, metal (and mem0/openmemory sidecars). So either:

- Morgan is deployed under a different name or in another namespace, or
- Morgan is not yet deployed as an OpenClaw agent, and `morgan.5dlabs.ai` is pointing at a different service (e.g. a shared gateway that must route by `x-openclaw-agent-id` to a backend that has `morgan` in its `agents.list`).

For the avatar to get a Morgan reply, the **request that hits the gateway for morgan.5dlabs.ai must be served by an OpenClaw process that has an agent with `id: "morgan"`** in its config.

## Reading OpenClaw pod logs

Use the pod that actually serves traffic for `morgan.5dlabs.ai` (e.g. the one the Cloudflare tunnel targets). The main OpenClaw process runs in the `agent` container.

```bash
# List OpenClaw pods
kubectl get pods -n openclaw -l app.kubernetes.io/part-of=openclaw

# Logs from the main agent container (last 150 lines)
kubectl logs -n openclaw <pod-name> -c agent --tail=150

# Example: intake
kubectl logs -n openclaw openclaw-intake-openclaw-0 -c agent --tail=150
```

Look for:

- Startup: `[gateway] agent model: ...` and `[gateway] listening on ws://0.0.0.0:18789`
- Errors or timeouts when handling chat-completion requests
- Any mention of agent id, routing, or missing config

## What Morgan needs (summary)

1. **An OpenClaw agent with `agent.id: "morgan"`** in the gateway config that serves `morgan.5dlabs.ai` (either a dedicated morgan pod or a shared gateway that routes by `x-openclaw-agent-id` to that agent).
2. **Tunnel** — `morgan.5dlabs.ai` CNAME and Cloudflare tunnel (e.g. `morgan-avatar-tunnel-credentials`) pointing at that gateway’s port (e.g. 18789).
3. **Avatar client** — Sends `x-openclaw-agent-id: morgan` on every request (already done in avatar `providers.py` when `MORGAN_LLM_AGENT_ID=morgan`).

If the pod that receives traffic for morgan.5dlabs.ai does **not** have `morgan` in its `agents.list`, you’ll get generic or failing behavior. Fix by deploying Morgan as an OpenClaw agent (e.g. via CTO Helm/ArgoCD, following the same pattern as `agents/metal` or `agents/intake`) and ensuring the tunnel targets that deployment.

### Response in logs but no audio in avatar

If OpenClaw logs show Morgan’s reply (e.g. “Hey Johnny! …”) but the user never hears it in the browser:

- **400 / model does not exist** — The Morgan agent’s `agent.model` in OpenClaw is set to an invalid model (e.g. `openai/gpt-5.3`). The gateway returns 400, so the avatar agent never gets a stream and has nothing to send to TTS. **Fix**: In the Morgan OpenClaw agent config (Helm values or openclaw.json), set `agent.model` to a valid model (e.g. `openai/gpt-4o`, `openai/gpt-4.1`, or whatever your provider supports). Restart the pod so the new model is used; then the gateway will return 200 and stream the reply to the avatar so TTS can play it.

## Key paths in openclaw-platform

| Path | Purpose |
|------|---------|
| `charts/openclaw/values.yaml` | Defaults, gateway (port 18789, chatCompletions), agentDefaults |
| `charts/openclaw/agents/<id>/values.yaml` | Per-agent overrides (agent.id, name, model, heartbeat, messaging, etc.) |
| `charts/openclaw/templates/agent/configmap-openclaw.yaml` | Renders openclaw.json (agents.list, gateway, NATS, etc.) |
| `charts/openclaw/templates/agent/statefulset.yaml` | Pod spec, volume mounts, agent container |
