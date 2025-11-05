# Bolt Dual-Mode Architecture
**Date:** November 5, 2025  
**Status:** 🚧 Design & Implementation In Progress

---

## 🎯 Overview

Bolt operates in **THREE distinct modes** to provide comprehensive deployment monitoring and management:

1. **Bolt-Monitor** (Daemon) - Continuous health monitoring
2. **Bolt-Preview** (Event-Driven) - Preview deployments for PRs
3. **Bolt-Production** (Event-Driven) - Production deployments after merge

---

## 🏗️ Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Bolt Dual-Mode System                    │
└─────────────────────────────────────────────────────────────┘

┌──────────────────────┐
│   Bolt-Monitor       │  ◄── Always Running
│   (Daemon Mode)      │      
└──────────────────────┘
         │
         ├─► Watches ALL ArgoCD apps
         ├─► Posts health updates to PRs
         ├─► Monitors sync status
         └─► Alerts on failures

┌──────────────────────┐
│   Bolt-Preview       │  ◄── PR Created/Updated
│   (Event-Driven)     │
└──────────────────────┘
         │
         ├─► Creates preview ArgoCD app
         ├─► Sets up preview ngrok tunnel
         ├─► Posts preview URL to PR
         └─► Available for Tess testing

┌──────────────────────┐
│   Bolt-Production    │  ◄── PR Merged (ready-for-production)
│   (Event-Driven)     │
└──────────────────────┘
         │
         ├─► Creates production ArgoCD app
         ├─► Sets up production ngrok tunnel
         ├─► Posts production URL to PR
         └─► Cleans up preview deployment
```

---

## 📋 Detailed Mode Specifications

### Mode 1: Bolt-Monitor (Daemon)

**Purpose:** Continuous visibility into ALL deployments

**Trigger:** Runs continuously (not event-driven)

**Responsibilities:**
- Query ArgoCD for all applications every 30 seconds
- Check application sync status (Synced, OutOfSync, Unknown)
- Check application health status (Healthy, Progressing, Degraded, Missing)
- Monitor pod status for all applications
- Analyze recent logs for errors/warnings
- Post progressive status updates to relevant PRs

**Implementation:**
- Sensor: `bolt-monitor-sensor.yaml` (watches ArgoCD app status changes)
- Script: `container-bolt-monitor.sh.hbs`
- Prompt: `agents/bolt-monitor-system-prompt.md.hbs`

**Comment Format:**
```markdown
## 📊 Bolt Monitor: Deployment Health

### Preview Deployments
- **task-5-preview** ✅ Healthy | Synced 2m ago
  - Pods: 3/3 Running
  - URL: https://task-5-preview.ngrok.io

### Production Deployments  
- **task-3-prod** ✅ Healthy | Synced 15m ago
  - Pods: 2/2 Running
  - URL: https://task-3-prod.ngrok.io

**Last Check:** 2025-11-05 10:45:00 UTC
```

**Correlation Logic:**
- ArgoCD app name format: `task-{id}-preview` or `task-{id}-prod`
- Extract task ID from app name
- Find PR associated with task ID (via labels: `task-*`)
- Post updates to that PR

---

### Mode 2: Bolt-Preview (Event-Driven)

**Purpose:** Create preview deployments for active development

**Trigger:** 
- PR opened
- PR updated (new commits pushed)
- Label `preview-deployment` added (optional)

**Responsibilities:**
1. Create preview namespace: `agent-platform-preview-task-{id}`
2. Create preview ArgoCD application: `task-{id}-preview`
   - Source: PR branch (not main)
   - Destination namespace: preview namespace
   - Auto-sync enabled
3. Wait for ArgoCD sync (timeout: 5 minutes)
4. Create ngrok Tunnel for preview access
5. Verify preview URL responds (HTTP 200/301/302)
6. Post preview URL to PR

**Implementation:**
- Sensor: `bolt-preview-sensor.yaml` (PR webhooks)
- Script: `container-bolt-preview.sh.hbs`
- Prompt: `agents/bolt-preview-system-prompt.md.hbs`

**ArgoCD Application Spec (Preview):**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: task-{id}-preview
  namespace: argocd
  labels:
    task-id: "{id}"
    environment: preview
    managed-by: bolt
spec:
  project: default
  source:
    repoURL: https://github.com/org/repo.git
    targetRevision: feature/branch-name  # PR branch!
    path: helm
  destination:
    server: https://kubernetes.default.svc
    namespace: agent-platform-preview-task-{id}
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

**Ngrok Tunnel Spec (Preview):**
```yaml
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: Tunnel
metadata:
  name: task-{id}-preview
  namespace: agent-platform-preview-task-{id}
spec:
  forwardsTo: service-name:port
  labels:
    task-id: "{id}"
    environment: preview
```

**Comment Format:**
```markdown
## 🔍 Bolt Preview: Task {id}

✅ **Preview environment is ready!**

### Preview Details
- **URL:** https://abc123-preview.ngrok.io
- **Environment:** `agent-platform-preview-task-{id}`
- **Branch:** `feature/branch-name`
- **ArgoCD App:** `task-{id}-preview`

### Deployment Status
- ✅ ArgoCD: Synced & Healthy
- ✅ Pods: 3/3 Running
- ✅ Public URL: Verified accessible

**This preview updates automatically when you push new commits!**

---
*Deployed by Bolt Preview at 2025-11-05 10:30:00 UTC*
```

**Cleanup Triggers:**
- PR closed (merged or not)
- Label `preview-deployment` removed (optional)
- Preview older than 7 days (configurable)

---

### Mode 3: Bolt-Production (Event-Driven)

**Purpose:** Deploy to production after all quality gates pass

**Trigger:**
- PR merged to main
- Label `ready-for-production` exists
- All previous agents approved (Cleo, Tess)

**Responsibilities:**
1. **Clean up preview deployment** (if exists)
   - Delete preview ArgoCD application
   - Delete preview namespace
   - Delete preview ngrok tunnel
2. Create production namespace: `agent-platform-prod-task-{id}`
3. Create production ArgoCD application: `task-{id}-prod`
   - Source: main branch (merged code)
   - Destination namespace: production namespace
   - Auto-sync enabled
4. Wait for ArgoCD sync (timeout: 10 minutes)
5. Create ngrok Tunnel for production access
6. Verify production URL responds
7. Post production URL to PR

**Implementation:**
- Sensor: `bolt-deployment-monitor-sensor.yaml` (existing, enhanced)
- Script: `container-bolt.sh.hbs` (existing, enhanced)
- Prompt: `agents/bolt-system-prompt.md.hbs` (existing, enhanced)

**ArgoCD Application Spec (Production):**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: task-{id}-prod
  namespace: argocd
  labels:
    task-id: "{id}"
    environment: production
    managed-by: bolt
spec:
  project: default
  source:
    repoURL: https://github.com/org/repo.git
    targetRevision: main  # Production uses main!
    path: helm
  destination:
    server: https://kubernetes.default.svc
    namespace: agent-platform-prod-task-{id}
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

**Comment Format:**
```markdown
## 🚀 Bolt Production: Task {id}

✅ **Your application is LIVE and publicly accessible!**

### Production Details
- **URL:** https://xyz789-prod.ngrok.io
- **Environment:** `agent-platform-prod-task-{id}`
- **Branch:** `main`
- **ArgoCD App:** `task-{id}-prod`

### Deployment Status
- ✅ ArgoCD: Synced & Healthy
- ✅ Pods: 3/3 Running
- ✅ Public URL: Verified accessible
- ✅ Preview Cleaned Up

### Quality Gates Passed
- ✅ Cleo: Code Quality
- ✅ Tess: QA Testing
- ✅ Atlas: Integration & Merge

**Your app is ready for users! 🎉**

---
*Deployed by Bolt Production at 2025-11-05 11:00:00 UTC*
```

---

## 🔄 Complete Workflow Integration

```
1. Rex implements code
   ↓ Creates PR with task-{id} label
   
2. Bolt-Preview activates
   ↓ Creates preview deployment
   ↓ Posts preview URL to PR
   
3. [Monitor runs continuously]
   ↓ Posts health updates
   
4. Cleo reviews quality
   ↓ Uses preview URL for context
   ↓ Approves PR
   
5. Tess runs QA tests
   ↓ Tests against preview URL
   ↓ Validates acceptance criteria
   ↓ Approves & adds "ready-for-production" label
   
6. Atlas merges to main
   ↓ PR closed (merged)
   
7. Bolt-Production activates
   ↓ Cleans up preview
   ↓ Creates production deployment
   ↓ Posts production URL
   
8. [Monitor continues]
   ↓ Tracks production health
   ↓ Posts ongoing updates
```

---

## 🎨 Naming Conventions

### Task ID Extraction
- PR label: `task-5` → Task ID: `5`
- Branch name: `task-5-implement-auth` → Task ID: `5`
- Correlation marker: `.taskmaster-correlation` file

### Namespace Naming
- Preview: `agent-platform-preview-task-{id}`
- Production: `agent-platform-prod-task-{id}`

### ArgoCD Application Naming
- Preview: `task-{id}-preview`
- Production: `task-{id}-prod`

### Ngrok Tunnel Naming
- Preview: `task-{id}-preview`
- Production: `task-{id}-prod`

---

## 🔒 Security & Isolation

### Preview Deployments
- ✅ Separate namespace per task
- ✅ NetworkPolicy isolation
- ✅ ResourceQuota limits
- ✅ Automatic cleanup after 7 days
- ✅ Public URL via ngrok (acceptable for preview)

### Production Deployments
- ✅ Separate namespace per task
- ✅ Production-grade NetworkPolicy
- ✅ Production ResourceQuota
- ✅ Persistent (no auto-cleanup)
- ✅ Public URL via ngrok (can upgrade to custom domain)

---

## 📊 Monitoring & Observability

### Bolt-Monitor Metrics
- Applications tracked: `bolt_monitor_apps_total`
- Health status: `bolt_monitor_health_status{app, status}`
- Sync status: `bolt_monitor_sync_status{app, status}`
- Check interval: 30 seconds
- PR updates: On status change only (avoid spam)

### Bolt-Preview Metrics
- Previews created: `bolt_preview_created_total`
- Preview deployment time: `bolt_preview_deploy_duration_seconds`
- Preview failures: `bolt_preview_failures_total`

### Bolt-Production Metrics
- Production deployments: `bolt_production_deployed_total`
- Production deployment time: `bolt_production_deploy_duration_seconds`
- Production failures: `bolt_production_failures_total`

---

## 🛠️ Configuration

### values.yaml Structure
```yaml
agents:
  bolt:
    name: "Bolt"
    githubApp: "5DLabs-Bolt"
    
    # Monitor mode configuration
    monitor:
      enabled: true
      interval: 30  # seconds
      updateOnChange: true  # Only post PR updates on status change
      
    # Preview mode configuration  
    preview:
      enabled: true
      autoCleanup: true
      retentionDays: 7
      namespace: "agent-platform-preview-task-{id}"
      
    # Production mode configuration
    production:
      enabled: true
      cleanupPreview: true  # Clean up preview on production deploy
      namespace: "agent-platform-prod-task-{id}"
      
    # Common configuration
    syncTimeout: 600  # 10 minutes
    verifyAccessibility: true
```

---

## 🧪 Testing Scenarios

### Test 1: Preview Deployment
1. Rex creates PR for task-5
2. **Expected:** Bolt-Preview creates preview deployment
3. **Expected:** Preview URL posted to PR within 2 minutes
4. **Expected:** Bolt-Monitor starts tracking preview
5. **Verify:** Preview URL is accessible

### Test 2: Monitor Updates
1. Preview deployment running
2. Make change that causes pod to restart
3. **Expected:** Bolt-Monitor detects status change
4. **Expected:** PR comment updated with new status

### Test 3: Production Deployment
1. Tess approves PR, adds "ready-for-production"
2. Atlas merges to main
3. **Expected:** Bolt-Production deletes preview
4. **Expected:** Production deployment created
5. **Expected:** Production URL posted to PR
6. **Expected:** Bolt-Monitor tracks production

### Test 4: Preview Cleanup
1. Close PR without merging
2. **Expected:** Bolt-Preview sensor detects closed PR
3. **Expected:** Preview ArgoCD app deleted
4. **Expected:** Preview namespace deleted
5. **Expected:** Preview ngrok tunnel deleted

---

## 🚀 Implementation Phases

### Phase 1: Bolt-Monitor (Daemon)
- [ ] Create monitor sensor YAML
- [ ] Create monitor container script
- [ ] Create monitor system prompt
- [ ] Deploy and test monitoring

### Phase 2: Bolt-Preview (Event-Driven)
- [ ] Create preview sensor YAML
- [ ] Create preview container script
- [ ] Create preview system prompt
- [ ] Implement namespace/ArgoCD/ngrok creation
- [ ] Test preview deployments

### Phase 3: Bolt-Production Enhancement
- [ ] Enhance existing production script
- [ ] Add preview cleanup logic
- [ ] Test production with preview handoff

### Phase 4: Integration & Testing
- [ ] Test all three modes together
- [ ] Validate end-to-end workflow
- [ ] Load testing and observability

---

## 💡 Benefits

### For Developers
- ✅ **Instant preview URLs** - See your code live immediately
- ✅ **Continuous monitoring** - Always know deployment status
- ✅ **Safe production** - Only deploys after all quality gates

### For QA (Tess)
- ✅ **Real deployment testing** - Test against actual deployed version
- ✅ **Preview isolation** - Each task has its own environment
- ✅ **Production confidence** - Preview matches production exactly

### For Platform
- ✅ **Event-driven** - Scales effortlessly
- ✅ **Resource efficient** - Auto-cleanup of previews
- ✅ **Observable** - Full monitoring of all deployments

---

## 📝 Future Enhancements

### Multi-Ingress Support
- Support ALB, NGINX, Traefik in addition to ngrok
- Custom domain mapping for production
- TLS/SSL certificate management

### Advanced Monitoring
- Integration with Prometheus/Grafana
- Slack/Discord notifications on failures
- Cost tracking per environment

### Preview Sharing
- Generate shareable preview links
- Authentication for preview environments
- Screenshot/video capture of previews

---

**Status:** Ready for implementation! 🚀

