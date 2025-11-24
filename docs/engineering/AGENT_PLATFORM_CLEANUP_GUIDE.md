# Agent Platform Resource Cleanup Guide

## Overview

The `agent-platform` namespace accumulates completed resources from workflow executions. This guide covers cleanup procedures and automation strategies.

## Current State (Before Cleanup)

```
✅ Succeeded Pods:      53
❌ Failed Pods:         0
✅ Succeeded Workflows: 39
✅ Succeeded CodeRuns:  16
```

**Total orphaned resources: ~108** (taking up valuable pod capacity)

## Manual Cleanup

### Quick Cleanup (Recommended)

Use the provided cleanup script:

```bash
./scripts/cleanup-agent-platform.sh
```

This script will:
1. Delete succeeded CodeRuns (cascades to pods/workflows)
2. Delete orphaned succeeded workflows
3. Delete succeeded pods
4. Delete failed pods
5. Report resources freed

**Expected pod savings: ~50-53 pods immediately**

### Manual Commands (If Needed)

```bash
# Delete succeeded CodeRuns
kubectl delete coderuns -n agent-platform --field-selector=status.phase==Succeeded

# Delete succeeded workflows
kubectl get workflows -n agent-platform | grep Succeeded | awk '{print $1}' | \
  xargs kubectl delete workflow -n agent-platform

# Delete succeeded pods
kubectl delete pods -n agent-platform --field-selector=status.phase==Succeeded

# Delete failed pods
kubectl delete pods -n agent-platform --field-selector=status.phase==Failed
```

## Automated Cleanup (Long-Term Solution)

### Workflow TTL Strategy

Workflows are configured with TTL (Time To Live) strategies:

**Play Workflows** (`play-workflow-template.yaml`):
```yaml
ttlStrategy:
  secondsAfterCompletion: 86400    # 24 hours
  secondsAfterFailure: 259200      # 3 days
  secondsAfterSuccess: 86400       # 24 hours
podGC:
  strategy: OnPodCompletion
  deleteDelayDuration: 60s
```

**CodeRun/DocsRun Workflows**:
```yaml
ttlStrategy:
  secondsAfterCompletion: 300      # 5 minutes
  secondsAfterFailure: 10800       # 3 hours
podGC:
  strategy: OnPodCompletion
```

### Global Argo Workflows Configuration

Added global defaults to `infra/gitops/applications/argo-workflows.yaml`:

```yaml
controller:
  workflowDefaults:
    spec:
      ttlStrategy:
        secondsAfterCompletion: 3600   # 1 hour default
        secondsAfterFailure: 10800     # 3 hours for failures
      podGC:
        strategy: OnPodCompletion
        deleteDelayDuration: 60s
```

This ensures ALL workflows (even those without explicit TTL) get cleaned up automatically.

### Controller-Based Cleanup

The controller has cleanup configuration in `values.yaml`:

```yaml
cleanup:
  enabled: true
  completedJobDelayMinutes: 5
  failedJobDelayMinutes: 60
  deleteConfigMap: true
```

This handles CodeRun/DocsRun resource cleanup after completion.

### Controller-Managed TTL Sweep (New)

- Every CodeRun and DocsRun resource now carries explicit cleanup labels (`cleanup.cto.dev/scope=run`, `cleanup.cto.dev/run=<name>`) so the controller can safely target only workflow-scoped artifacts.
- When a run reaches a terminal phase, the controller records `finishedAt`, computes an `expireAt` deadline (default: 60s for success, 300s for failure—configurable via `cleanup.successTTLSeconds` and `cleanup.failureTTLSeconds`), and schedules a follow-up reconciliation.
- After `expireAt` passes, the controller requeues automatically, deletes remaining Jobs/Pods/ConfigMaps via the resource manager, then stamps `cleanupCompletedAt` to avoid double work.
- Operators can opt out per run by setting the annotation `cleanup.cto.dev/preserve: "true"` or override the TTL with `cleanup.cto.dev/ttl-seconds: "<seconds>"`.
- Observability: `finishedAt`, `expireAt`, and `cleanupCompletedAt` now surface in the `status` block of each run for quick audits and troubleshooting.

## Preventing Resource Accumulation

### 1. Enable Workflow Archiving (Optional)

For workflows you want to keep for auditing:

```yaml
# In Argo Workflows config
artifactRepository:
  archiveLogs: true
  s3:  # Or other storage backend
    bucket: workflow-archives
    endpoint: s3.amazonaws.com
    insecure: false
```

### 2. CronJob for Regular Cleanup (Recommended)

Create a Kubernetes CronJob to run cleanup daily:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: agent-platform-cleanup
  namespace: agent-platform
spec:
  schedule: "0 2 * * *"  # Run at 2 AM daily
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: cleanup-sa
          containers:
          - name: cleanup
            image: bitnami/kubectl:latest
            command:
            - /bin/bash
            - -c
            - |
              # Delete succeeded resources older than 24 hours
              kubectl delete pods -n agent-platform \
                --field-selector=status.phase==Succeeded \
                --field-selector=metadata.creationTimestamp<$(date -u -d '24 hours ago' +%Y-%m-%dT%H:%M:%SZ)
              
              kubectl delete workflows -n agent-platform \
                --field-selector=status.phase==Succeeded
              
              kubectl delete coderuns -n agent-platform \
                --field-selector=status.phase==Succeeded
          restartPolicy: OnFailure
```

### 3. Monitoring & Alerts

Set up alerts for resource accumulation:

```yaml
# Prometheus alert rule
- alert: AgentPlatformHighPodCount
  expr: count(kube_pod_info{namespace="agent-platform"}) > 50
  for: 10m
  annotations:
    summary: "High pod count in agent-platform namespace"
    description: "{{ $value }} pods detected, cleanup may be needed"
```

## Resource Limits & Quotas

Consider setting resource quotas to prevent runaway resource consumption:

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: agent-platform-quota
  namespace: agent-platform
spec:
  hard:
    pods: "100"
    requests.cpu: "20"
    requests.memory: "40Gi"
    limits.cpu: "40"
    limits.memory: "80Gi"
```

## Best Practices

1. **Run cleanup script weekly** during low-activity periods
2. **Monitor pod count** regularly: `kubectl get pods -n agent-platform | wc -l`
3. **Adjust TTL values** based on debugging needs:
   - Development: Shorter TTL (1-6 hours)
   - Production: Longer TTL (24-48 hours)
4. **Archive important workflow logs** before TTL expires
5. **Set up automated cleanup** via CronJob for hands-off maintenance

## Troubleshooting

### Pods Stuck in Terminating

```bash
# Force delete stuck pods
kubectl delete pod <pod-name> -n agent-platform --force --grace-period=0
```

### Workflows Not Cleaning Up

Check Argo Workflows controller logs:
```bash
kubectl logs -n argo deployment/argo-workflows-controller | grep -i ttl
```

### CodeRuns Not Cascading Deletion

Check controller logs:
```bash
kubectl logs -n agent-platform deployment/controller | grep -i cleanup
```

## Deployment Instructions

After updating Argo Workflows configuration, commit and push:

```bash
git add infra/gitops/applications/argo-workflows.yaml
git commit -m "feat: add global workflow TTL and pod GC configuration"
git push origin <branch>
```

ArgoCD will automatically sync the changes. The new TTL settings will apply to all future workflows.

## Expected Results

After implementing these strategies:

- **Immediate**: 50-53 pods freed from manual cleanup
- **Ongoing**: Automatic cleanup keeps pod count under 20-30
- **Long-term**: No manual intervention needed for routine cleanup

## Monitoring Cleanup

```bash
# Watch pods being cleaned up
kubectl get pods -n agent-platform --watch

# Count total pods
kubectl get pods -n agent-platform --no-headers | wc -l

# Check workflow count
kubectl get workflows -n agent-platform --no-headers | wc -l

# Check CodeRun count
kubectl get coderuns -n agent-platform --no-headers | wc -l
```

## Summary

- ✅ **Manual cleanup available**: `./scripts/cleanup-agent-platform.sh`
- ✅ **Global TTL configured**: Automatic cleanup for all workflows
- ✅ **Pod GC enabled**: Immediate pod deletion after completion
- ✅ **Controller cleanup**: CodeRun/DocsRun automatic cleanup
- 📋 **Consider**: Setting up CronJob for regular automated cleanup


