# Task 5: Create GitHub Webhook Correlation Logic

## Overview

Implement Argo Events Sensor logic to extract task IDs from webhook payloads and correlate with suspended workflows using precise label selectors, enabling event-driven coordination between GitHub events and multi-agent workflows.

## Technical Context

The multi-agent orchestration system requires sophisticated correlation logic to map GitHub webhook events to specific suspended workflows. This task implements the critical link between external GitHub events (PR creation, labeling, approval) and internal workflow state transitions.

## Implementation Guide

### Phase 1: Task ID Extraction Logic

1. **JQ Expression Development**
   ```yaml
   # Extract task ID from PR labels
   - src:
       dependencyName: github-pr-event
       dataTemplate: |
         {{jq '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}
     dest: spec.arguments.parameters.task-id
   ```

2. **Fallback Branch Name Parsing**
   ```yaml
   # Extract task ID from branch name as fallback
   - src:
       dependencyName: github-pr-event
       dataTemplate: |
         {{jq '.pull_request.head.ref | capture("^task-(?<id>[0-9]+)-.*").id // empty'}}
     dest: spec.arguments.parameters.branch-task-id
   ```

### Phase 2: Workflow Correlation Implementation

1. **Label Selector Construction**
   ```yaml
   triggers:
   - template:
       name: resume-after-pr-created
       argoWorkflow:
         operation: resume
         source:
           resource:
             apiVersion: argoproj.io/v1alpha1
             kind: Workflow
             metadata:
               labelSelector: |
                 workflow-type=play-orchestration,
                 task-id={{task-id}},
                 current-stage=waiting-pr-created
   ```

2. **Multi-Stage Targeting**
   ```yaml
   # Different events target different workflow stages
   - name: pr-created-correlation
     targetStage: "waiting-pr-created"
   - name: ready-for-qa-correlation  
     targetStage: "waiting-ready-for-qa"
   - name: pr-approved-correlation
     targetStage: "waiting-pr-approved"
   ```

### Phase 3: Event Type Processing

1. **PR Creation Events**
   ```yaml
   dependencies:
   - name: github-pr-opened
     eventSourceName: github
     eventName: pull_request
     filters:
       data:
       - path: action
         type: string
         value: "opened"
       - path: pull_request.labels
         type: []
         template: |
           {{ range .pull_request.labels }}
           {{- if hasPrefix .name "task-" }}true{{ end }}
           {{ end }}
   ```

2. **PR Labeling Events**
   ```yaml
   dependencies:
   - name: github-pr-labeled
     eventSourceName: github
     eventName: pull_request
     filters:
       data:
       - path: action
         type: string
         value: "labeled"
       - path: label.name
         type: string
         value: "ready-for-qa"
   ```

3. **PR Approval Events**
   ```yaml
   dependencies:
   - name: github-pr-reviewed
     eventSourceName: github
     eventName: pull_request_review
     filters:
       data:
       - path: action
         type: string
         value: "submitted"
       - path: review.state
         type: string
         value: "approved"
   ```

### Phase 4: Validation and Error Handling

1. **Multi-Method Validation**
   ```yaml
   # Validate task ID correlation using both methods
   - name: validate-correlation
     script:
       image: alpine:latest
       command: [sh]
       source: |
         LABEL_TASK="{{task-id}}"
         BRANCH_TASK="{{branch-task-id}}"
         
         if [ "$LABEL_TASK" != "$BRANCH_TASK" ] && [ -n "$BRANCH_TASK" ]; then
           echo "ERROR: Task ID mismatch - Label: $LABEL_TASK, Branch: $BRANCH_TASK"
           exit 1
         fi
         
         echo "Task correlation validated: $LABEL_TASK"
   ```

2. **Workflow Existence Validation**
   ```yaml
   # Verify target workflow exists before resume
   - name: verify-target-workflow
     resource:
       action: get
       manifest: |
         apiVersion: argoproj.io/v1alpha1
         kind: Workflow
         metadata:
           labelSelector: |
             workflow-type=play-orchestration,
             task-id={{task-id}},
             current-stage={{target-stage}}
   ```

## Code Examples

### Complete Sensor Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-webhook-correlation
  namespace: argo
spec:
  dependencies:
  - name: github-pr-created
    eventSourceName: github
    eventName: pull_request
    filters:
      data:
      - path: action
        type: string
        value: "opened"
  
  - name: github-pr-labeled
    eventSourceName: github
    eventName: pull_request
    filters:
      data:
      - path: action
        type: string
        value: "labeled"
      - path: label.name
        type: string
        value: "ready-for-qa"
  
  - name: github-pr-approved
    eventSourceName: github
    eventName: pull_request_review
    filters:
      data:
      - path: action
        type: string
        value: "submitted"
      - path: review.state
        type: string
        value: "approved"
  
  triggers:
  - template:
      name: correlate-and-resume
      argoWorkflow:
        operation: resume
        parameters:
        - src:
            dependencyName: github-pr-created
            dataTemplate: |
              {{jq '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}
          dest: spec.arguments.parameters.task-id
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              labelSelector: |
                workflow-type=play-orchestration,
                task-id={{task-id}},
                current-stage=waiting-pr-created
```

### Task ID Extraction Functions
```bash
# JQ expressions for different payload structures
extract_task_from_labels() {
  echo '$payload' | jq -r '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'
}

extract_task_from_branch() {
  echo '$payload' | jq -r '.pull_request.head.ref | capture("^task-(?<id>[0-9]+)-.*").id // empty'
}

validate_task_correlation() {
  LABEL_TASK=$(extract_task_from_labels)
  BRANCH_TASK=$(extract_task_from_branch)
  
  if [ "$LABEL_TASK" != "$BRANCH_TASK" ] && [ -n "$BRANCH_TASK" ]; then
    echo "ERROR: Task correlation mismatch"
    return 1
  fi
  
  echo "$LABEL_TASK"
}
```

### Conditional Workflow Targeting
```yaml
# Dynamic stage targeting based on event type
- template:
    name: dynamic-workflow-resume
    conditions: |
      {{ if eq .Input.action "opened" }}
      waiting-pr-created
      {{ else if eq .Input.label.name "ready-for-qa" }}
      waiting-ready-for-qa  
      {{ else if eq .Input.review.state "approved" }}
      waiting-pr-approved
      {{ end }}
    argoWorkflow:
      operation: resume
      source:
        resource:
          labelSelector: |
            workflow-type=play-orchestration,
            task-id={{task-id}},
            current-stage={{target-stage}}
```

## Architecture Patterns

### Event-Driven Correlation
The correlation system implements a sophisticated mapping between:
1. **GitHub Events**: PR actions, label changes, review submissions
2. **Workflow State**: Suspended workflows waiting for specific events
3. **Task Association**: Multiple validation methods ensure accurate correlation

### Multi-Method Validation Strategy
```yaml
Validation Hierarchy:
1. Primary: PR labels (task-3) - most reliable for automation
2. Secondary: Branch names (task-3-feature) - human readable backup
3. Validation: Both methods must agree or processing fails
4. Fallback: Use primary if secondary missing, error if mismatch
```

### Precise Workflow Targeting
- **Label Selectors**: Combine multiple criteria for exact workflow matching
- **Stage Awareness**: Different events target workflows in specific stages
- **Namespace Isolation**: Workflow targeting respects namespace boundaries

## Key Implementation Details

### Webhook Payload Processing
- **JQ Expressions**: Robust JSON field extraction from complex payloads
- **Error Handling**: Graceful failure when expected fields are missing
- **Type Safety**: Validate extracted data types before processing

### Correlation Accuracy
- **False Positive Prevention**: Multi-method validation prevents wrong correlations
- **Race Condition Handling**: Atomic operations prevent concurrent conflicts
- **Idempotency**: Repeated events don't cause duplicate resumptions

### Performance Considerations
- **Efficient Filtering**: Use Argo Events filters to reduce processing load
- **Label Indexing**: Kubernetes label selectors are efficient for workflow lookup
- **Payload Size**: Minimize data extraction and processing overhead

## Testing Strategy

### Unit Testing
1. **JQ Expression Testing**: Validate extraction with various payload formats
2. **Label Selector Validation**: Test targeting accuracy with mock workflows
3. **Error Condition Testing**: Verify handling of malformed payloads
4. **Performance Testing**: Benchmark extraction and correlation speed

### Integration Testing
1. **End-to-End Correlation**: Test with actual GitHub webhook events
2. **Concurrent Workflow Handling**: Multiple simultaneous workflows
3. **Error Scenario Validation**: Missing workflows, invalid task IDs
4. **Race Condition Testing**: Rapid sequential events

### Operational Validation
1. **GitHub Webhook Delivery**: Test with real PR workflows
2. **Sensor Processing Latency**: Measure event-to-resume timing
3. **Correlation Accuracy**: Validate targeting precision in production
4. **Error Recovery**: Test failure scenarios and recovery mechanisms

## References

- [Argo Events Sensor Configuration](https://argoproj.github.io/argo-events/concepts/sensor/)
- [GitHub Webhook Payloads](https://docs.github.com/en/developers/webhooks-and-events/webhooks/webhook-events-and-payloads)
- [JQ Manual for JSON Processing](https://stedolan.github.io/jq/manual/)
- [Multi-Agent Architecture](.taskmaster/docs/architecture.md)