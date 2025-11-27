# Cloudflare Tunnel Migration

This document describes the work completed to migrate from NGrok to Cloudflare Tunnel for exposing services from the CTO platform.

## Overview

The CTO platform previously used NGrok for:
1. **GitHub Webhooks** - Receiving webhook events from GitHub for automation triggers
2. **Application Publishing** - Exposing preview deployments via Bolt agents

This migration replaces NGrok with Cloudflare Tunnel using the community `cloudflare-operator` by adyanth.

## Architecture Comparison

### Previous (NGrok)
```
Internet → github.public.5dlabs.ai → NGrok Edge → NGrok Operator (Gateway API) → Service
```

### New (Cloudflare Tunnel)
```
Internet → github.public.5dlabs.ai → Cloudflare Edge → cloudflared pods → Service
```

## Changes Made

### 1. Cloudflare Operator Deployment

**File:** `infra/gitops/applications/cloudflare-operator.yaml`

Deploys the [adyanth/cloudflare-operator](https://github.com/adyanth/cloudflare-operator) v0.13.1 to manage Cloudflare Tunnels via Kubernetes CRDs.

### 2. Cloudflare Tunnel Resources

**Directory:** `infra/gitops/resources/cloudflare-tunnel/`

| File | Purpose |
|------|---------|
| `cluster-tunnel.yaml` | Creates `cto-main` ClusterTunnel with 2 replicas |
| `webhook-binding.yaml` | Routes `github.public.5dlabs.ai/github/webhook` to `github-eventsource-svc:12000` |
| `kustomization.yaml` | Bundles resources for deployment |

**File:** `infra/gitops/applications/cloudflare-tunnel.yaml`

ArgoCD Application to deploy the tunnel resources.

### 3. Secrets Configuration

**File:** `infra/vault/secrets/infrastructure.yaml`

Added VaultStaticSecret for Cloudflare API credentials:
- Namespace: `cloudflare-operator-system`
- Secret name: `cloudflare-api-credentials`
- Vault path: `secret/cloudflare-operator`
- Required keys:
  - `CLOUDFLARE_API_TOKEN` - API token with Zone:DNS:Edit and Account:Cloudflare Tunnel:Edit
  - `CLOUDFLARE_ACCOUNT_ID` - Cloudflare account ID

### 4. Security: Removed Hardcoded Credentials

Fixed hardcoded database credentials in multiple files:

| File | Change |
|------|--------|
| `infra/charts/tools/values.yaml` | Replaced hardcoded `DATABASE_URI` with `secretRef` to `tools-postgres-secrets` |
| `infra/vault/secrets/tools.yaml` | Added `tools-postgres-secrets` VaultStaticSecret and documentation |
| `infra/charts/rustdocs-mcp/templates/deployment.yaml` | Changed hardcoded DB URL to `secretKeyRef` |
| `infra/charts/rustdocs-mcp/values.yaml` | Proper templated values without credentials |
| `infra/charts/rustdocs-mcp/templates/service.yaml` | Cleaned up hardcoded values |

### 5. Documentation

**File:** `docs/ngrok-implementation-reference.md`

Comprehensive reference document detailing the existing NGrok implementation including:
- Architecture overview
- Core components (Operator, Gateway API, CRDs)
- Webhook infrastructure
- Application publishing via Bolt
- Traffic policies and security
- DNS integration

**File:** `docs/cloudflare-tunnel-implementation-guide.md`

Implementation guide for Cloudflare Tunnel covering:
- Feature comparison matrix with NGrok
- Architecture mapping
- Deployment options
- Tunnel configuration
- Security configuration (WAF, headers)
- Migration plan

## Deployment Steps

### Prerequisites

1. **Populate Vault secrets:**
```bash
vault kv put secret/cloudflare-operator \
  CLOUDFLARE_API_TOKEN="<your-token>" \
  CLOUDFLARE_ACCOUNT_ID="<your-account-id>"
```

2. **API Token Permissions:**
   - Zone:DNS:Edit (for creating DNS records)
   - Account:Cloudflare Tunnel:Edit (for managing tunnels)

### Deployment Order

1. Merge this PR to main
2. ArgoCD will automatically deploy:
   - Cloudflare Operator to `cloudflare-operator-system`
   - ClusterTunnel `cto-main`
   - TunnelBinding `github-webhooks`
3. The operator will:
   - Create the tunnel in Cloudflare
   - Deploy 2 `cloudflared` replicas
   - Create DNS CNAME record for `github.public.5dlabs.ai`

### Verification

```bash
# Check operator status
kubectl get pods -n cloudflare-operator-system

# Check tunnel status
kubectl get clustertunnel cto-main

# Check tunnel binding
kubectl get tunnelbinding -n automation

# Test webhook endpoint
curl -I https://github.public.5dlabs.ai/github/webhook
```

## Future Work

- [ ] Update Bolt preview scripts to use TunnelBinding CRD
- [ ] Update Bolt production scripts for application publishing
- [ ] Update Bolt cleanup scripts to delete TunnelBindings
- [ ] Remove NGrok resources after validation
- [ ] Configure Cloudflare WAF rules for rate limiting
- [ ] Add Cloudflare Access for authentication (optional)

## Rollback

If issues occur, the NGrok configuration remains in place and can be re-enabled by:
1. Disabling the Cloudflare operator ArgoCD application
2. Re-enabling the NGrok gateway application
3. Updating DNS to point back to NGrok

## References

- [Cloudflare Operator GitHub](https://github.com/adyanth/cloudflare-operator)
- [Cloudflare Tunnel Documentation](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/)
- [NGrok Kubernetes Operator](https://ngrok.com/docs/k8s/)
