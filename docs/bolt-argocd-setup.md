# Bolt ArgoCD Setup Guide

**Prerequisites for Bolt Dual-Mode System**

---

## 📋 Required ArgoCD Setup

Before Bolt can deploy applications, ArgoCD needs the following configuration:

### 1. Platform Project Updates ✅

**File:** `infra/gitops/projects/platform-project.yaml`

**Add to sourceRepos:**
```yaml
sourceRepos:
  - 'https://github.com/5dlabs/cto-apps'  # ← ADD THIS
```

**Add to destinations:** (wildcard for preview/production namespaces)
```yaml
destinations:
  - namespace: 'agent-platform-preview-*'  # ← ADD THIS (preview namespaces)
    server: https://kubernetes.default.svc
  - namespace: 'agent-platform-prod-*'     # ← ADD THIS (production namespaces)
    server: https://kubernetes.default.svc
```

### 2. Deploy CTO-Apps App-of-Apps

**One-time deployment:**
```bash
kubectl apply -f - <<EOF
---
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: cto-apps
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: platform
  source:
    repoURL: https://github.com/5dlabs/cto-apps.git
    targetRevision: main
    path: .
    directory:
      recurse: true
      exclude: |
        templates/*
        README.md
        app-of-apps.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=false
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
EOF
```

### 3. Calendar EventSource (for Bolt-Monitor)

**File:** Already created in `infra/gitops/resources/sensors/bolt-monitor-sensor.yaml`

**Deploy manually if needed:**
```bash
kubectl apply -f infra/gitops/resources/sensors/bolt-monitor-sensor.yaml
```

---

## 🚀 Quick Setup Script

Run this all at once:

```bash
#!/bin/bash
set -e

echo "===== Bolt ArgoCD Setup ====="

# Step 1: Update platform project (apply from this repo)
echo "Step 1: Updating platform project..."
kubectl apply -f infra/gitops/projects/platform-project.yaml

# Step 2: Deploy cto-apps app-of-apps
echo "Step 2: Deploying cto-apps app-of-apps..."
kubectl apply -f - <<'EOF'
---
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: cto-apps
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: platform
  source:
    repoURL: https://github.com/5dlabs/cto-apps.git
    targetRevision: main
    path: .
    directory:
      recurse: true
      exclude: |
        templates/*
        README.md
        app-of-apps.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=false
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
EOF

# Step 3: Wait for cto-apps to sync
echo "Step 3: Waiting for cto-apps to sync..."
sleep 5
kubectl wait --for=condition=Synced application/cto-apps -n argocd --timeout=60s || true

# Step 4: Verify setup
echo "Step 4: Verifying setup..."
kubectl get application cto-apps -n argocd
kubectl get appproject platform -n argocd

echo ""
echo "===== Setup Complete ====="
echo "✅ Platform project updated"
echo "✅ cto-apps app-of-apps deployed"
echo ""
echo "Next: Bolt sensors will automatically create applications in cto-apps repo"
echo "      ArgoCD will sync them within 30 seconds"
```

---

## 🔍 Verification Steps

### Check Platform Project
```bash
# Verify cto-apps repo is whitelisted
kubectl get appproject platform -n argocd -o yaml | grep cto-apps

# Verify preview/prod namespaces are whitelisted
kubectl get appproject platform -n argocd -o yaml | grep "agent-platform-"
```

### Check CTO-Apps Application
```bash
# Check if app-of-apps is deployed
kubectl get application cto-apps -n argocd

# Check sync status
kubectl get application cto-apps -n argocd -o jsonpath='{.status.sync.status}'

# Check health status
kubectl get application cto-apps -n argocd -o jsonpath='{.status.health.status}'
```

### Check Sensors
```bash
# Verify Bolt sensors are deployed
kubectl get sensors -n argo | grep bolt

# Expected output:
# bolt-monitor-daemon
# bolt-preview-deployment
# bolt-preview-cleanup
# bolt-production-deployment
```

---

## 🛠️ Troubleshooting

### App-of-Apps Not Syncing

```bash
# Check application status
kubectl describe application cto-apps -n argocd

# Force refresh
kubectl patch application cto-apps -n argocd \
  --type merge -p '{"metadata":{"annotations":{"argocd.argoproj.io/refresh":"hard"}}}'

# Check ArgoCD logs
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-application-controller
```

### Repository Access Issues

```bash
# Verify ArgoCD can access cto-apps repo
kubectl get secrets -n argocd | grep repo

# Test repository connection
kubectl exec -n argocd -it deploy/argocd-server -- \
  argocd repo list

# Add repository if missing (public repos don't need this)
kubectl exec -n argocd -it deploy/argocd-server -- \
  argocd repo add https://github.com/5dlabs/cto-apps.git
```

### Namespace Permissions

If applications fail to deploy to preview/production namespaces:

```bash
# Check if namespace pattern is in platform project
kubectl get appproject platform -n argocd -o yaml

# The destinations should include wildcards:
# - namespace: 'agent-platform-preview-*'
# - namespace: 'agent-platform-prod-*'
```

---

## 📊 Monitoring

### Watch Applications Being Created

```bash
# Watch all applications (will show preview/prod as they're created)
kubectl get applications -n argocd -w

# Watch Bolt-managed applications only
kubectl get applications -n argocd -l managed-by=bolt -w
```

### Watch Git Activity

```bash
# Clone cto-apps and watch for commits
git clone https://github.com/5dlabs/cto-apps.git
cd cto-apps
watch -n 5 git pull
```

### Watch Bolt CodeRuns

```bash
# Watch Bolt agents working
kubectl get coderuns -n agent-platform -l agent=bolt -w
```

---

## 🎯 What Happens Next

Once this setup is complete:

1. **PR Opened** → Bolt-Preview sensor triggers
2. **Bolt clones cto-apps** → Generates preview YAML
3. **Commits to cto-apps/main** → `preview/task-{id}-preview.yaml`
4. **ArgoCD detects change** (within 30s)
5. **ArgoCD creates Application** → Deploys to preview namespace
6. **Bolt waits for sync** → Verifies health
7. **Bolt posts preview URL** → PR comment

Same flow for production when PR is merged!

---

## ✅ Checklist

Before testing Bolt:

- [ ] Platform project updated with cto-apps repo
- [ ] Platform project allows preview/prod namespaces (wildcards)
- [ ] CTO-apps app-of-apps deployed
- [ ] CTO-apps application shows Synced + Healthy
- [ ] Bolt sensors are deployed and running
- [ ] Calendar EventSource deployed (for monitor)

Run this to check all at once:

```bash
kubectl get appproject platform -n argocd -o yaml | grep -A2 "cto-apps\|preview-\|prod-" && \
kubectl get application cto-apps -n argocd && \
kubectl get sensors -n argo | grep bolt && \
echo "✅ All prerequisites met!"
```

---

## 🔐 Security Notes

### Repository Access
- `cto-apps` is a **public** repository (no credentials needed)
- Bolt pushes using GitHub CLI (`gh`) with agent credentials
- Application repositories can be public or private (ArgoCD handles auth)

### Namespace Isolation
- Preview namespaces: `agent-platform-preview-task-{id}`
- Production namespaces: `agent-platform-prod-task-{id}`
- Each task gets isolated namespace
- Resource quotas applied automatically

### RBAC
- Bolt uses `agent-workflow-sa` service account
- Limited to creating/deleting Applications in argocd namespace
- ArgoCD enforces platform project restrictions

---

Ready to test! 🚀

