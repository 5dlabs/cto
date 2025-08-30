# Acceptance Criteria: Task 1 - Setup GitHub Webhook Infrastructure

## Functional Requirements

### ✅ Sensor Creation and Deployment
- [ ] Argo Events Sensor resource created with name `pr-comment-remediation`
- [ ] Sensor deployed to `argo-events` namespace
- [ ] Sensor pod running without errors or restarts
- [ ] Sensor successfully connected to existing `github-eventsource`
- [ ] Sensor appears in Argo Events controller logs as active

### ✅ Event Detection and Filtering
- [ ] Sensor triggers only on `issue_comment` events with `created` action
- [ ] Comments without '🔴 Required Changes' marker are ignored
- [ ] Comments on issues (not PRs) are filtered out
- [ ] Only authorized users' comments trigger remediation (5DLabs-Tess, authorized reviewers)
- [ ] Closed or merged PRs' comments are ignored

### ✅ Data Extraction and Processing
- [ ] Task ID correctly extracted from PR labels (format: `task-{number}`)
- [ ] PR number accurately captured from webhook payload
- [ ] Comment ID properly extracted for feedback reference
- [ ] Comment author username captured for audit trail
- [ ] Iteration count initialized or incremented correctly

### ✅ CodeRun Resource Generation
- [ ] CodeRun CRD created in `agent-platform` namespace
- [ ] Resource name follows pattern: `rex-remediation-{hash}`
- [ ] All required labels applied:
  - `task-id`: Extracted task identifier
  - `pr-number`: Pull request number
  - `trigger-type`: Set to "comment-feedback"
  - `iteration`: Current remediation iteration
- [ ] CodeRun spec properly configured:
  - `service`: Set to "task{taskId}"
  - `github_app`: Set to "5DLabs-Rex"
  - `pr_number`: Correct PR number
  - `pr_comment_id`: Comment ID for context
  - `continue_session`: Set to true

### ✅ Environment Configuration
- [ ] `REMEDIATION_MODE` environment variable set to "true"
- [ ] `FEEDBACK_COMMENT_ID` contains correct comment ID
- [ ] `ITERATION_COUNT` properly tracks remediation cycles
- [ ] All environment variables accessible to Rex agent container

## Integration Requirements

### ✅ Existing Infrastructure Compatibility
- [ ] No conflicts with existing sensors (`play-workflow-sensors.yaml`, `implementation-agent-remediation`)
- [ ] Uses same service account as existing sensors (`argo-events-sa`)
- [ ] Leverages existing GitHub webhook secret
- [ ] Compatible with current HTTPRoute configuration
- [ ] No disruption to existing webhook event processing

### ✅ RBAC and Permissions
- [ ] Sensor has permission to create CodeRun resources in `agent-platform` namespace
- [ ] Service account can read webhook events from EventSource
- [ ] Proper permissions for accessing GitHub API if needed
- [ ] No privilege escalation or security violations

## Performance Requirements

### ✅ Response Time
- [ ] Event processing latency < 5 seconds from webhook receipt
- [ ] CodeRun creation completed within 10 seconds of trigger
- [ ] No blocking of other sensor operations

### ✅ Resource Usage
- [ ] Sensor pod memory usage < 128Mi
- [ ] CPU usage < 200m under normal load
- [ ] No memory leaks during extended operation

### ✅ Reliability
- [ ] Sensor automatically recovers from transient failures
- [ ] No data loss during pod restarts
- [ ] Handles malformed webhook payloads gracefully
- [ ] Proper error logging for debugging

## Testing Validation

### ✅ Unit Tests
- [ ] Event filter logic tested with sample payloads
- [ ] Task ID extraction tested with various label formats
- [ ] CodeRun template generation validated
- [ ] Edge cases handled (missing fields, null values)

### ✅ Integration Tests
- [ ] End-to-end test with real GitHub PR comment
- [ ] Multiple concurrent comments processed correctly
- [ ] Different comment formats tested:
  - Standard feedback format
  - Feedback with code blocks
  - Multi-line feedback
  - Unicode characters in feedback
- [ ] PR label variations tested:
  - Single task label
  - Multiple labels including task
  - No task label (should not trigger)

### ✅ Negative Test Cases
- [ ] Non-feedback comments ignored
- [ ] Comments from unauthorized users filtered
- [ ] Malformed task labels handled gracefully
- [ ] Comments on closed PRs don't trigger remediation
- [ ] System remains stable under invalid input

## Documentation and Monitoring

### ✅ Documentation
- [ ] Sensor configuration documented in README
- [ ] Troubleshooting guide created
- [ ] Event flow diagram updated
- [ ] RBAC requirements documented

### ✅ Monitoring and Observability
- [ ] Metrics exposed for:
  - Total events processed
  - Successful CodeRun creations
  - Filter match rate
  - Processing latency
- [ ] Structured logging implemented
- [ ] Error conditions properly logged
- [ ] Alerts configured for sensor failures

## Security Requirements

### ✅ Access Control
- [ ] Only authorized comment authors can trigger remediation
- [ ] No exposure of sensitive data in logs
- [ ] Webhook secret properly secured
- [ ] No arbitrary code execution vulnerabilities

### ✅ Input Validation
- [ ] All user input sanitized before processing
- [ ] Regex patterns prevent ReDoS attacks
- [ ] JSON path expressions validated
- [ ] Resource names properly escaped

## Rollback Plan

### ✅ Rollback Capability
- [ ] Sensor can be safely deleted without affecting other components
- [ ] Previous sensor versions can be restored
- [ ] No persistent state that would block rollback
- [ ] Clear rollback procedure documented

## Definition of Done

This task is considered complete when:
1. All acceptance criteria marked as complete (✅)
2. Sensor successfully processes production PR comments
3. At least 5 successful remediation triggers demonstrated
4. No critical issues in 24-hour stability test
5. Documentation reviewed and approved
6. Security review passed
7. Performance benchmarks met
8. Rollback procedure tested

## Test Scenarios

### Scenario 1: Standard Feedback Comment
**Given**: A PR with label `task-42` exists
**When**: Tess posts a comment with '🔴 Required Changes' and bug description
**Then**: CodeRun created with REMEDIATION_MODE=true and task-42 reference

### Scenario 2: Multiple Simultaneous Comments
**Given**: Multiple PRs with different task labels
**When**: Feedback comments posted on all PRs within 1 second
**Then**: All comments processed and respective CodeRuns created

### Scenario 3: Invalid Comment Format
**Given**: A PR with task label
**When**: Comment posted without feedback marker
**Then**: No CodeRun created, event logged as filtered

### Scenario 4: Unauthorized User Comment
**Given**: A PR with task label and valid feedback format
**When**: Unauthorized user posts feedback comment
**Then**: Comment ignored, security event logged

### Scenario 5: System Recovery
**Given**: Sensor pod crashes during event processing
**When**: Pod restarts automatically
**Then**: New events processed correctly, no data corruption