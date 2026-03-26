# Morgan OpenClaw Setup (CTO Repo)

This runbook documents the Morgan OpenClaw deployment used by the avatar PoC.
It is source-verified against the CTO repo manifests, not a one-off cluster snapshot.

The avatar pipeline sends chat requests to `morgan.5dlabs.ai` and must include
`x-openclaw-agent-id: morgan`.

## Canonical config paths

| Path | Purpose |
|------|---------|
| `infra/gitops/agents/morgan-values.yaml` | Morgan OpenClaw Helm values (agent identity, model, gateway, tools, skills mount) |
| `infra/gitops/applications/workloads/deliberation-agents.yaml` | ArgoCD ApplicationSet that deploys the `morgan` OpenClaw workload |
| `infra/manifests/morgan-support/cluster-tunnel.yaml` | Dedicated Cloudflare tunnel (`morgan-avatar`) |
| `infra/manifests/morgan-support/tunnel-binding.yaml` | Binds `morgan.5dlabs.ai` to `openclaw-morgan` service on port `18789` |
| `infra/manifests/morgan-support/skills-configmap.yaml` | External skills ConfigMap mounted into Morgan |
| `infra/gitops/applications/workloads/morgan-support.yaml` | ArgoCD Application for tunnel + skills support resources |

## Current declared runtime contract

From `infra/gitops/agents/morgan-values.yaml`:

- `agent.id: morgan`
- `agent.name: Morgan`
- `agent.model: openai-api/gpt-5.4-pro`
- Gateway `chatCompletions` endpoint enabled (`responses` disabled)
- `tools.profile: minimal`, web search/fetch disabled, ACP disabled
- External skills ConfigMap: `openclaw-morgan-external-skills`

From `infra/manifests/morgan-support/tunnel-binding.yaml`:

- `morgan.5dlabs.ai` is routed to `http://openclaw-morgan.openclaw.svc:18789`
- Routing is through ClusterTunnel `morgan-avatar`

## Validate deployment state quickly

```bash
# 1) Confirm Argo apps are synced
kubectl get applications.argoproj.io -n argocd \
  cto-deliberation-morgan morgan-support

# 2) Confirm Morgan pod/service exist
kubectl get pods,svc -n openclaw | rg 'morgan|openclaw-morgan'

# 3) Confirm tunnel binding is present
kubectl get tunnelbinding -n openclaw morgan-gateway -o yaml

# 4) Check gateway logs from the Morgan pod
kubectl logs -n openclaw <morgan-pod-name> -c agent --tail=200
```

Look for:

- Gateway startup on port `18789`
- Successful chat-completion requests
- No model/provider errors

## Avatar-side routing requirement

The avatar agent only reaches Morgan persona when the OpenClaw header is set:

```http
x-openclaw-agent-id: morgan
```

`avatar/agent/morgan_avatar_agent/config.py` defaults `MORGAN_LLM_AGENT_ID` to
`morgan`, and `avatar/agent/morgan_avatar_agent/providers.py` adds the header
when calling OpenClaw.

## Failure modes and fixes

- 401/403 from gateway: `OPENCLAW_TOKEN`/`MORGAN_LLM_API_KEY` mismatch between avatar agent and gateway auth.
- 400 model/provider errors: verify `agent.model` and provider flags in `morgan-values.yaml`, then resync Argo app.
- Generic/non-Morgan responses: missing `x-openclaw-agent-id` header or stale Morgan config not rolled out.
- No remote access at `morgan.5dlabs.ai`: check `morgan-support` app sync and TunnelBinding status/events.

## Notes

Avoid relying on static pod-name examples in docs; validate live names with
`kubectl get` commands because names can change across rollouts.
