# Kind Cluster: Minimum Requirements for Play Workflow E2E Testing

## Executive Summary

This document evaluates the minimum requirements to run a full Play workflow end-to-end on the local Kind cluster (`kind-cto-dev`). The goal is to enable local testing without the overhead of full production infrastructure while maintaining functional parity for core workflows.

## Current State Assessment

### ✅ Already Running

| Component | Status | Notes |
|-----------|--------|-------|
| CTO Controller | ✅ Running | v0.12.38, 128Mi memory |
| Argo Workflows | ✅ Running | Controller + Server + MinIO |
| CodeRun CRD | ✅ Installed | `coderuns.agents.platform` |
| DocsRun CRD | ✅ Installed | `docsruns.agents.platform` |
| WorkflowTemplates | ✅ Deployed | play-workflow-template, coderun-template, etc. |
| Storage Class | ✅ Available | `standard` (local-path) |
| API Keys | ✅ Complete | 10 keys in `cto-secrets` |

### ❌ Missing Components

| Component | Required For | Priority |
|-----------|-------------|----------|
| Argo Events | Event-driven triggers, webhooks | Medium (can skip for manual testing) |
| GitHub App Secrets | Agent authentication | **High** |
| CLI Images | Agent execution | **Critical** |

---

## Resource Constraints

**Available Resources (Kind Node):**
- CPU: 12 cores allocatable
- Memory: ~8GB allocatable
- Storage: ~470GB ephemeral

**Current Usage:**
- CPU: ~8% (1050m requests)
- Memory: ~5% (418Mi requests)
- Pods: 14/110

**CLI Image Sizes (Critical Consideration):**
| Image | Size | Notes |
|-------|------|-------|
| `ghcr.io/5dlabs/claude:latest` | 12.1 GB | Primary agent runtime |
| `ghcr.io/5dlabs/codex:latest` | 12.4 GB | OpenAI Codex CLI |
| `ghcr.io/5dlabs/factory:latest` | 11.6 GB | Factory CLI |
| `ghcr.io/5dlabs/cursor:latest` | ~12 GB | Not pulled yet |
| `ghcr.io/5dlabs/opencode:latest` | ~12 GB | Not pulled yet |

⚠️ **Warning:** Loading all CLI images into Kind would require ~60GB of node storage and significant load time. **Recommend loading only 1-2 images for initial testing.**

---

## Secrets Management Options

### Option 1: Manual Kubernetes Secrets (Current Approach)
**Pros:**
- Already working for API keys
- No additional infrastructure
- Simple and direct

**Cons:**
- Manual creation/rotation
- No audit trail
- Secrets in plain YAML files

**Implementation:** Already in use for `cto-secrets` and 2 GitHub Apps (Morgan, Rex)

### Option 2: 1Password CLI Integration (Recommended for Local Dev)
**Pros:**
- Already installed (`op` v2.32.0)
- Personal vault available
- Secure storage outside repo
- Easy scripting with `op read`

**Cons:**
- Requires `op signin` before use
- Not K8s-native (needs shim script)
- Single-user focused

**Implementation:**
```bash
# Create a shim script: scripts/kind-secrets-from-1password.sh
#!/bin/bash
set -euo pipefail

# Fetch and create GitHub App secret from 1Password
create_github_app_secret() {
    local agent=$1
    local vault=${2:-"Personal"}
    
    kubectl create secret generic "github-app-5dlabs-${agent}" \
        --namespace cto \
        --from-literal="app-id=$(op read "op://${vault}/GitHub-App-${agent}/app-id")" \
        --from-literal="client-id=$(op read "op://${vault}/GitHub-App-${agent}/client-id")" \
        --from-literal="private-key=$(op read "op://${vault}/GitHub-App-${agent}/private-key")" \
        --dry-run=client -o yaml | kubectl apply -f -
}

# Create API keys secret from 1Password
create_api_keys_secret() {
    local vault=${1:-"Personal"}
    
    kubectl create secret generic cto-secrets \
        --namespace cto \
        --from-literal="ANTHROPIC_API_KEY=$(op read "op://${vault}/Anthropic/api-key")" \
        --from-literal="OPENAI_API_KEY=$(op read "op://${vault}/OpenAI/api-key")" \
        --from-literal="XAI_API_KEY=$(op read "op://${vault}/xAI/api-key")" \
        # ... other keys
        --dry-run=client -o yaml | kubectl apply -f -
}
```

### Option 3: Vault Secrets Operator
**Pros:**
- Production parity
- Automatic rotation
- Audit logging

**Cons:**
- Heavy for local dev (~500MB+ RAM)
- Requires Vault server
- Complex setup

**Recommendation:** Skip for Kind, use in production only.

### ✅ Recommended Approach: Hybrid

1. **API Keys:** Keep using existing `cto-secrets` (already populated)
2. **GitHub Apps:** Create script to populate from 1Password on demand
3. **Future:** Consider external-secrets-operator if team grows

---

## Required Secrets for Play Workflow

### Minimum for Single-Agent Test
```
cto-secrets (API keys)        ✅ Already exists
github-app-5dlabs-rex         ✅ Already exists  
```

### Full Play Workflow (Rex → Cleo → Tess → Atlas)
```
cto-secrets (API keys)        ✅ Already exists
github-app-5dlabs-morgan      ✅ Already exists
github-app-5dlabs-rex         ✅ Already exists
github-app-5dlabs-blaze       ❌ Missing
github-app-5dlabs-cleo        ❌ Missing
github-app-5dlabs-tess        ❌ Missing
github-app-5dlabs-atlas       ❌ Missing
github-app-5dlabs-bolt        ❌ Missing (optional for deployment stage)
```

Each GitHub App secret needs:
- `app-id`: GitHub App ID
- `client-id`: OAuth client ID
- `private-key`: PEM-encoded private key

---

## Component Breakdown

### Tier 1: Essential (Must Have)

| Component | Purpose | RAM | Install Command |
|-----------|---------|-----|-----------------|
| Controller | CR reconciliation | 128-512Mi | Already running |
| Argo Workflows | Workflow orchestration | ~200Mi | Already running |
| 1x CLI Image | Agent execution | ~1GB runtime | `kind load docker-image ghcr.io/5dlabs/claude:latest --name cto-dev` |
| Secrets | Authentication | N/A | See scripts below |

### Tier 2: Recommended (For Full Play)

| Component | Purpose | RAM | Notes |
|-----------|---------|-----|-------|
| 2nd CLI Image | Multiple CLI testing | ~1GB runtime | Load codex OR factory |
| All GitHub App Secrets | Full agent chain | N/A | 6 more secrets needed |

### Tier 3: Optional (Can Skip for Local)

| Component | Purpose | Why Skip |
|-----------|---------|----------|
| Argo Events | Event-driven triggers | Can trigger workflows manually via `argo submit` |
| Vault | Secrets management | 1Password CLI sufficient |
| Observability Stack | Monitoring/alerting | Not needed for functional testing |
| ArgoCD | GitOps deployment | Direct kubectl/helm is fine |
| Notifications | Discord/Slack alerts | Not needed locally |

---

## Implementation Plan

### Phase 1: Minimal Play Test (Single Agent)

**Time Estimate:** 15-30 minutes

```bash
# 1. Load Claude CLI image into Kind (takes ~5-10 min)
kind load docker-image ghcr.io/5dlabs/claude:latest --name cto-dev

# 2. Verify Rex secrets exist
kubectl get secret github-app-5dlabs-rex -n cto

# 3. Test single CodeRun
kubectl apply -f - <<EOF
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-rex-run
  namespace: cto
spec:
  service: test-service
  repositoryUrl: "https://github.com/5dlabs/cto"
  docsRepositoryUrl: "https://github.com/5dlabs/cto"
  workingDirectory: "."
  model: "claude-sonnet-4-5-20250929"
  githubApp: "5DLabs-Rex"
  taskId: 1
EOF

# 4. Monitor execution
kubectl get coderuns -n cto -w
kubectl logs -f -l job-name=test-rex-run -n cto
```

### Phase 2: Full Play Workflow

**Time Estimate:** 1-2 hours (mostly image loading)

```bash
# 1. Create missing GitHub App secrets
./scripts/kind-secrets-from-1password.sh blaze cleo tess atlas

# 2. Load additional CLI image(s) as needed
kind load docker-image ghcr.io/5dlabs/codex:latest --name cto-dev

# 3. Trigger play workflow
argo submit -n cto \
  --from workflowtemplate/play-workflow-template \
  -p task-id=1 \
  -p repository=5dlabs/test-repo \
  -p service=test-service \
  -p docs-repository=5dlabs/test-repo \
  -p docs-project-directory=docs \
  -p implementation-agent=rex \
  -p cli=claude \
  --watch
```

---

## Scripts to Create

### 1. `scripts/kind-secrets-from-1password.sh`

Populates K8s secrets from 1Password vault.

### 2. `scripts/kind-load-cli-images.sh`

Selectively loads CLI images based on which agents you're testing:

```bash
#!/bin/bash
# Usage: ./kind-load-cli-images.sh [claude|codex|factory|all]

CLI=${1:-claude}
CLUSTER=${2:-cto-dev}

case $CLI in
  claude)
    echo "Loading Claude CLI (~12GB)..."
    kind load docker-image ghcr.io/5dlabs/claude:latest --name $CLUSTER
    ;;
  codex)
    echo "Loading Codex CLI (~12GB)..."
    kind load docker-image ghcr.io/5dlabs/codex:latest --name $CLUSTER
    ;;
  factory)
    echo "Loading Factory CLI (~12GB)..."
    kind load docker-image ghcr.io/5dlabs/factory:latest --name $CLUSTER
    ;;
  all)
    echo "Loading ALL CLI images (~36GB total)..."
    kind load docker-image ghcr.io/5dlabs/claude:latest --name $CLUSTER
    kind load docker-image ghcr.io/5dlabs/codex:latest --name $CLUSTER
    kind load docker-image ghcr.io/5dlabs/factory:latest --name $CLUSTER
    ;;
  *)
    echo "Unknown CLI: $CLI"
    exit 1
    ;;
esac
```

### 3. `scripts/kind-test-coderun.sh`

Quick CodeRun test script.

---

## Argo Events: Do We Need It?

### For Local Testing: **No**

Argo Events provides:
- GitHub webhook ingestion
- Event-driven workflow triggers
- PR status sensors

For local testing, you can:
1. Trigger workflows manually via `argo submit`
2. Use `kubectl apply` for CodeRun CRs
3. Skip PR integration testing

### For Full E2E with GitHub Integration: **Yes**

If you need to test:
- Automatic PR creation workflows
- CI failure remediation sensors
- PR review triggers

Then install Argo Events:
```bash
kubectl create namespace argo-events
kubectl apply -n argo-events -f https://raw.githubusercontent.com/argoproj/argo-events/stable/manifests/install.yaml
kubectl apply -n argo-events -f https://raw.githubusercontent.com/argoproj/argo-events/stable/examples/eventbus/native.yaml
```

**Resource Impact:** ~200-400MB additional RAM

---

## Decision Matrix

| Scenario | CLI Images | Secrets | Argo Events | Estimated RAM |
|----------|-----------|---------|-------------|---------------|
| Smoke Test | Claude only | Rex only | No | ~2GB |
| Single Agent E2E | Claude only | Rex + API | No | ~2GB |
| Full Play (no events) | Claude + Codex | All agents | No | ~3GB |
| Full Play (with events) | Claude + Codex | All agents | Yes | ~3.5GB |

---

## Recommendation

### For Initial Testing (Today)

1. ✅ Use existing controller deployment
2. ✅ Load only Claude CLI image (~12GB, ~10 min)
3. ✅ Use existing Rex secrets
4. ✅ Skip Argo Events
5. ✅ Test single CodeRun first

### For Full Play Testing (This Week)

1. Create 1Password shim script for GitHub App secrets
2. Load Claude + one other CLI image
3. Populate remaining agent secrets (Cleo, Tess, Atlas)
4. Run full play workflow manually via `argo submit`

### Not Needed for Local

- ❌ Vault Secrets Operator
- ❌ Full observability stack
- ❌ ArgoCD
- ❌ Notifications
- ❌ All 5 CLI images (just load what you need)

---

## Quick Start Commands

```bash
# 1. Load Claude image (one-time, ~10 min)
kind load docker-image ghcr.io/5dlabs/claude:latest --name cto-dev

# 2. Verify cluster state
kubectl get pods -n cto
kubectl get pods -n argo
kubectl get coderuns -n cto

# 3. Check secrets
kubectl get secrets -n cto | grep github-app

# 4. Test CodeRun (uses Rex + Claude)
cat <<EOF | kubectl apply -f -
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: kind-test-$(date +%s)
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

# 5. Watch execution
kubectl get coderuns -n cto -w
```



