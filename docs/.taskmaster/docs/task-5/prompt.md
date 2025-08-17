# Autonomous Agent Prompt: Create GitHub Webhook Correlation Logic (Aligned)

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

**✅ SUPPORTED Patterns:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows)
- Setting `dest: metadata.name` deterministically

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!

## Mission

Implement Argo Events Sensor logic to extract task IDs from GitHub webhook payloads and correlate them with suspended workflows using deterministic workflow names (no templated labelSelectors), enabling precise event-driven coordination.

## Context

The multi-agent workflow uses GitHub events to trigger stage transitions. Rex creates PRs, Cleo adds labels, and Tess approves PRs. Each event must resume the correct suspended workflow based on task correlation.

## Objectives

1. **Implement JQ Extraction Logic**
   - Extract task IDs from PR labels using JQ expressions
   - Parse task numbers from label format "task-X"
   - Handle multiple labels and edge cases

2. **Create Correlation Mechanisms**
   - Use deterministic workflow names: `play-task-{{task-id}}-workflow`
   - Resume workflows by `metadata.name`
   - Implement fallback strategies for missing labels (branch parsing)

3. **Handle Multiple Event Types**
   - PR opened events for post-Rex resumption
   - PR labeled events for post-Cleo resumption
   - PR review events for post-Tess resumption
   - Push events for Rex remediation triggers

4. **Implement Dynamic Parameterization**
   - Use Argo Events v1.9+ features for dynamic targeting
   - Pass extracted task IDs to workflow operations
   - Update workflow stage labels after resumption (if needed)

## Technical Requirements

### JQ Expression Implementation
```jq
# Primary extraction from PR labels
.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]

# Fallback extraction from branch name
.pull_request.head.ref | capture("^task-(?<id>[0-9]+)") | .id
```

### Deterministic Workflow Name Targeting (No labelSelector templating)
```yaml
triggers:
- template:
    name: resume-workflow
    argoWorkflow:
      operation: resume
      source:
        resource:
          apiVersion: argoproj.io/v1alpha1
          kind: Workflow
          metadata:
            # Deterministic name computed from payload
            name: "PLACEHOLDER_REPLACED_BY_PARAMETER"
      parameters:
        - src:
            dependencyName: github-pr-event
            dataTemplate: |
              play-task-{{jq '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}-workflow
          dest: argoWorkflow.source.resource.metadata.name
```

### Event Type Mapping
- `pull_request.opened` → `waiting-pr-created`
- `pull_request.labeled` → `waiting-ready-for-qa`
- `pull_request_review.submitted` → `waiting-pr-approved`
- `push` → trigger remediation logic

## Implementation Approach

### Step 1: Create Sensor Templates
Develop reusable Sensor templates with:
- Event dependency configuration
- JQ extraction logic
- Deterministic workflow name targeting via metadata.name
- Error handling for edge cases

### Step 2: Implement Extraction Logic
- Primary extraction from PR labels
- Secondary extraction from branch names
- Validation of extracted task IDs
- Handling of missing or malformed data

### Step 3: Configure Correlation
- Use computed metadata.name to target the workflow
- Handle concurrent workflow scenarios by unique names
- Add logging for debugging

### Step 4: Test Edge Cases
- Missing task labels
- Multiple task labels
- Malformed label formats
- Concurrent workflows for same task

## Code Examples

### Sensor Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-pr-correlation
spec:
  dependencies:
  - name: pr-event
    eventSourceName: github
    eventName: pull-request
    filters:
      data:
      - path: action
        type: string
        value: ["opened", "labeled"]

  triggers:
  - template:
      name: resume-workflow
      argoWorkflow:
        operation: resume
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              name: "TO_BE_SET"
        parameters:
        - src:
            dependencyName: pr-event
            dataTemplate: |
              play-task-{{jq '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}-workflow
          dest: argoWorkflow.source.resource.metadata.name
```

### JQ Extraction Step (Inline Example)
```yaml
- name: extract-task-id
  inline:
    script: |
      echo '{{inputs.body}}' | \
      jq -r '.pull_request.labels[] | 
             select(.name | startswith("task-")) | 
             .name | split("-")[1]' | \
      head -1
```

## Success Criteria

- Task IDs correctly extracted from all webhook types
- Workflows accurately targeted by deterministic name
- Fallback mechanisms work when primary extraction fails
- No false positive correlations
- Proper handling of concurrent workflows
- Clear logging for troubleshooting

## Testing Requirements

1. **Unit Tests**: JQ expressions with sample payloads
2. **Integration Tests**: End-to-end webhook processing
3. **Edge Case Tests**: Missing labels, malformed data
4. **Performance Tests**: Concurrent event processing
5. **Failure Tests**: Network issues, malformed webhooks

## Important Notes

- Ensure backward compatibility with existing sensors
- Implement comprehensive error handling
- Add metrics for correlation success/failure rates
- Document all JQ expressions thoroughly
- Consider rate limiting and webhook replay scenarios

Your implementation must be robust enough to handle production webhook traffic while maintaining precise workflow correlation using supported Argo Events patterns.