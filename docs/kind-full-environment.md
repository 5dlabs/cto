# Kind Cluster: Full Local Development Environment

> ⚠️ **LOCAL TESTING ONLY** - This setup is for development/testing on Kind.
> Production uses HashiCorp Vault with Vault Secrets Operator.
> See `infra/vault/` for production secrets management.

## Overview

This document describes setting up a **complete** local development environment on Kind that mirrors production capabilities, including:

- ✅ CTO Controller (CodeRun reconciliation)
- ✅ Argo Workflows (workflow orchestration)
- 🔲 Argo Events (webhooks, sensors, event-driven triggers)
- 🔲 Tools Server (MCP tools for agents)
- 🔲 OpenMemory (memory/context management)
- 🔲 Healer (CI remediation workflows)
- 🔲 All CLI Images (Claude, Codex, Cursor, Factory, OpenCode)
- 🔲 All GitHub App Secrets (10 agents)
- 🔲 All Tool Secrets (Brave, Firecrawl, Context7, etc.)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Kind Cluster (cto-dev)                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ CTO         │  │ Argo        │  │ Argo        │                  │
│  │ Controller  │  │ Workflows   │  │ Events      │                  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                  │
│         │                │                │                          │
│         ▼                ▼                ▼                          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                     CodeRun Jobs                             │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │    │
│  │  │ Claude  │  │ Codex   │  │ Cursor  │  │ Factory │        │    │
│  │  │ CLI     │  │ CLI     │  │ CLI     │  │ CLI     │        │    │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘        │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ Tools       │  │ OpenMemory  │  │ Healer      │                  │
│  │ Server      │  │             │  │ Config      │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│  Secrets (from 1Password)                                            │
│  ├── cto-secrets (API keys)                                         │
│  ├── github-app-5dlabs-* (10 agents)                                │
│  ├── tools-*-secrets (MCP tool configs)                             │
│  └── github-webhook-secret                                          │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ Webhook Tunnel (ngrok/cloudflare)
                              ▼
                    ┌─────────────────┐
                    │ GitHub.com      │
                    │ Webhooks        │
                    └─────────────────┘
```

---

## Component Inventory

### Tier 1: Core (Already Running)

| Component | Namespace | Status | RAM |
|-----------|-----------|--------|-----|
| CTO Controller | cto | ✅ Running | 128-512Mi |
| Argo Workflows Controller | argo | ✅ Running | ~200Mi |
| Argo Server | argo | ✅ Running | ~100Mi |
| MinIO (artifacts) | argo | ✅ Running | ~100Mi |

### Tier 2: Event-Driven (To Install)

| Component | Namespace | Status | RAM |
|-----------|-----------|--------|-----|
| Argo Events Controller | argo-events | 🔲 Not installed | ~150Mi |
| EventBus (NATS) | argo-events | 🔲 Not installed | ~100Mi |
| GitHub EventSource | automation | 🔲 Not installed | ~50Mi |
| Sensors (various) | automation | 🔲 Not installed | ~50Mi each |

### Tier 3: Support Services (To Install)

| Component | Namespace | Status | RAM |
|-----------|-----------|--------|-----|
| Tools Server | cto | 🔲 Not installed | 512Mi-2Gi |
| OpenMemory | cto-system | 🔲 Not installed | 256-512Mi |
| Healer ConfigMap | cto | 🔲 Not installed | N/A |

### Tier 4: CLI Images (To Load)

| Image | Size | Status |
|-------|------|--------|
| ghcr.io/5dlabs/claude | ~12GB | 🔲 Not loaded |
| ghcr.io/5dlabs/codex | ~12GB | 🔲 Not loaded |
| ghcr.io/5dlabs/cursor | ~12GB | 🔲 Not loaded |
| ghcr.io/5dlabs/factory | ~12GB | 🔲 Not loaded |
| ghcr.io/5dlabs/opencode | ~12GB | 🔲 Not loaded |
| ghcr.io/5dlabs/tools | ~2GB | 🔲 Not loaded |
| ghcr.io/5dlabs/openmemory | ~1GB | 🔲 Not loaded |

---

## Secrets Inventory

### 1. API Keys (cto-secrets)

| Key | 1Password Item | Required For |
|-----|----------------|--------------|
| ANTHROPIC_API_KEY | Anthropic | Claude CLI |
| OPENAI_API_KEY | OpenAI | Codex CLI |
| XAI_API_KEY | xAI/Grok | OpenCode CLI |
| CURSOR_API_KEY | Cursor | Cursor CLI |
| FACTORY_API_KEY | Factory | Factory CLI |
| GEMINI_API_KEY | Google-Gemini | Gemini CLI |
| GOOGLE_API_KEY | Google | Various |
| CONTEXT7_API_KEY | Context7 | Context7 MCP |
| PERPLEXITY_API_KEY | Perplexity | Research |

### 2. GitHub App Secrets (per agent)

| Agent | 1Password Item | Fields |
|-------|----------------|--------|
| Morgan | GitHub-App-Morgan | app-id, client-id, private-key |
| Rex | GitHub-App-Rex | app-id, client-id, private-key |
| Blaze | GitHub-App-Blaze | app-id, client-id, private-key |
| Cleo | GitHub-App-Cleo | app-id, client-id, private-key |
| Tess | GitHub-App-Tess | app-id, client-id, private-key |
| Atlas | GitHub-App-Atlas | app-id, client-id, private-key |
| Bolt | GitHub-App-Bolt | app-id, client-id, private-key |
| Cipher | GitHub-App-Cipher | app-id, client-id, private-key |
| Stitch | GitHub-App-Stitch | app-id, client-id, private-key |
| Spark | GitHub-App-Spark | app-id, client-id, private-key |

### 3. Tools Server Secrets

| Secret Name | 1Password Item | Key |
|-------------|----------------|-----|
| tools-brave-search-secrets | Brave-Search | BRAVE_API_KEY |
| tools-context7-secrets | Context7 | CONTEXT7_API_KEY |
| tools-github-secrets | GitHub-PAT-Tools | GITHUB_PERSONAL_ACCESS_TOKEN |
| tools-firecrawl-secrets | Firecrawl | FIRECRAWL_API_KEY |
| tools-cloudflare-secrets | CloudFlare | CLOUDFLARE_API_TOKEN |
| tools-gemini-secrets | Google-Gemini | GEMINI_API_KEY |

### 4. Webhook Secrets

| Secret Name | 1Password Item | Key |
|-------------|----------------|-----|
| github-webhook-secret | GitHub-Webhook | secret |

---

## Installation Steps

### Phase 1: Secrets from 1Password (Local Testing Only)

> **Note:** This approach is for local Kind development only.
> Production uses Vault Secrets Operator - see `infra/vault/secrets/`

```bash
# 1. Sign in to 1Password
op signin

# 2. Verify you're on Kind (script will block production clusters)
kubectl config current-context  # Should show "kind-*"

# 3. Run the secrets migration
./scripts/kind-secrets-from-1password.sh --all

# 4. Verify secrets
kubectl get secrets -n cto | grep -E "(github-app|tools|cto-secrets)"
```

### Phase 2: Install Argo Events

```bash
# 1. Create namespace
kubectl create namespace argo-events
kubectl create namespace automation

# 2. Install Argo Events
kubectl apply -n argo-events -f https://raw.githubusercontent.com/argoproj/argo-events/stable/manifests/install.yaml

# 3. Install EventBus (NATS-based)
kubectl apply -n argo-events -f https://raw.githubusercontent.com/argoproj/argo-events/stable/examples/eventbus/native.yaml

# 4. Wait for components
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=controller-manager -n argo-events --timeout=120s

# 5. Verify
kubectl get pods -n argo-events
```

### Phase 3: Load CLI Images

```bash
# Load images (one at a time to manage resources)
./scripts/kind-load-cli-images.sh claude
./scripts/kind-load-cli-images.sh codex
./scripts/kind-load-cli-images.sh factory

# Or load all (requires ~60GB+ disk)
./scripts/kind-load-cli-images.sh all
```

### Phase 4: Deploy Tools Server

```bash
# 1. Load tools image
kind load docker-image ghcr.io/5dlabs/tools:latest --name cto-dev

# 2. Deploy via Helm
helm upgrade --install tools ./infra/charts/tools -n cto \
  -f ./infra/charts/tools/values.yaml \
  --set image.pullPolicy=Never
```

### Phase 5: Deploy OpenMemory

```bash
# 1. Load OpenMemory image
kind load docker-image ghcr.io/5dlabs/openmemory:latest --name cto-dev

# 2. Create namespace
kubectl create namespace cto-system

# 3. Deploy via Helm
helm upgrade --install openmemory ./infra/charts/openmemory -n cto-system \
  -f ./infra/charts/openmemory/values.yaml \
  --set image.pullPolicy=Never \
  --set database.storage.storageClassName=standard
```

### Phase 6: Configure Healer

```bash
# Apply healer configuration
kubectl apply -f ./infra/gitops/resources/healer/healer-config.yaml
```

### Phase 7: Setup Webhook Tunnel

For GitHub webhooks to reach your local Kind cluster, you need a tunnel:

#### Option A: ngrok (Simpler)

```bash
# Install ngrok
brew install ngrok

# Start tunnel to Argo Events port
kubectl port-forward -n automation svc/github-eventsource 12000:12000 &
ngrok http 12000

# Configure GitHub webhook with ngrok URL
# https://xxxxx.ngrok.io/github/webhook
```

#### Option B: Cloudflare Tunnel (Production-like)

```bash
# This requires Cloudflare account and tunnel configuration
# See: infra/gitops/resources/cloudflare-tunnel/
```

### Phase 8: Deploy Sensors & EventSources

```bash
# Create webhook secret
kubectl create secret generic github-webhook-secret \
  --namespace automation \
  --from-literal=secret="$(op read op://Personal/GitHub-Webhook/secret)"

# Apply EventSource
kubectl apply -f ./infra/gitops/resources/github-webhooks/eventsource.yaml

# Apply core sensors for Play workflow
kubectl apply -f ./infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml
kubectl apply -f ./infra/gitops/resources/github-webhooks/stage-aware-pr-merged-sensor.yaml
kubectl apply -f ./infra/gitops/resources/github-webhooks/stage-aware-cleo-approval-sensor.yaml
kubectl apply -f ./infra/gitops/resources/github-webhooks/stage-aware-tess-approval-sensor.yaml
```

---

## Resource Estimates

### Total RAM Required

| Component | RAM |
|-----------|-----|
| Kind system (K8s) | ~1.5GB |
| CTO Controller | 512Mi |
| Argo Workflows | 300Mi |
| Argo Events | 300Mi |
| Tools Server | 512Mi |
| OpenMemory | 256Mi |
| Sensors (5x) | 250Mi |
| **Subtotal (idle)** | **~3.5GB** |
| Agent Job (1x running) | ~1GB |
| **Total (with 1 agent)** | **~4.5GB** |

### Disk Space Required

| Component | Size |
|-----------|------|
| CLI Images (all 5) | ~60GB |
| Kind node storage | ~10GB |
| PVCs (workspaces) | ~5GB |
| **Total** | **~75GB** |

---

## Quick Validation Commands

```bash
# Check all components
kubectl get pods -n cto
kubectl get pods -n argo
kubectl get pods -n argo-events
kubectl get pods -n automation
kubectl get pods -n cto-system

# Check secrets
kubectl get secrets -n cto | wc -l
# Expected: 15+ secrets

# Check images in Kind
./scripts/kind-load-cli-images.sh --status

# Test CodeRun
kubectl apply -f - <<EOF
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-full-$(date +%s)
  namespace: cto
spec:
  service: cto
  repositoryUrl: "https://github.com/5dlabs/cto"
  docsRepositoryUrl: "https://github.com/5dlabs/cto"
  workingDirectory: "."
  model: "claude-sonnet-4-5-20250929"
  githubApp: "5DLabs-Rex"
  cliConfig:
    cliType: claude
    model: "claude-sonnet-4-5-20250929"
EOF

# Watch execution
kubectl get coderuns -n cto -w
```

---

## Master Setup Script

A single script to set up everything:

```bash
./scripts/kind-full-setup.sh
```

This script will:
1. ✅ Verify Kind cluster is running
2. ✅ Migrate all secrets from 1Password
3. ✅ Install Argo Events
4. ✅ Load CLI images
5. ✅ Deploy Tools Server
6. ✅ Deploy OpenMemory
7. ✅ Configure Healer
8. ✅ Deploy sensors (without webhook tunnel)

---

## Troubleshooting

### Pod Stuck in Pending
```bash
kubectl describe pod <pod-name> -n <namespace>
# Check for: ImagePullBackOff, Insufficient resources, Affinity issues
```

### Image Not Found
```bash
# Verify image is in Kind
docker exec cto-dev-control-plane crictl images | grep <image-name>

# Reload if missing
kind load docker-image <image> --name cto-dev
```

### Secrets Missing
```bash
# Check 1Password connectivity
op whoami

# Re-run secret migration
./scripts/kind-secrets-from-1password.sh --all
```

### Webhook Not Triggering
```bash
# Check EventSource logs
kubectl logs -n automation -l eventsource-name=github

# Verify tunnel is running
curl -s localhost:12000/health
```

