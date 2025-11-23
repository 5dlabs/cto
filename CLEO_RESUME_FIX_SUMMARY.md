# Cleo Resume Fix - Implementation Summary

## Problem Statement

Rex (implementation agent) was completing successfully, but Cleo (quality agent) never ran. When play workflows were re-submitted, they always restarted at Task 1 (Rex) instead of resuming at the Cleo quality stage.

## Root Causes Identified

### 1. ConfigMap Creation Gap
**Location**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`

The `update-workflow-stage` template only **patched** existing ConfigMaps but never **created** them:

```yaml
if kubectl get configmap "$CONFIGMAP_NAME" -n {{ .Release.Namespace }} >/dev/null 2>&1; then
  # Patch existing ConfigMap
  kubectl patch configmap "$CONFIGMAP_NAME" ...
else
  echo "ConfigMap does not exist yet - will be created by controller"
  # ❌ NO ACTUAL CREATION HAPPENED
fi
```

**Impact**: If the MCP server's initial ConfigMap write failed or was garbage collected, all subsequent stage transitions silently failed. The `determine-resume-point` step would find no ConfigMap and force a restart from `implementation`.

### 2. Sensor Looking for Removed Suspend Node
**Location**: `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml` and `stage-aware-pr-created.yaml`

The PR-created sensors were polling for a `wait-for-pr-created` suspend node that was removed in a previous refactor:

```bash
SUSPEND_PHASE=$(echo "$WORKFLOW_JSON" | \
  jq -r '(.items[0].status.nodes // {})
    | to_entries[]?
    | select(.value.displayName == "wait-for-pr-created")  # ❌ THIS NODE NO LONGER EXISTS
    | .value.phase' 2>/dev/null || echo "")

if [ "$SUSPEND_READY" != true ]; then
  echo "⚠️ Workflow did not reach wait-for-pr-created suspend point in time; skipping resume"
  exit 0  # ❌ SENSOR ALWAYS EXITED HERE
fi
```

**Impact**: Sensors always logged "skipping resume" and exited without updating workflow metadata. Rex would time out waiting for the PR to be detected, and workflows never progressed to Cleo.

## Solutions Implemented

### Fix 1: ConfigMap Creation in update-workflow-stage

**File**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`

Added ConfigMap creation logic when it doesn't exist:

```yaml
else
  echo "⚠️ ConfigMap does not exist - creating it now..."
  
  # Create ConfigMap with current stage
  cat <<EOF | kubectl apply -f - || {
    echo "❌ Failed to create ConfigMap, but workflow will continue"
  }
apiVersion: v1
kind: ConfigMap
metadata:
  name: $CONFIGMAP_NAME
  namespace: {{ .Release.Namespace }}
  labels:
    play-tracking: "true"
data:
  repository: "$REPO"
  branch: "main"
  current-task-id: "$TASK_ID"
  workflow-name: "{{`{{workflow.name}}`}}"
  status: "in-progress"
  stage: "{{`{{inputs.parameters.new-stage}}`}}"
  started-at: "$CURRENT_TIMESTAMP"
  last-updated: "$CURRENT_TIMESTAMP"
EOF
  
  echo "✅ ConfigMap created with stage: {{`{{inputs.parameters.new-stage}}`}}"
fi
```

**Benefits**:
- Stage transitions now persist even if MCP bootstrap failed
- Resume functionality becomes self-healing
- No dependency on external ConfigMap creation

### Fix 2: Updated PR-Created Sensors

**Files**: 
- `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`
- `infra/gitops/resources/github-webhooks/stage-aware-pr-created.yaml`

Removed suspend node polling and updated to check workflow stage directly:

```bash
# NOTE: wait-for-pr-created suspend node was removed
# Workflows now poll for PR in implementation-cycle
# We just need to update PR metadata - the workflow will detect it

if [ "$CURRENT_STAGE" = "implementation-in-progress" ] && [ "$WORKFLOW_PHASE" = "Running" ]; then
  echo "✅ Workflow is in implementation stage - updating PR metadata..."
  
  # Update workflow parameters with PR details
  if ! update_workflow_pr_metadata "$WORKFLOW_NAME" \
      "{{ .Input.body.pull_request.html_url }}" \
      "{{ .Input.body.pull_request.number }}"; then
    echo "⚠️ Failed to update PR metadata on $WORKFLOW_NAME"
    echo "   Workflow will continue polling GitHub API for PR"
  else
    echo "✅ PR metadata updated - workflow can now proceed to quality stage"
  fi
elif [ "$CURRENT_STAGE" = "quality-in-progress" ] || [ "$CURRENT_STAGE" = "security-in-progress" ] || [ "$CURRENT_STAGE" = "testing-in-progress" ]; then
  echo "ℹ️ Workflow already progressed past implementation (stage: $CURRENT_STAGE)"
  echo "   No action needed"
else
  echo "ℹ️ Workflow not in expected stage (current: $CURRENT_STAGE, phase: $WORKFLOW_PHASE)"
  echo "   Skipping to prevent incorrect stage progression"
fi
```

**Benefits**:
- Sensors now work with current workflow structure
- Faster detection (2s polling vs 10s)
- Fewer polling attempts (30 vs 90)
- Better error messages and logging
- Graceful handling of already-progressed workflows

## Expected Behavior After Fix

1. **Rex completes** and creates PR
2. **ConfigMap is updated** to `quality-in-progress` (even if it didn't exist before)
3. **PR-created sensor** detects workflow in `implementation-in-progress` stage
4. **Sensor updates** workflow parameters with PR URL/number
5. **Workflow proceeds** to Cleo quality stage automatically
6. **If workflow is re-run**, `determine-resume-point` reads ConfigMap and skips Rex, starting at Cleo

## Testing Recommendations

1. **Fresh workflow test**:
   ```bash
   # Delete any existing ConfigMap
   kubectl delete configmap play-progress-5dlabs-cto -n agent-platform --ignore-not-found
   
   # Submit new workflow
   cto play --task-id 1
   
   # Verify ConfigMap is created during stage transitions
   kubectl get configmap play-progress-5dlabs-cto -n agent-platform -o yaml
   ```

2. **Resume test**:
   ```bash
   # After Rex completes and ConfigMap shows quality-in-progress
   # Submit same task again
   cto play --task-id 1
   
   # Verify workflow skips Rex and starts at Cleo
   kubectl logs -n agent-platform -l workflows.argoproj.io/workflow=<name> | grep "Resuming at"
   ```

3. **Sensor test**:
   ```bash
   # Watch sensor logs during PR creation
   kubectl logs -f -n argo -l sensor-name=stage-aware-pr-created
   
   # Should see:
   # "✅ Workflow is in implementation stage - updating PR metadata..."
   # "✅ PR metadata updated - workflow can now proceed to quality stage"
   ```

## Files Changed

- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
  - Added ConfigMap creation in `update-workflow-stage` template
  
- `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`
  - Removed suspend node polling
  - Updated to check workflow stage directly
  - Improved logging and error handling
  
- `infra/gitops/resources/github-webhooks/stage-aware-pr-created.yaml`
  - Removed suspend node polling
  - Updated to check workflow stage directly
  - Added graceful handling for already-progressed workflows

## Deployment

After merging to main, ArgoCD will automatically sync:
1. Controller Helm chart (workflow template changes)
2. GitHub webhooks (sensor changes)

No manual intervention required.

## Rollback Plan

If issues occur:
```bash
# Revert to previous commit
git revert <commit-hash>
git push origin main

# ArgoCD will auto-sync the revert
# Or force sync immediately:
argocd app sync platform-controller
argocd app sync platform-github-webhooks
```

## Related Documentation

- Original investigation: `RESUME_DEBUG.md`
- Architecture reference: `docs/.taskmaster/docs/architecture.md`
- Workflow resume design: `WORKFLOW_RESUME_IMPLEMENTATION_PLAN.md`

