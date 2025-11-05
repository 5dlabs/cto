# Bolt GitOps Integration - Complete

**Date:** November 5, 2025  
**Status:** ✅ Complete - GitOps Pattern Implemented  
**Repository:** https://github.com/5dlabs/cto-apps

---

## 🎯 What Changed

Bolt now uses a **full GitOps approach** instead of directly creating Kubernetes resources. All ArgoCD Applications are managed as YAML files in the `cto-apps` repository.

### Old Approach ❌
```bash
Bolt → kubectl apply -f - <<EOF
  apiVersion: argoproj.io/v1alpha1
  kind: Application
  ...
EOF
```

### New Approach ✅
```bash
Bolt → Clone cto-apps
     → Generate YAML from template
     → Git commit & push
     → ArgoCD auto-syncs
```

---

## 📦 GitOps Repository Structure

**Repository:** `5dlabs/cto-apps`

```
cto-apps/
├── README.md                      # Complete documentation
├── app-of-apps.yaml              # Self-documenting
├── preview/                       # Preview deployments
│   ├── task-1-preview.yaml
│   ├── task-2-preview.yaml
│   └── ...
├── production/                    # Production deployments
│   ├── task-1-prod.yaml
│   ├── task-2-prod.yaml
│   └── ...
└── templates/
    ├── preview-app.yaml.template     # Template for previews
    └── production-app.yaml.template  # Template for production
```

---

## 🔄 How It Works

### 1. Deploy the App-of-Apps (One-Time)

```bash
kubectl apply -f /tmp/deploy-cto-apps.yaml
```

This creates an ArgoCD Application that:
- Watches `https://github.com/5dlabs/cto-apps`
- Auto-syncs any changes within 30 seconds
- Creates/updates/deletes applications based on YAML files

### 2. Bolt Creates Preview Deployment

```bash
# When PR opened
Bolt-Preview:
  1. Clones cto-apps repository
  2. Generates preview/task-{id}-preview.yaml from template
  3. Commits: "Add preview deployment for task-{id}"
  4. Pushes to main branch
  5. ArgoCD syncs automatically
```

### 3. Bolt Creates Production Deployment

```bash
# When PR merged with ready-for-production label
Bolt-Production:
  1. Clones cto-apps repository
  2. Removes preview/task-{id}-preview.yaml
  3. Generates production/task-{id}-prod.yaml from template
  4. Commits: "Add production deployment for task-{id}"
  5. Pushes to main branch
  6. ArgoCD syncs automatically
```

### 4. Bolt Cleans Up Preview

```bash
# When PR closed
Bolt-Cleanup:
  1. Clones cto-apps repository
  2. Deletes preview/task-{id}-preview.yaml
  3. Commits: "Remove preview deployment for task-{id}"
  4. Pushes to main branch
  5. ArgoCD prunes the application
```

---

## ✅ Multi-CLI Support

### Bolt Works With ALL CLIs

The Bolt scripts are located in `code/integration/` directory, making them **CLI-agnostic**.

**All CLIs Supported:**
- ✅ **Claude** (primary)
- ✅ **Codex**
- ✅ **OpenCode**
- ✅ **Cursor**
- ✅ **Factory**

**Why?** The integration scripts use **standard tools** available in all environments:
- `kubectl` - Kubernetes CLI
- `gh` - GitHub CLI
- `git` - Git CLI
- `sed` - Stream editor
- `jq` - JSON processor
- `curl` - HTTP client

**No CLI-specific dependencies!**

### Controller Mapping

The controller automatically maps Bolt (`5DLabs-Bolt`) to the integration scripts:

```rust
// In controller/src/templates.rs
"5DLabs-Bolt" => {
    // All CLIs use the same integration scripts
    container_script: "code/integration/container-bolt-*.sh.hbs"
    system_prompt: "agents/bolt-*-system-prompt.md.hbs"
}
```

The `BOLT_MODE` environment variable determines which script runs:
- `BOLT_MODE=monitor` → `container-bolt-monitor.sh.hbs`
- `BOLT_MODE=preview` → `container-bolt-preview.sh.hbs`
- `BOLT_MODE=cleanup` → `container-bolt-cleanup.sh.hbs`
- `BOLT_MODE=production` → `container-bolt.sh.hbs`

---

## 📋 Benefits of GitOps Approach

### For Operations
- ✅ **Full audit trail** - Every deployment has a Git commit
- ✅ **Easy rollback** - `git revert` to undo deployments
- ✅ **Declarative** - Desired state in Git, not imperative kubectl
- ✅ **GitOps compliance** - Industry best practice

### For Development
- ✅ **Transparent** - See all deployments in one repo
- ✅ **Reviewable** - Can PR review deployments if needed
- ✅ **Debuggable** - Git history shows what changed when
- ✅ **Reproducible** - Can recreate exact state from Git

### For Platform
- ✅ **Separation of concerns** - Apps separate from platform
- ✅ **ArgoCD native** - Leverages ArgoCD's strengths
- ✅ **Auto-sync** - Changes propagate automatically
- ✅ **Self-healing** - ArgoCD maintains desired state

---

## 🔧 Configuration

### GitHub Authentication

Bolt needs to push to `cto-apps` repository:

```bash
# GitHub credentials must be configured
# Either via gh CLI (automatically authenticated)
# Or via git credential helper
```

The scripts use `gh` CLI which is pre-configured in the agent containers.

### ArgoCD Sync Interval

Default: 30 seconds (ArgoCD polls Git every 30s)

Can be adjusted in the app-of-apps manifest:
```yaml
spec:
  source:
    repoURL: https://github.com/5dlabs/cto-apps.git
    targetRevision: main
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

---

## 🎨 Template System

### Preview Template

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: {{APP_NAME}}              # task-{id}-preview
  labels:
    task-id: "{{TASK_ID}}"        # {id}
    environment: preview
    managed-by: bolt
spec:
  source:
    repoURL: {{REPO_URL}}         # Application repo
    targetRevision: {{BRANCH}}    # PR branch (auto-updates!)
    path: {{ARGOCD_PATH}}         # helm/ or k8s/
  destination:
    namespace: {{NAMESPACE}}      # agent-platform-preview-task-{id}
```

### Production Template

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: {{APP_NAME}}              # task-{id}-prod
  labels:
    task-id: "{{TASK_ID}}"        # {id}
    environment: production
    managed-by: bolt
spec:
  source:
    repoURL: {{REPO_URL}}         # Application repo
    targetRevision: main          # Always main for production
    path: {{ARGOCD_PATH}}         # helm/ or k8s/
  destination:
    namespace: {{NAMESPACE}}      # agent-platform-prod-task-{id}
```

---

## 🚀 Deployment Instructions

### Initial Setup (One-Time)

1. **Deploy the App-of-Apps:**
```bash
kubectl apply -f /tmp/deploy-cto-apps.yaml
```

2. **Verify it's running:**
```bash
kubectl get application cto-apps -n argocd
```

3. **Watch for sync:**
```bash
kubectl get application cto-apps -n argocd -w
```

### Verification

Check ArgoCD UI:
```bash
# Get password
kubectl -n argocd get secret argocd-initial-admin-secret \
  -o jsonpath="{.data.password}" | base64 -d

# Port forward
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Open browser
open https://localhost:8080
```

Look for the `cto-apps` application - it should be green (Synced + Healthy).

---

## 🧪 Testing

### Test Preview Deployment

1. Create a PR in an application repo
2. Verify Bolt-Preview sensor triggers
3. Check `cto-apps` repository for new file:
   - `preview/task-{id}-preview.yaml`
4. Verify ArgoCD creates the application
5. Verify preview URL posted to PR

### Test Production Deployment

1. Merge PR with `ready-for-production` label
2. Verify Bolt-Production sensor triggers
3. Check `cto-apps` repository:
   - `preview/task-{id}-preview.yaml` deleted
   - `production/task-{id}-prod.yaml` created
4. Verify ArgoCD creates production application
5. Verify production URL posted to PR

### Test Cleanup

1. Close a PR (without merging)
2. Verify Bolt-Cleanup sensor triggers
3. Check `cto-apps` repository:
   - `preview/task-{id}-preview.yaml` deleted
4. Verify ArgoCD deletes the application
5. Verify namespace is removed

---

## 🛠️ Troubleshooting

### Application Not Syncing

```bash
# Check app-of-apps status
kubectl get application cto-apps -n argocd -o yaml

# Check for sync errors
kubectl describe application cto-apps -n argocd

# Force sync
kubectl patch application cto-apps -n argocd \
  --type merge -p '{"metadata":{"annotations":{"argocd.argoproj.io/refresh":"hard"}}}'
```

### Git Push Failures

Bolt scripts include fallback logic:
- If Git push fails, falls back to direct `kubectl apply`
- Check Bolt CodeRun logs for details:
```bash
kubectl logs -n agent-platform -l agent=bolt --tail=100
```

### Template Variables Not Replaced

Check the generated YAML in Bolt logs:
```bash
kubectl logs -n agent-platform <coderun-bolt-pod> | grep "Generated:"
```

Variables should be replaced:
- `{{APP_NAME}}` → `task-5-preview`
- `{{TASK_ID}}` → `5`
- `{{REPO_URL}}` → `https://github.com/org/repo.git`

---

## 📊 Monitoring

### Git Activity

Watch `cto-apps` repository for commits:
```bash
# Clone and watch
git clone https://github.com/5dlabs/cto-apps.git
cd cto-apps
watch -n 5 git pull
```

### ArgoCD Activity

Watch applications being created:
```bash
kubectl get applications -n argocd -l managed-by=bolt -w
```

### Bolt Activity

Watch Bolt CodeRuns:
```bash
kubectl get coderuns -n agent-platform -l agent=bolt -w
```

---

## ✨ Summary

The Bolt dual-mode system now uses **full GitOps** for all deployments:

- ✅ **Standalone `cto-apps` repository** - Decoupled from CTO platform
- ✅ **Template-based generation** - Consistent application manifests
- ✅ **Git as source of truth** - Full audit trail and rollback
- ✅ **ArgoCD auto-sync** - Changes propagate within 30 seconds
- ✅ **Multi-CLI support** - Works with Claude, Codex, OpenCode, Cursor, Factory
- ✅ **Fallback mechanisms** - Graceful degradation if Git fails

**Let's fucking go! 🚀**

