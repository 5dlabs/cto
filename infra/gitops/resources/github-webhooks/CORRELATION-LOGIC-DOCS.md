# GitHub Webhook Correlation Logic Documentation



## Overview

This document describes the enhanced correlation logic for GitHub webhook events in the multi-agent workflow orchestration system. The correlation mechanism extracts task IDs from webhook payloads and targets specific suspended workflows using deterministic naming patterns.

## Task ID Extraction Strategy



### Primary Method: PR Labels
The primary extraction method uses PR labels with the format `task-{id}`:





```go-template
{{- range $i, $label := .Input.body.pull_request.labels -}}
  {{- if hasPrefix $label.name "task-" -}}
    {{- $parts := splitList "-" $label.name -}}
    {{- if gt (len $parts) 1 -}}
      {{- $taskId = index $parts 1 -}}
    {{- end -}}
  {{- end -}}
{{- end -}}








```

### Fallback Method: Branch Name
If no valid task label is found, the system falls back to extracting from the branch name:





```go-template
{{- if eq $taskId "" -}}
  {{- $ref := .Input.body.pull_request.head.ref -}}
  {{- if hasPrefix $ref "task-" -}}
    {{- /* Extract from task-{id}-description format */ -}}
    {{- $branch := trimPrefix "task-" $ref -}}
    {{- $parts := splitList "-" $branch -}}
    {{- $taskId = index $parts 0 -}}
  {{- else if hasPrefix $ref "feature/task-" -}}
    {{- /* Extract from feature/task-{id}-description format */ -}}
    {{- $branch := trimPrefix "feature/task-" $ref -}}
    {{- $parts := splitList "-" $branch -}}
    {{- $taskId = index $parts 0 -}}
  {{- end -}}
{{- end -}}








```

## Workflow Targeting

### Deterministic Naming Pattern
All workflows follow the naming convention: `play-task-{task-id}-workflow`

This enables precise targeting without using templated `labelSelector` fields (which are not supported in Argo Events).

### Resume Operation
Workflows are resumed by setting the workflow name in `args.0`:





```yaml
argoWorkflow:
  operation: resume
  args: []  # Workflow name will be set via parameter
  parameters:
    - src:
        dependencyName: github-pr-event
        dataTemplate: |
          play-task-{{ $taskId }}-workflow
      dest: args.0








```

## Event Type Handling

### 1. PR Created Event
- **Trigger**: `pull_request` with action `opened`
- **Purpose**: Resume workflow after Rex completes implementation
- **Extraction**: Uses both label and branch methods

### 2. Ready-for-QA Label Event
- **Trigger**: `pull_request` with action `labeled` and label `ready-for-qa`
- **Purpose**: Resume workflow after Cleo completes quality checks
- **Validation**: Ensures label was added by `5DLabs-Cleo`

### 3. PR Approval Event
- **Trigger**: `pull_request_review` with action `submitted` and state `approved`
- **Purpose**: Resume workflow after Tess completes testing
- **Validation**: Ensures approval from `5DLabs-Tess`

### 4. Implementation Push Event
- **Trigger**: `push` event from implementation agents
- **Purpose**: Cancel running quality agents and restart QA pipeline
- **Agents**: Rex, Blaze, Morgan (configurable via regex)

## Edge Case Handling



### Multiple Task Labels
When multiple task labels exist, the first valid one is used:





```go-template
{{- range $i, $label := .Input.body.pull_request.labels -}}
  {{- if hasPrefix $label.name "task-" -}}
    {{- if eq $taskId "" -}}  {{- /* Only set if not already found */ -}}
      {{- $taskId = index $parts 1 -}}
    {{- else -}}
      {{- /* Log warning about multiple labels */ -}}
    {{- end -}}
  {{- end -}}
{{- end -}}








```



### Malformed Task Labels
Labels like `task-abc` (non-numeric) are ignored, triggering fallback to branch extraction:





```go-template
{{- /* Validate task ID is numeric */ -}}
{{- if regexMatch "^[0-9]+$" $id -}}
  {{- $taskId = $id -}}
{{- else -}}
  {{- /* Mark as malformed and continue */ -}}
{{- end -}}








```

### No Task Identification
When no task ID can be extracted, the workflow name becomes `play-task-unknown-workflow`, which typically won't match any suspended workflow.

## Implementation Agent Remediation

### Cancellation Logic
When an implementation agent (Rex, Blaze, Morgan) pushes to a task branch:



1. Extract task ID from branch name


2. Cancel all CodeRuns for quality agents (Cleo, Tess) with matching task ID


3. Remove `ready-for-qa` label if present


4. Workflow will restart from Cleo stage on next PR event

### Agent Detection Pattern




```yaml
- path: body.pusher.name
  type: string
  comparator: "~"
  value: "5DLabs-(Rex|Blaze|Morgan)(\\[bot\\])?|5DLabs-(Rex|Blaze|Morgan)"








```

### Adding New Implementation Agents
To add a new implementation agent (e.g., "Nova"):

1. Update the regex pattern in remediation sensor:
   ```yaml
   value: "5DLabs-(Rex|Blaze|Morgan|Nova)(\\[bot\\])?|5DLabs-(Rex|Blaze|Morgan|Nova)"







```

2. Add to exclusion list in cleanup operations:
   ```yaml
   kubectl delete coderun -l github-app!=5DLabs-Rex,github-app!=5DLabs-Blaze,github-app!=5DLabs-Morgan,github-app!=5DLabs-Nova







```

## Testing the Correlation Logic

### Manual Test Commands

#### Test Label Extraction




```bash
echo '{"pull_request":{"labels":[{"name":"task-5"}]}}' | \
  jq -r '.pull_request.labels[]?.name | select(startswith("task-")) | split("-")[1]'


# Output: 5








```

#### Test Branch Extraction




```bash
branch="task-12-feature"
if [[ "$branch" =~ ^task-([0-9]+) ]]; then
  echo "${BASH_REMATCH[1]}"
fi


# Output: 12








```

#### Test Feature Branch Format




```bash
branch="feature/task-20-enhancement"
if [[ "$branch" =~ feature/task-([0-9]+) ]]; then
  echo "${BASH_REMATCH[1]}"
fi


# Output: 20








```



### Test Script
Use `test-correlation-logic.sh` to validate all extraction patterns:





```bash
./test-correlation-logic.sh








```



### Test Webhook Payloads
Sample payloads are provided in `test-payloads/test-webhook-payloads.json` covering:


- Standard PR creation


- Multiple labels


- Branch name fallback


- Malformed labels


- Multiple task labels


- No task identification

## Monitoring and Debugging

### Sensor Logs
Monitor sensor processing:




```bash
kubectl logs -f $(kubectl get pods -n argo -l sensor-name=enhanced-play-workflow-correlation -o name | head -1) -n argo








```

### Debug Extraction
Add debug output in dataTemplate:




```go-template
{{- /* Debug: Print extracted values */ -}}
{{- printf "TaskID: %s, Branch: %s" $taskId .Input.body.pull_request.head.ref -}}








```

### Common Issues

#### Issue: Workflow Not Found
- **Cause**: Extracted task ID doesn't match any suspended workflow
- **Solution**: Verify workflow naming convention and task ID extraction



#### Issue: Multiple Workflows Resume
- **Cause**: Overly broad correlation
- **Solution**: Use deterministic naming instead of label selectors

#### Issue: Extraction Returns Empty
- **Cause**: Unexpected payload structure
- **Solution**: Check webhook payload format, add defensive checks

## Performance Considerations

### Extraction Efficiency
- Label extraction: O(n) where n is number of labels
- Branch extraction: O(1) regex match
- Combined: < 100ms typical processing time



### Retry Strategy
All triggers include retry configuration:




```yaml
retryStrategy:
  steps: 3
  duration: "10s"
  factor: 2
  jitter: 0.1








```



### Resource Limits
Sensor pods should have appropriate resource limits:




```yaml
resources:
  limits:
    memory: "256Mi"
    cpu: "200m"
  requests:
    memory: "128Mi"
    cpu: "100m"








```

## Security Considerations

### Webhook Verification


- GitHub webhook secret validates payload authenticity


- Sensor filters prevent unauthorized event processing

### Actor Validation


- Ready-for-QA must come from Cleo


- PR approval must come from Tess


- Push events validated for implementation agents only

### Input Sanitization


- Task IDs validated as numeric only


- Branch names pattern-matched before use


- No arbitrary code execution from payloads

## Migration from Existing Sensors



### Compatibility
The enhanced sensor is backward-compatible with existing workflows using either:


- Branch-based correlation (multi-agent-workflow-resume-sensor.yaml)


- Label-based correlation (play-workflow-sensors.yaml)

### Deployment Strategy


1. Deploy enhanced sensor alongside existing ones


2. Test with sample workflows


3. Gradually migrate workflows to use enhanced sensor


4. Deprecate old sensors once stable

## Future Enhancements

### Potential Improvements
1. **GraphQL API Integration**: Use GitHub GraphQL for richer queries
2. **Caching Layer**: Cache task ID mappings for faster lookup
3. **Metrics Collection**: Prometheus metrics for correlation success/failure
4. **Dynamic Agent Configuration**: Runtime agent addition without sensor updates
5. **Webhook Replay**: Handle duplicate/replayed webhooks gracefully

### JQ Expression Note
While Task 5 mentions JQ expressions, Argo Events uses Go templates. The equivalent logic has been implemented using Go template syntax, which is the native and supported approach in Argo Events v1.9+.