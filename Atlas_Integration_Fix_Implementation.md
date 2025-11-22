# Atlas Integration Fix - Implementation Plan

## Consensus Root Causes (All 6 Agents Agree)

1. **Tess PR Review Fails Silently** - Script continues despite `gh pr review` failure
2. **No Fallback Sensor** - Only listens for PR review events, not label events  
3. **Missing Atlas Stage** - Workflow skips from testing → merge without Atlas

## IMMEDIATE FIXES (Deploy Now)

### Fix 1: Make Tess PR Review Fatal
**File**: `infra/charts/controller/agent-templates/code/integration/container-tess.sh.hbs`

Replace lines 2417-2437:
```bash
echo "📝 Posting APPROVE review (CRITICAL)..."
MAX_RETRIES=3
RETRY_COUNT=0
REVIEW_SUCCESS=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
  if timeout 30 gh pr review "$PR_NUMBER" -R "$REPO_SLUG" --approve --body "### 🧪 QA Testing - APPROVED

All tests passed. Ready for integration.

*QA testing by Tess*" 2>&1; then
    REVIEW_SUCCESS=true
    echo "✅ PR review submitted successfully"
    break
  else
    REVIEW_EXIT=$?
    echo "⚠️ PR review failed (attempt $((RETRY_COUNT+1))/$MAX_RETRIES)"
    RETRY_COUNT=$((RETRY_COUNT+1))
    [ $RETRY_COUNT -lt $MAX_RETRIES ] && sleep 5
  fi
done

if [ "$REVIEW_SUCCESS" = "false" ]; then
  echo "❌ CRITICAL: Failed to submit PR approval after $MAX_RETRIES attempts"
  echo "   Checking GitHub App permissions..."
  if ! gh api "/repos/$REPO_SLUG/installation" >/dev/null 2>&1; then
    echo "❌ GitHub App not installed on $REPO_SLUG"
  fi
  exit 1  # FAIL THE CODERUN
fi
```

### Fix 2: Add Label Fallback Sensor
Create `infra/gitops/resources/github-webhooks/tess-label-fallback-sensor.yaml`:
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: tess-label-fallback
  namespace: argo
spec:
  template:
    serviceAccountName: argo-events-sa
  dependencies:
    - name: approved-label
      eventSourceName: github
      eventName: org
      filters:
        data:
          - path: body.X-GitHub-Event
            type: string
            value: ["pull_request"]
          - path: body.action
            type: string
            value: ["labeled"]
          - path: body.label.name
            type: string
            value: ["approved"]
  triggers:
    - template:
        name: trigger-atlas-on-label
        conditions: "approved-label"
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              metadata:
                generateName: atlas-label-trigger-
                namespace: agent-platform
              spec:
                entrypoint: trigger-atlas
                serviceAccountName: argo-workflow
                templates:
                  - name: trigger-atlas
                    script:
                      image: alpine/k8s:1.31.0
                      command: [bash]
                      source: |
                        #!/bin/bash
                        PR_NUMBER='{{workflow.parameters.pr-number}}'
                        TASK_ID=$(echo '{{workflow.parameters.pr-labels}}' | jq -r '.[] | select(.name | test("task-[0-9]+")) | .name' | grep -oE '[0-9]+' | head -1)
                        
                        # Find and resume workflow
                        WORKFLOW=$(kubectl get workflows -n agent-platform -l "task-id=$TASK_ID" --field-selector status.phase=Running -o name | head -1)
                        if [ -n "$WORKFLOW" ]; then
                          kubectl patch $WORKFLOW -n agent-platform --type='json' -p='[{"op":"replace","path":"/spec/suspend","value":null}]'
                          echo "✅ Resumed workflow via label fallback"
                        fi
```

Update `infra/gitops/resources/github-webhooks/kustomization.yaml`:
```yaml
resources:
  # ... existing ...
  - tess-label-fallback-sensor.yaml  # ADD THIS
```

### Fix 3: Verify/Add Atlas Stage
Check if stage exists:
```bash
grep -n "waiting-atlas-integration" infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml
```

If missing, add after `testing-work`:
```yaml
- name: update-to-waiting-atlas
  dependencies: [testing-work]
  template: update-workflow-stage
  arguments:
    parameters:
      - name: new-stage
        value: "waiting-atlas-integration"

- name: wait-for-atlas-integration
  dependencies: [update-to-waiting-atlas]
  template: suspend-for-event

- name: update-to-waiting-merge
  dependencies: [wait-for-atlas-integration]
  template: update-workflow-stage
  arguments:
    parameters:
      - name: new-stage
        value: "waiting-pr-merged"
```

## VERIFICATION STEPS

### 1. Test Happy Path
```bash
# Run test workflow
./scripts/test-play-workflow.sh --task-id 200

# Verify Tess PR review
gh pr view 200 --repo 5dlabs/cto-parallel-test --json reviews

# Verify Atlas triggers
kubectl get coderuns -n agent-platform -l task-id=200,agent=atlas
```

### 2. Test Failure Path (Cipher Blocks)
```bash
# Create PR with security issues
./scripts/test-play-workflow.sh --task-id 201 --with-security-issues

# Verify Tess fails but label triggers Atlas
kubectl logs -n agent-platform $(kubectl get pods -n agent-platform -l task-id=201,stage=testing -o name)
kubectl get workflows -n agent-platform -l type=stage-resume
```

### 3. Check Stuck Workflows
```bash
# Find stuck workflows
kubectl get workflows -n agent-platform \
  -l current-stage=waiting-atlas-integration \
  --field-selector status.phase=Running

# Resume manually if needed
for wf in $(kubectl get workflows -n agent-platform -l current-stage=waiting-atlas-integration -o name); do
  kubectl patch $wf -n agent-platform --type='json' -p='[{"op":"replace","path":"/spec/suspend","value":null}]'
done
```

## MONITORING

Add alert for stuck workflows:
```yaml
alert: WorkflowStuckAtAtlas
expr: |
  (time() - argo_workflow_status_phase_timestamp{
    label_current_stage="waiting-atlas-integration"
  }) > 1800
annotations:
  summary: "Workflow stuck > 30min"
```

## SUCCESS CRITERIA

✅ Tess CodeRuns fail when PR review fails (no silent failures)
✅ Label fallback sensor triggers when PR review blocked
✅ Atlas triggers within 5 minutes of Tess completion
✅ No workflows stuck > 30 minutes
✅ Parallel execution works without manual intervention

## ROLLBACK

If issues occur:
```bash
# Revert Tess script
git revert HEAD

# Disable fallback sensor
kubectl scale sensor tess-label-fallback -n argo --replicas=0

# Resume all stuck workflows
kubectl get workflows -n agent-platform -o name | xargs -I {} kubectl patch {} -n agent-platform --type='json' -p='[{"op":"replace","path":"/spec/suspend","value":null}]'
```

---

**Total Time**: 4-6 hours
**Risk**: Low with testing
**Impact**: Restores full multi-agent pipeline

*Implementation Plan - November 22, 2025*

