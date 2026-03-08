# Cloudflare Tunnel Setup for Intake Agent Testing

Instructions for ensuring the intake testing agent has access to the Cloudflare Tunnel setup for webhook-based communication with Linear and external callers.

## Context

The intake pipeline now uses HTTP webhooks instead of NATS. Linear webhooks and per-run callbacks arrive at `agents.5dlabs.ai` via a Cloudflare Tunnel, which routes to `linear-bridge` in the `bots` namespace. The testing agent needs to verify this tunnel routing works end-to-end.

## Existing Infrastructure

### ClusterTunnel (already deployed)

The platform has a single ClusterTunnel managed by the Cloudflare Tunnel Operator:

```yaml
# infra/charts/cto/templates/cloudflare/cluster-tunnel.yaml
apiVersion: networking.cfargotunnel.com/v1alpha2
kind: ClusterTunnel
metadata:
  name: cto-main
spec:
  existingTunnel:
    id: "87889b67-ee20-4ed1-8de7-fb43d1b5156f"
    name: cto-main
  cloudflare:
    domain: 5dlabs.ai
    secret: cto-main-tunnel-credentials
    accountId: b73ec19faa187789b3f9d1deb0e0d95f
    email: j@jonathonfritz.com
  fallbackTarget: http_status:404
```

### TunnelBinding for Agent Webhooks (new, needs deployment)

```yaml
# infra/manifests/linear-bridge/tunnel-binding.yaml
apiVersion: networking.cfargotunnel.com/v1alpha1
kind: TunnelBinding
metadata:
  name: agent-webhooks
  namespace: bots
spec:
  tunnelRef:
    kind: ClusterTunnel
    name: cto-main
  subjects:
    - hostname: agents.5dlabs.ai
      target: http://linear-bridge.bots.svc:3100
```

This routes ALL paths on `agents.5dlabs.ai` to linear-bridge on port 3100. Path-based routing happens inside linear-bridge:

| Path | Handler |
|------|---------|
| `POST /webhooks/linear` | Linear Agent Session webhooks (created, prompted) |
| `POST /runs/{runId}/callback` | Per-run webhook callback (Linear select responses) |
| `POST /notify` | Agent message → Linear comment |
| `POST /elicitation` | Elicitation → Linear select signal |
| `POST /elicitation/cancel` | Cancel pending elicitation (answered elsewhere) |
| `POST /runs/{runId}/register` | Register run → {agent, sessionKey, issueId} |
| `DELETE /runs/{runId}` | Deregister run |
| `GET /health` | Health check |

## What You Need To Do

### 1. Deploy the TunnelBinding

The TunnelBinding manifest already exists at `infra/manifests/linear-bridge/tunnel-binding.yaml`. Apply it:

```bash
kubectl apply -f infra/manifests/linear-bridge/tunnel-binding.yaml
```

Verify the binding was created and the operator picked it up:

```bash
kubectl get tunnelbindings -n bots
# Should show: agent-webhooks

# Check the tunnel operator logs for the route being added:
kubectl logs -n cloudflare-operator-system deployment/cloudflare-operator-controller-manager --tail=20 | grep agents.5dlabs.ai
```

### 2. Verify DNS Resolution

The Cloudflare Tunnel Operator should auto-create the CNAME record. Verify:

```bash
dig agents.5dlabs.ai CNAME +short
# Expected: 87889b67-ee20-4ed1-8de7-fb43d1b5156f.cfargotunnel.com (or similar)
```

If the CNAME isn't created automatically, add it manually in the Cloudflare dashboard:
- **Type**: CNAME
- **Name**: agents
- **Target**: `87889b67-ee20-4ed1-8de7-fb43d1b5156f.cfargotunnel.com`
- **Proxy**: ON (orange cloud)

### 3. Verify Linear Bridge is Running

```bash
kubectl get pods -n bots -l app=linear-bridge
# Should show 1/1 Running

kubectl get svc -n bots linear-bridge
# Should show ClusterIP on port 3100
```

### 4. Test the Tunnel End-to-End

From outside the cluster (your local machine):

```bash
# Health check
curl -s https://agents.5dlabs.ai/health
# Expected: {"status":"ok"} or 200

# If linear-bridge isn't deployed yet, you'll get the fallback 404
curl -s -o /dev/null -w "%{http_code}" https://agents.5dlabs.ai/health
# Expected: 200 (bridge running) or 404 (fallback, bridge not yet deployed)
```

From inside the cluster (any pod):

```bash
# Direct service call (bypasses tunnel, verifies bridge is serving)
curl -s http://linear-bridge.bots.svc:3100/health

# Through tunnel (verifies full path)
curl -s https://agents.5dlabs.ai/health
```

### 5. Configure Linear Webhook

In the Linear Agent Session settings, set the webhook URL to:

```
https://agents.5dlabs.ai/webhooks/linear
```

Linear will POST to this URL when:
- A new agent session is **created** (user delegates an issue)
- A session is **prompted** (user sends a message, selects an option, or clicks stop)

The linear-bridge verifies the webhook signature using `LINEAR_WEBHOOK_SECRET` from the `openclaw-linear` secret (pulled from OpenBao via ExternalSecret).

### 6. Discord Bridge HTTP Access (Internal Only)

The discord-bridge HTTP API is cluster-internal only (no tunnel exposure needed):

```yaml
# infra/manifests/discord-bridge/service-http.yaml
apiVersion: v1
kind: Service
metadata:
  name: discord-bridge-http
  namespace: bots
spec:
  type: ClusterIP
  ports:
    - name: http
      port: 3200
      targetPort: 3200
  selector:
    app: discord-bridge
```

Workflow steps POST to `http://discord-bridge-http.bots.svc:3200/notify` for Discord notifications. This is already configured in linear-bridge's `DISCORD_BRIDGE_URL` env var:

```yaml
- name: DISCORD_BRIDGE_URL
  value: "http://discord-bridge.bots.svc:3200"
```

Apply the discord-bridge HTTP service:

```bash
kubectl apply -f infra/manifests/discord-bridge/service-http.yaml
```

## Secrets Required

The following secrets must exist in OpenBao at `secret/openclaw-linear`:

| Key | Description |
|-----|-------------|
| `LINEAR_API_KEY` | Linear API key with agent session permissions |
| `LINEAR_WEBHOOK_SECRET` | Webhook signing secret (configured in Linear webhook settings) |

These are pulled into the `openclaw-linear` Kubernetes secret by the ExternalSecret in `infra/manifests/linear-bridge/deployment.yaml`.

## Architecture Diagram

```
Internet
  │
  │  HTTPS (agents.5dlabs.ai)
  ▼
Cloudflare Edge (proxy: ON)
  │
  │  cloudflared tunnel (QUIC/HTTP2)
  ▼
cloudflare-operator-system/cloudflared pod
  │
  │  HTTP (TunnelBinding route)
  ▼
bots/linear-bridge:3100
  │
  ├── /webhooks/linear     (Linear Agent Session events)
  ├── /runs/{id}/callback  (per-run webhook callbacks)
  ├── /notify              (workflow → Linear comment)
  ├── /elicitation         (workflow → Linear select signal)
  └── /health

  ┌──────────────────────┐
  │ bots/discord-bridge   │
  │ :3200 (cluster-only)  │
  │ ├── /notify           │
  │ ├── /elicitation      │
  │ └── /health           │
  └──────────────────────┘
```

## Verification Checklist

- [ ] `kubectl get tunnelbindings -n bots` shows `agent-webhooks`
- [ ] `dig agents.5dlabs.ai CNAME +short` returns tunnel FQDN
- [ ] `curl https://agents.5dlabs.ai/health` returns 200
- [ ] `kubectl get svc -n bots discord-bridge-http` shows ClusterIP:3200
- [ ] Linear webhook configured to `https://agents.5dlabs.ai/webhooks/linear`
- [ ] ExternalSecret `linear-bridge-secrets` in `bots` namespace is synced (check `kubectl get externalsecret -n bots`)
