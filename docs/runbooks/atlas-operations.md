# Atlas Operations Runbook

## Quick Reference

### Check Atlas Status
```bash
# Active Atlas CodeRuns
kubectl get coderuns -n agent-platform -l agent=atlas

# Guardian CodeRuns
kubectl get coderuns -n agent-platform -l agent=atlas,role=guardian

# Integration CodeRuns  
kubectl get coderuns -n agent-platform -l agent=atlas,role=integration

# Atlas workflows
kubectl get workflows -n agent-platform -l type=atlas-pr-monitor
kubectl get workflows -n agent-platform -l type=stage-resume,target-stage=waiting-atlas-integration
```

### View Atlas Logs
```bash
# Get logs for specific CodeRun
kubectl logs -n agent-platform coderun-atlas-guardian-xxxxx -c agent

# Follow logs
kubectl logs -n agent-platform coderun-atlas-guardian-xxxxx -c agent -f

# Get sensor logs
kubectl logs -n argo deployment/atlas-pr-monitor-sensor-controller
```

## Common Issues and Solutions

### Issue: Atlas Not Activating on PR Open

**Symptoms:**
- PR created but no Atlas guardian CodeRun
- No `atlas-pr-monitor` workflows created

**Diagnosis:**
```bash
# Check if sensor is running
kubectl get sensors -n argo atlas-pr-monitor

# Check sensor logs
kubectl logs -n argo deployment/atlas-pr-monitor-sensor-controller

# Check GitHub webhook delivery
# Go to GitHub → Settings → Webhooks → Recent Deliveries
```

**Solution:**
1. Verify sensor is deployed and running
2. Check GitHub webhook configuration
3. Ensure EventSource is receiving events:
   ```bash
   kubectl logs -n argo deployment/github-eventsource-xxxxx
   ```

### Issue: Atlas Not Triggering After Tess Approval

**Symptoms:**
- Tess approved PR but Atlas integration not starting
- Workflow stuck at `waiting-atlas-integration`

**Diagnosis:**
```bash
# Check workflow stage
kubectl get workflows -n agent-platform \
  -l workflow=play-orchestration \
  -o jsonpath='{.items[*].metadata.labels.current-stage}'

# Check Tess approval sensor
kubectl logs -n argo deployment/stage-aware-tess-approval-sensor-controller

# Look for integration gate workflows
kubectl get workflows -n agent-platform \
  -l type=stage-resume,target-stage=waiting-atlas-integration
```

**Solution:**
1. Manually trigger Atlas integration:
   ```bash
   # Create integration CodeRun manually
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     generateName: coderun-atlas-integration-manual-
     namespace: agent-platform
     labels:
       agent: atlas
       role: integration
       pr-number: "YOUR_PR_NUMBER"
   spec:
     taskId: "YOUR_TASK_ID"
     service: "atlas"
     githubApp: "5DLabs-Atlas"
     model: "claude-sonnet-4-5-20250929"
     repositoryUrl: "https://github.com/5dlabs/cto.git"
     env:
       - name: PR_NUMBER
         value: "YOUR_PR_NUMBER"
       - name: ATLAS_MODE
         value: "integration-gate"
   EOF
   ```

2. Resume workflow manually:
   ```bash
   kubectl patch workflow WORKFLOW_NAME -n agent-platform \
     --type='json' -p='[{"op": "replace", "path": "/metadata/labels/current-stage", "value": "atlas-integration-in-progress"}]'
   ```

### Issue: Duplicate Atlas CodeRuns

**Symptoms:**
- Multiple Atlas CodeRuns for same PR
- High resource usage

**Diagnosis:**
```bash
# Check for duplicate CodeRuns
kubectl get coderuns -n agent-platform \
  -l agent=atlas,pr-number=YOUR_PR_NUMBER

# Check ConfigMap locks
kubectl get configmaps -n agent-platform \
  -l atlas-guardian-lock
```

**Solution:**
1. Delete duplicate CodeRuns:
   ```bash
   # Keep only the oldest one
   kubectl delete coderuns -n agent-platform \
     -l agent=atlas,pr-number=YOUR_PR_NUMBER \
     --field-selector metadata.name!=KEEP_THIS_ONE
   ```

2. Clean up stale locks:
   ```bash
   kubectl delete configmaps -n agent-platform \
     -l atlas-guardian-lock \
     --field-selector metadata.creationTimestamp<2h
   ```

### Issue: Atlas Stuck in Loop

**Symptoms:**
- Atlas repeatedly trying same fix
- CodeRun running for hours

**Diagnosis:**
```bash
# Check runtime
kubectl get coderuns -n agent-platform \
  -l agent=atlas \
  -o custom-columns=NAME:.metadata.name,AGE:.metadata.creationTimestamp

# Check cycle count in logs
kubectl logs -n agent-platform coderun-atlas-xxxxx -c agent | grep "Cycle"
```

**Solution:**
1. Set max cycles if not set:
   ```bash
   kubectl set env coderun/coderun-atlas-xxxxx \
     -n agent-platform \
     ATLAS_MAX_CYCLES=60
   ```

2. Restart the CodeRun:
   ```bash
   kubectl delete pod -n agent-platform \
     -l coderun=coderun-atlas-xxxxx
   ```

### Issue: Merge Conflicts Not Being Detected

**Symptoms:**
- PR has conflicts but Atlas not activated
- GitHub shows "This branch has conflicts"

**Diagnosis:**
```bash
# Check conflict monitor sensor
kubectl logs -n argo deployment/atlas-conflict-monitor-sensor-controller

# Manually check PR status via API
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/5dlabs/cto/pulls/PR_NUMBER | \
  jq '.mergeable, .mergeable_state'
```

**Solution:**
1. Manually trigger conflict resolution:
   ```bash
   # Simulate conflict event
   kubectl apply -f - <<EOF
   apiVersion: v1
   kind: Event
   metadata:
     name: manual-conflict-PR_NUMBER
     namespace: argo
   data:
     # ... (see test script for structure)
   EOF
   ```

## Maintenance Tasks

### Clean Up Old Resources

```bash
# Delete completed workflows older than 1 day
kubectl delete workflows -n agent-platform \
  --field-selector status.phase=Succeeded \
  --field-selector metadata.creationTimestamp<24h

# Delete old ConfigMap locks
kubectl delete configmaps -n agent-platform \
  -l atlas-guardian-lock \
  --field-selector metadata.creationTimestamp<2h

# Delete failed CodeRuns
kubectl delete coderuns -n agent-platform \
  --field-selector status.phase=Failed
```

### Monitor Atlas Performance

```bash
# Get Atlas metrics
curl -s http://victoria-metrics:8428/api/v1/query \
  -d 'query=atlas:coderuns:active' | jq

# Check success rates
curl -s http://victoria-metrics:8428/api/v1/query \
  -d 'query=atlas:success_rate:bugbot' | jq
```

### Update Atlas Configuration

```bash
# Update Atlas model
kubectl edit configmap controller-config -n agent-platform
# Find atlas section, update model field

# Update system prompt
helm upgrade controller infra/charts/controller \
  --reuse-values \
  --set agents.atlas.systemPrompt="NEW_PROMPT"

# Restart controller to pick up changes
kubectl rollout restart deployment/controller -n agent-platform
```

## Emergency Procedures

### Disable Atlas Completely

```bash
# Scale down all Atlas sensors
kubectl scale sensor atlas-pr-monitor -n argo --replicas=0
kubectl scale sensor atlas-conflict-monitor -n argo --replicas=0
kubectl scale sensor atlas-batch-integration -n argo --replicas=0

# Delete all running Atlas CodeRuns
kubectl delete coderuns -n agent-platform -l agent=atlas
```

### Force Merge Without Atlas

```bash
# Skip Atlas stage in workflow
kubectl patch workflow WORKFLOW_NAME -n agent-platform \
  --type='json' -p='[
    {"op": "replace", "path": "/metadata/labels/current-stage", "value": "waiting-pr-merged"}
  ]'

# Manually merge PR
gh pr merge PR_NUMBER --squash --admin
```

### Rollback Atlas Changes

```bash
# Revert to previous sensor configuration
cd infra/gitops/resources/github-webhooks
git checkout HEAD~1 -- atlas-*.yaml
git checkout HEAD~1 -- stage-aware-tess-approval-sensor.yaml

# Apply rollback
kubectl apply -k .

# Revert workflow template
helm rollback controller -n agent-platform
```

## Monitoring Checklist

### Daily Checks
- [ ] Check active Atlas CodeRuns count (should be < 10)
- [ ] Review Atlas success rates (> 80%)
- [ ] Check for stuck workflows at Atlas stage
- [ ] Clean up old ConfigMap locks

### Weekly Checks
- [ ] Review Atlas integration times (P95 < 10 min)
- [ ] Analyze failure patterns in logs
- [ ] Update documentation based on incidents
- [ ] Review and tune alert thresholds

### Metrics to Watch

| Metric | Normal Range | Alert Threshold |
|--------|-------------|-----------------|
| Active CodeRuns | 0-5 | > 10 |
| Guardian Success Rate | > 90% | < 70% |
| Integration Time (P95) | < 10 min | > 30 min |
| Conflict Resolution Rate | > 85% | < 60% |
| ConfigMap Locks | < 20 | > 50 |

## Contact and Escalation

### Primary Contacts
- Platform Team: #platform-support
- On-call: PagerDuty → CTO-Platform

### Escalation Path
1. Check runbook for known issues
2. Review sensor and CodeRun logs
3. Contact platform team in Slack
4. Page on-call if critical (workflow blocked > 1 hour)

### Useful Links
- [Atlas Architecture](../atlas-integration-architecture.md)
- [GitHub App Settings](https://github.com/organizations/5dlabs/settings/apps/5dlabs-atlas)
- [Grafana Dashboard](https://grafana.5dlabs.ai/d/atlas-integration)
- [Alert Manager](https://alerts.5dlabs.ai/#/alerts?filter=component%3Datlas)

---

*Last Updated: November 2025*
*Version: 1.0*
