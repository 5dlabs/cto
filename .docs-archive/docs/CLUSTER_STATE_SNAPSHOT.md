# Cluster State Snapshot - Namespace Consolidation Refactor

**Date:** 2025-11-27  
**Context:** Post namespace consolidation refactor (PR #1738, #1739, #1740, #1742, #1743)
**Last Updated:** 2025-11-27 04:10 UTC (Post-remediation)

---

## 1. Desired Namespace Structure (Target State)

| Namespace | Purpose | Components |
|-----------|---------|------------|
| `cto` | Main application | cto-controller, cto-tools, openmemory |
| `automation` | Argo Workflows & Events | argo-workflows, argo-events, sensors, eventbus, eventsources |
| `observability` | Monitoring & Logging | grafana, victoria-metrics, victoria-logs, otel-collector, fluent-bit, kube-state-metrics |
| `infra` | Operators & Controllers | cert-manager, ingress-nginx, external-dns, cloudnative-pg, redis-operator, ngrok-operator, vault-secrets-operator, arc-controller |
| `arc-runners` | GitHub Actions Runners | ephemeral runner pods |
| `databases` | Data Layer | PostgreSQL, Redis instances |
| `vault` | Secrets Management | HashiCorp Vault |
| `argocd` | GitOps | ArgoCD components |

**Namespaces Removed:**
- `argo` → merged into `automation`
- `telemetry` → renamed to `observability`
- `operators` → renamed to `infra`
- `arc-systems` → merged into `infra`
- `cert-manager` → merged into `infra`
- `external-dns` → merged into `infra`
- `ingress-nginx` → merged into `infra`

---

## 2. Current Cluster State

### 2.1 ArgoCD Applications (Post-Remediation)

| Application | Sync | Health | Namespace | Notes |
|-------------|------|--------|-----------|-------|
| arc-controller | Unknown | Healthy | infra | OCI chart issue, manually deployed |
| argo-events | Synced | Healthy | automation | ✅ |
| argo-workflows | Synced | Healthy | automation | ✅ |
| cert-manager | Synced | Healthy | infra | ✅ |
| cloudnative-pg-operator | Synced | Healthy | infra | ✅ |
| cto-controller | Synced | **Degraded** | cto | ⚠️ Needs image rebuild |
| cto-tools | Synced | **Degraded** | cto | ⚠️ Needs Vault secret |
| external-dns | Synced | Healthy | infra | ✅ |
| fluent-bit | Synced | Healthy | observability | ✅ |
| github-webhooks | Synced | Healthy | automation | ✅ Fixed (PR #1742) |
| grafana | Synced | Healthy | observability | ✅ |
| ingress-nginx | Synced | Healthy | infra | ✅ |
| k8s-runner | Synced | Healthy | arc-runners | ✅ |
| ngrok-gateway | Synced | Progressing | default | ✅ |
| ngrok-operator | Synced | Healthy | infra | ✅ |
| otel-collector | Synced | Healthy | observability | ✅ Fixed |
| redis-operator | Synced | Healthy | infra | ✅ |
| vault | Synced | Healthy | vault | ✅ |
| vault-config | Synced | Healthy | infra | ✅ Fixed (PR #1743) |
| vault-secrets-operator | Synced | Healthy | infra | ✅ |
| victoria-logs | Synced | Healthy | observability | ✅ |
| victoria-metrics | Synced | Healthy | observability | ✅ |

### 2.2 Pod Status by Namespace (Post-Remediation)

#### automation (Argo Workflows/Events) - ✅ ALL HEALTHY
| Pod | Status | Issue |
|-----|--------|-------|
| argo-events-controller-manager | Running | ✅ |
| argo-workflows-server | Running | ✅ |
| argo-workflows-workflow-controller | Running | ✅ |
| eventbus-default-stan-0/1/2 | Running | ✅ Fixed (PR #1742) |
| github-eventsource | Running | ✅ |

#### cto (Main Application)
| Pod | Status | Issue |
|-----|--------|-------|
| cto-controller | **CrashLoopBackOff** | Image has hardcoded `agent-platform` namespace |
| cto-tools (init) | **Init:0/1** | `tools-kubernetes-secrets` not in Vault |
| cto-tools | Running | ✅ (old replica) |
| cto-tools-k8s-mcp | ContainerCreating | Waiting for init |
| openmemory | Running | ✅ |

#### infra (Operators)
| Pod | Status |
|-----|--------|
| arc-controller-gha-rs-controller | Running ✅ |
| cert-manager (3 pods) | Running ✅ |
| cloudnative-pg-operator | Running ✅ |
| external-dns | Running ✅ |
| ingress-nginx-controller | Running ✅ |
| k8s-runner-listener | Running ✅ |
| ngrok-operator (3 pods) | Running ✅ |
| redis-operator | Running ✅ |
| vault-secrets-operator | Running ✅ |

#### observability (Monitoring)
| Pod | Status |
|-----|--------|
| fluent-bit | Running ✅ |
| grafana | Running ✅ |
| kube-state-metrics | Running ✅ |
| otel-collector | Running ✅ |
| victoria-logs | Running ✅ |
| victoria-metrics | Running ✅ |

#### arc-runners (GitHub Runners)
- 7 runners currently active ✅

#### databases
- test-postgres-1: Running ✅
- test-redis-0: Running ✅

---

## 3. Critical Issues (Post-Remediation Status)

### Issue 1: EventBus CrashLoopBackOff - ✅ FIXED
**Error:** `parameter "max_age" value is expected to be string, got int64`

**Root Cause:** NATS streaming has issues with duration parameter types.

**Fix Applied:** Removed `maxAge`, `maxMsgs`, `maxBytes` parameters to use NATS defaults (PR #1742)

**Status:** ✅ EventBus running with 3 replicas, EventSource connected

---

### Issue 2: tools-kubernetes-secrets Missing - ⚠️ PENDING
**Error:** `MountVolume.SetUp failed for volume "kubeconfig" : secret "tools-kubernetes-secrets" not found`

**Root Cause:** Vault path `secret/data/tools-kubernetes` returns empty response.

**Fix Required:** Create the secret in Vault manually:
```bash
vault kv put secret/tools-kubernetes KUBECONFIG="<base64-kubeconfig>"
```

**Impact:**
- cto-tools init container fails → cto-tools-k8s-mcp can't start

---

### Issue 3: cto-controller Hardcoded Namespace - ⚠️ PENDING
**Error:** Controller looking for ConfigMaps in `agent-platform` namespace

**Root Cause:** The `ghcr.io/5dlabs/controller:latest` image was built with hardcoded `agent-platform` namespace in an older version of the code.

**Fix Required:** Rebuild and push the controller image with current code that uses `cto` namespace.

**Impact:**
- cto-controller can't start → No task/workflow processing

---

### Issue 4: Stuck Terminating Namespaces - ✅ FIXED
**Namespaces:** `argo`, `arc-systems`

**Status:** ✅ Cleared by removing finalizers.

---

### Issue 5: Ingress-NGINX Webhook Certificate - ✅ FIXED
**Error:** TLS certificate mismatch after namespace change

**Root Cause:** Webhook certificate was issued for `ingress-nginx.svc` but service moved to `infra.svc`

**Fix Applied:** Force synced ingress-nginx app which regenerated certificates

---

### Issue 6: vault-config Sync Error - ✅ FIXED
**Error:** `namespaces "trader" not found`

**Root Cause:** ghcr.yaml had entries for non-existent `trader` namespace

**Fix Applied:** Removed trader namespace entries and duplicates (PR #1743)

---

## 4. Previous State (Before Refactor)

### Old Namespace Structure
| Old Namespace | New Location | Status |
|---------------|--------------|--------|
| `argo` | `automation` | Migrated |
| `telemetry` | `observability` | Migrated |
| `operators` | `infra` | Migrated |
| `arc-systems` | `infra` | Migrated |
| `cert-manager` | `infra` | Migrated |
| `external-dns` | `infra` | Migrated |
| `ingress-nginx` | `infra` | Migrated |
| `cnpg-system` | `infra` | Already migrated earlier |
| `redis-operator` | `infra` | Already migrated earlier |
| `ngrok-operator` | `infra` | Already migrated earlier |
| `vault-secrets-operator` | `infra` | Already migrated earlier |

### Key Configuration Changes
1. All `vaultAuthRef` updated from `operators/vault-auth` → `infra/vault-auth`
2. All DNS references updated from `.telemetry.svc` → `.observability.svc`
3. All sensor/eventbus namespaces updated from `argo` → `automation`
4. Prometheus alert expressions updated with new namespace labels

---

## 5. Remediation Steps

### Completed Steps ✅

1. **Fixed EventBus** (PR #1742):
   - Removed problematic NATS config parameters
   - EventBus now running with 3 replicas
   - EventSource connected and receiving webhooks

2. **Fixed Ingress-NGINX Webhook Certificate**:
   - Force synced ingress-nginx app
   - Certificates regenerated for infra namespace

3. **Fixed vault-config** (PR #1743):
   - Removed trader namespace references
   - Removed duplicate entries from ghcr.yaml

4. **Synced all OutOfSync applications**:
   - otel-collector now healthy
   - ngrok-gateway synced
   - vault-config synced

### Remaining Manual Steps ⚠️

1. **Create tools-kubernetes secret in Vault:**
   ```bash
   vault kv put secret/tools-kubernetes KUBECONFIG="<base64-kubeconfig>"
   ```

2. **Rebuild cto-controller image:**
   - The current image has hardcoded `agent-platform` namespace
   - Code in `controller/src/bin/agent_controller.rs` is correct (`cto`)
   - Need to trigger CI/CD to rebuild and push new image

### Verification Commands

1. Check all pods running:
   ```bash
   kubectl get pods -A --field-selector=status.phase!=Running,status.phase!=Succeeded | grep -v Completed
   ```

2. Verify ArgoCD sync status:
   ```bash
   argocd app list | grep -E "OutOfSync|Degraded|Missing"
   ```

3. Test GitHub webhook processing:
   - Create a test PR or comment
   - Check sensors: `kubectl get sensors -n automation`
   - Watch for workflows: `kubectl get workflows -n automation`

---

## 6. Files Changed in Refactor

### Namespace Changes (60+ files)
- All files in `infra/gitops/applications/` - destination namespaces updated
- All files in `infra/gitops/resources/` - metadata namespaces updated
- All files in `infra/vault/secrets/` - vaultAuthRef updated
- All files in `infra/telemetry/alerts/` - namespace labels in expressions
- DNS references in monitoring configs

### Key PRs
- **#1738** - Comprehensive namespace consolidation
- **#1739** - arc-controller OCI format fix
- **#1740** - eventbus maxAge quoting + platform-runners namespace
- **#1742** - Remove maxAge config from eventbus (NATS type fix)
- **#1743** - Remove trader namespace and duplicate entries from ghcr.yaml

---

## 7. Rollback Plan

If critical issues persist, revert to pre-refactor state:

```bash
# Revert all 5 PRs from the namespace consolidation refactor (in reverse order)
git revert <commit-for-PR-1743>  # PR #1743 - Remove trader namespace and duplicate entries
git revert <commit-for-PR-1742>  # PR #1742 - Remove maxAge config from eventbus
git revert ec3b2050              # PR #1740 - eventbus maxAge quoting + platform-runners
git revert 6d406abf              # PR #1739 - arc-controller OCI format fix
git revert 4a5259cc              # PR #1738 - Comprehensive namespace consolidation
git push origin main
```

**Important:** All 5 PRs (#1738, #1739, #1740, #1742, #1743) must be reverted together
to avoid leaving the system in an inconsistent state. Reverting only some PRs will
cause namespace mismatches between resources.

Then manually restore old namespaces and redeploy.



