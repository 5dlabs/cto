# Acceptance Criteria: Create GitHub Webhook Correlation Logic (Aligned with Supported Argo Events Patterns)

## Functional Requirements

### 1. JQ Expression Implementation


- [ ] Task ID extraction from PR labels implemented


- [ ] Label format "task-X" correctly parsed


- [ ] Multiple labels handled with proper selection


- [ ] Branch name fallback extraction working


- [ ] Edge cases handled gracefully

### 2. Correlation Mechanism


- [ ] Deterministic workflow name targeting used (no templated labelSelector)
- [ ] Workflow name format: `play-task-{{task-id}}-workflow`


- [ ] Resume operation targets workflow by `metadata.name`


- [ ] Stage awareness handled within workflow logic (labels may be updated post-resume)


- [ ] No false positive correlations

### 3. Event Type Handling


- [ ] PR opened events trigger correct stage


- [ ] PR labeled events (ready-for-qa) handled


- [ ] PR review approved events processed


- [ ] Push events trigger remediation logic


- [ ] All event filters properly configured

### 4. Error Handling


- [ ] Missing labels handled with fallback


- [ ] Malformed task IDs rejected gracefully


- [ ] Duplicate labels resolved correctly


- [ ] Network failures handled with retry


- [ ] Webhook replay scenarios supported

## Technical Requirements

### JQ Expression Validation
- [ ] Primary extraction expression tested:
  ```jq
  .pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]





```
- [ ] Fallback branch extraction tested:
  ```jq
  .pull_request.head.ref | capture("^task-(?<id>[0-9]+)") | .id





```


- [ ] Empty result handling implemented


- [ ] Multiple matches resolved to single ID

### Deterministic Name Targeting (No labelSelector templating)


- [ ] Workflow name constructed as `play-task-{{extracted-task-id}}-workflow`
- [ ] Argo Events trigger sets:
  ```yaml
  parameters:
    - src:
        dependencyName: <github-event-dep>
        dataTemplate: |
          play-task-{{jq '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}-workflow
      dest: argoWorkflow.source.resource.metadata.name





```


- [ ] Strictly avoid templating within `labelSelector` fields



## Test Cases

### Test Case 1: Standard PR Creation
**Objective**: Verify task ID extraction from PR with single task label

**Input**:



```json
{
  "action": "opened",
  "pull_request": {
    "labels": [{"name": "task-5"}],
    "head": {"ref": "task-5-webhook-correlation"}
  }
}






```

**Expected**:


- Extracts task ID "5"


- Constructs workflow name `play-task-5-workflow`


- Resumes workflow by metadata.name



### Test Case 2: Multiple Labels
**Objective**: Handle PR with multiple labels including task label

**Input**:



```json
{
  "pull_request": {
    "labels": [
      {"name": "enhancement"},
      {"name": "task-8"},
      {"name": "priority-high"}
    ]
  }
}






```

**Expected**:


- Correctly extracts "8" from task-8 label


- Targets `play-task-8-workflow` by name

### Test Case 3: Branch Name Fallback
**Objective**: Extract task ID from branch when label missing

**Input**:



```json
{
  "pull_request": {
    "labels": [],
    "head": {"ref": "task-12-implement-feature"}
  }
}






```

**Expected**:


- Falls back to branch extraction


- Extracts task ID "12"


- Targets `play-task-12-workflow` by name

### Test Case 4: Ready-for-QA Label Event
**Objective**: Handle PR labeling for Cleo completion

**Input**:



```json
{
  "action": "labeled",
  "label": {"name": "ready-for-qa"},
  "pull_request": {
    "labels": [{"name": "task-3"}, {"name": "ready-for-qa"}]
  }
}






```

**Expected**:


- Detects ready-for-qa label addition


- Targets `play-task-3-workflow` by name

### Test Case 5: PR Approval Event
**Objective**: Handle PR review approval from Tess

**Input**:



```json
{
  "action": "submitted",
  "review": {
    "state": "approved",
    "user": {"login": "5DLabs-Tess[bot]"}
  },
  "pull_request": {
    "labels": [{"name": "task-7"}]
  }
}






```

**Expected**:


- Detects approval from Tess


- Targets `play-task-7-workflow` by name

### Test Case 6: Rex Push Remediation
**Objective**: Detect Rex push and trigger remediation

**Input**:



```json
{
  "pusher": {"name": "5DLabs-Rex[bot]"},
  "ref": "refs/heads/task-9-fix-issues"
}






```

**Expected**:


- Identifies Rex as pusher


- Extracts task ID "9"


- Cancels/resubmits `play-task-9-workflow` as remediation



### Test Case 7: Malformed Task Label
**Objective**: Handle invalid task label format

**Input**:



```json
{
  "pull_request": {
    "labels": [{"name": "task-abc"}]
  }
}






```

**Expected**:


- Extraction fails gracefully


- Falls back to branch name


- Logs warning about malformed label

### Test Case 8: Concurrent Workflows
**Objective**: Target correct workflow among multiple suspended

**Setup**:
- Three suspended workflows by name: `play-task-1-workflow`, `play-task-2-workflow`, `play-task-3-workflow`

**Input**: PR created event with label "task-2"

**Expected**:


- Only `play-task-2-workflow` resumed


- Other workflows remain suspended

## Performance Criteria



- [ ] JQ extraction completes in < 100ms


- [ ] Workflow correlation in < 500ms


- [ ] Handles 10 concurrent events without errors


- [ ] No memory leaks in sensor pods


- [ ] Webhook processing queue doesn't backlog

## Security Requirements



- [ ] Webhook signatures verified


- [ ] No arbitrary code execution from payloads


- [ ] Task IDs validated as integers only


- [ ] No SQL injection in parameters


- [ ] Rate limiting enforced

## Monitoring & Observability



- [ ] Correlation success/failure metrics exposed


- [ ] Extraction errors logged with context


- [ ] Webhook processing latency tracked


- [ ] Failed correlations generate alerts


- [ ] Debugging logs include full payload

## Documentation Requirements



- [ ] JQ expressions documented with examples


- [ ] Correlation logic flow diagram created


- [ ] Troubleshooting guide for common issues


- [ ] Event type mapping reference table


- [ ] Configuration parameters documented



## Success Metrics

1. **Accuracy**: 100% correct task ID extraction
2. **Reliability**: 99.9% correlation success rate
3. **Performance**: < 1 second total processing time
4. **Robustness**: Zero false positive correlations
5. **Maintainability**: Clear logs for debugging