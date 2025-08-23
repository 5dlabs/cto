# Rex Remediation Sensor Implementation

## 🚨 CRITICAL: Argo Events Reference Documentation

**BEFORE implementing ANY Argo Events sensors/triggers, MUST review official examples:**
- **Location:** [docs/references/argo-events/](../../../references/argo-events/)
- **Key Files:**


  - `github.yaml` - GitHub webhook sensor patterns


  - `complete-trigger-parameterization.yaml` - Dynamic parameter extraction


  - `special-workflow-trigger.yaml` - ArgoWorkflow operations (submit/resume)


  - `trigger-standard-k8s-resource.yaml` - K8s resource creation patterns

**❌ UNSUPPORTED Operations (will cause deployment failures):**
- `operation: delete` ❌
- `operation: patch` ❌
- `operation: update` ❌


- Template variables in `labelSelector` ❌

**✅ SUPPORTED Operations:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows)
- `dest: metadata.name` (dynamic targeting)

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!

You are implementing a dedicated Argo Events Sensor to detect Rex push events and automatically restart the QA pipeline with downstream agent cancellation. This ensures clean pipeline restarts when Rex addresses PR feedback.



## Objective

Create an event-driven system that detects when Rex pushes code fixes and automatically cancels any running Cleo/Tess work, resets the workflow state, and restarts the QA pipeline from the quality-work stage.

## Context

When Rex addresses PR feedback by pushing fixes, any ongoing Cleo or Tess work becomes obsolete since it's based on outdated code. The system needs to:


- **Detect Rex push events** on task branches


- **Cancel running downstream agents** (Cleo, Tess) for the affected task


- **Remove ready-for-qa labels** to reset pipeline state


- **Update workflow stage** back to waiting-pr-created


- **Resume the main workflow** to restart the QA pipeline

## Implementation Requirements

### 1. Create Rex Push EventSource

Configure GitHub webhook EventSource to detect Rex pushes:



```yaml
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: rex-remediation-events
spec:
  github:
    rex-push:
      events: ["push"]
      filter:
        expression: |
          body.sender.login == "5DLabs-Rex[bot]" &&
          body.ref matches "refs/heads/task-.*"






```

### 2. Implement Agent Cancellation Logic

Create Kubernetes operations to cancel running agents:



```bash
# Cancel downstream agents using label selectors
kubectl delete coderun -l "task-id=${TASK_ID},github-app in (5DLabs-Cleo,5DLabs-Tess)"






```

### 3. Build Rex Remediation Sensor

Create sensor that orchestrates the complete remediation flow:


- Event detection and validation


- Agent cancellation with proper label targeting


- GitHub API calls to remove ready-for-qa labels


- Workflow stage reset to waiting-pr-created


- Main workflow resumption

### 4. Implement Safety and Validation

Add comprehensive validation:


- Verify sender is actually 5DLabs-Rex[bot]


- Extract and validate task ID from branch name


- Check for existing agents before cancellation


- Include idempotency to handle duplicate events

## Technical Specifications

### Event Processing Flow






```
GitHub Push → EventSource → Sensor → Remediation Workflow → Agent Cancellation → Label Removal → Stage Reset → Pipeline Resume






```

### Task ID Extraction
Extract task ID from branch names using regex:






```
refs/heads/task-3-implement-auth → task ID: 3
refs/heads/task-15-add-feature → task ID: 15






```

### Label Selector Targeting
Use precise label selectors to target only affected agents:



```yaml
labelSelector: |
  task-id={{task-id}},
  github-app in (5DLabs-Cleo,5DLabs-Tess)






```

### Idempotent Operations
Design all operations to be safe for retry and duplication:


- Agent deletion ignores non-existent resources


- Label removal checks existence before removal


- Workflow updates verify current state


- Resume operations check workflow status

## Workflow Integration Points

### Remediation Workflow Structure



```yaml
dag:
  tasks:
  - name: validate-event
    template: validate-rex-push
  - name: cancel-agents
    dependencies: [validate-event]
    template: cancel-downstream-agents
  - name: remove-qa-label
    dependencies: [cancel-agents]
    template: remove-ready-for-qa-label
  - name: reset-workflow
    dependencies: [remove-qa-label]
    template: reset-workflow-stage
  - name: resume-pipeline
    dependencies: [reset-workflow]
    template: resume-main-workflow






```

### GitHub API Integration
Use GitHub CLI or API to manage PR labels:



```bash


# Remove ready-for-qa label from affected PR
gh pr edit ${PR_NUMBER} --remove-label "ready-for-qa"






```

### Main Workflow Coordination
Target and manipulate main play workflows:



```bash
# Find workflow by task ID
WORKFLOW_NAME=$(kubectl get workflow \


  -l "workflow-type=play-orchestration,task-id=${TASK_ID}" \


  -o jsonpath='{.items[0].metadata.name}')

# Reset stage and resume
kubectl patch workflow "$WORKFLOW_NAME" \


  --type='merge' \
  --patch='{"metadata":{"labels":{"current-stage":"waiting-pr-created"}}}'
argo resume "$WORKFLOW_NAME"






```



## Success Criteria

1. **Event Detection**: Rex push events correctly detected and filtered
2. **Agent Cancellation**: All running Cleo/Tess agents cancelled for affected task
3. **Label Management**: Ready-for-qa labels removed from affected PRs
4. **Workflow Reset**: Main workflow stage reset to waiting-pr-created
5. **Pipeline Restart**: QA pipeline resumes cleanly from quality-work stage
6. **Safety**: No false positives or unintended cancellations
7. **Idempotency**: Duplicate events handled gracefully without side effects

## Testing Requirements

### Event Simulation Testing


- Test Rex push event detection with various branch name patterns


- Verify filtering correctly identifies Rex vs other user pushes


- Test task ID extraction from branch names

### Agent Cancellation Testing


- Create test CodeRun CRDs and verify they're cancelled correctly


- Test label selector targeting doesn't affect unrelated agents


- Verify cancellation completes before proceeding to next step

### Integration Testing


- Test complete remediation flow end-to-end


- Verify main workflow resumes correctly after remediation


- Test multiple concurrent tasks don't interfere with each other


- Test rapid sequential pushes are handled properly

### Error Handling Testing


- Test behavior when no agents are running to cancel


- Test handling of missing PRs or GitHub API failures


- Test workflow resumption when main workflow is not found


- Test validation failures and error propagation

## Implementation Deliverables

### EventSource Configuration


- GitHub webhook EventSource with Rex push filtering


- Proper webhook endpoint and authentication setup


- Event data extraction for task ID correlation

### Sensor Implementation


- Complete Rex remediation sensor with trigger logic


- Remediation workflow template with all required steps


- Error handling and validation throughout the flow

### Integration Scripts


- Agent cancellation scripts with label selector targeting


- GitHub API integration for label removal


- Workflow manipulation scripts for stage reset and resumption

Focus on creating a reliable, safe system that automatically handles Rex remediation while preventing any unintended cancellations or pipeline disruptions.
