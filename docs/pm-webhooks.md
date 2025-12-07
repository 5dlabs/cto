# PM Service Webhooks

The PM (Project Management) service handles webhooks from various integrations including Linear, GitHub, and future providers like Asana and Jira.

## URL Structure

| Environment | Base URL | Example |
|-------------|----------|---------|
| **Development** | `pm-dev.5dlabs.ai` | `https://pm-dev.5dlabs.ai/webhooks/linear` |
| **Production** | `pm.5dlabs.ai` | `https://pm.5dlabs.ai/webhooks/linear` |

## Available Endpoints

| Path | Integration | Description |
|------|-------------|-------------|
| `/webhooks/linear` | Linear | Linear issue and agent session webhooks |
| `/webhooks/github` | GitHub | GitHub PR merge events for task completion |
| `/health` | - | Health check endpoint |

## Architecture

```
Internet                    CloudFlare Tunnel              Kubernetes Cluster
─────────────────────────────────────────────────────────────────────────────

                              ┌──────────────┐
pm-dev.5dlabs.ai ────────────►│  cloudflared │
       │                      │   (tunnel)   │
       │                      └──────┬───────┘
       │                             │
       ▼                             ▼
/webhooks/linear  ──────────────►  pm-svc:8081  ──────►  PM Server Pod
/webhooks/github                        │
/health                                 │
                                        ▼
                              ┌─────────────────────┐
                              │   Axum HTTP Router  │
                              ├─────────────────────┤
                              │ /webhooks/linear    │──► Linear handler
                              │ /webhooks/github    │──► GitHub handler
                              │ /health             │──► Health check
                              └─────────────────────┘
```

## Environment Configuration

### Development (Kind Cluster)

- **Tunnel Binding:** `infra/gitops/resources/cloudflare-tunnel/pm-binding.yaml`
- **Domain:** `pm-dev.5dlabs.ai`
- **Secrets:** `pm-secrets` in `cto` namespace
- **Linear App:** CTO Development (separate OAuth app)

### Production (Talos Cluster)

- **Domain:** `pm.5dlabs.ai`
- **Secrets:** Vault-managed (`infrastructure/pm/secrets`)
- **Linear App:** CTO Production (separate OAuth app)

## Secrets Required

| Secret Key | Description | Source |
|------------|-------------|--------|
| `LINEAR_WEBHOOK_SECRET` | HMAC signing secret for Linear webhooks | Linear App Settings |
| `LINEAR_OAUTH_TOKEN` | API token for Linear GraphQL API | Linear Personal API Keys |
| `LINEAR_OAUTH_CLIENT_ID` | OAuth application client ID | Linear OAuth App |
| `LINEAR_OAUTH_CLIENT_SECRET` | OAuth application client secret | Linear OAuth App |
| `GITHUB_TOKEN` | GitHub PAT for API access | GitHub Developer Settings |

## Linear App Setup

### Creating Separate Dev/Prod Apps

1. Go to **Linear Settings → API → OAuth Applications**
2. Create two applications:
   - **CTO Development** - webhook URL: `https://pm-dev.5dlabs.ai/webhooks/linear`
   - **CTO Production** - webhook URL: `https://pm.5dlabs.ai/webhooks/linear`
3. For each app, copy and securely store:
   - Client ID
   - Client Secret
   - Webhook Signing Secret

### Webhook Configuration

Each Linear OAuth app should have:
- **Webhook URL:** `https://pm-{dev|prod}.5dlabs.ai/webhooks/linear`
- **Resource Types:** Issue, Comment, AgentSessionEvent
- **All Public Teams:** Enabled (or specific team selection)

## Updating Secrets

### Development (Kind)

```bash
# Get current secret from 1Password
WEBHOOK_SECRET=$(op item get "Linear API Credentials - Dev" --fields "Webhook Secret" --reveal)

# Update K8s secret
kubectl create secret generic pm-secrets -n cto \
  --from-literal=LINEAR_WEBHOOK_SECRET="$WEBHOOK_SECRET" \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart PM service
kubectl rollout restart deployment/pm -n cto
```

### Production (Talos)

Secrets are managed via Vault at `infrastructure/pm/secrets`.

## Troubleshooting

### Webhook Signature Mismatch

If you see `Invalid webhook signature` in logs:

1. Check the secret length matches (should be ~51 chars for Linear):
   ```bash
   kubectl exec -n cto $(kubectl get pods -n cto -l app.kubernetes.io/name=pm -o name | head -1) -- env | grep LINEAR_WEBHOOK_SECRET
   ```

2. Verify the secret in Linear matches what's in K8s/Vault

3. Ensure the webhook URL in Linear points to the correct environment

### Testing Webhooks

```bash
# Check if endpoint is reachable
curl -s https://pm-dev.5dlabs.ai/health

# Test webhook endpoint (will return 401 without valid signature)
curl -X POST https://pm-dev.5dlabs.ai/webhooks/linear \
  -H "Content-Type: application/json" \
  -d '{"test": true}'
```

