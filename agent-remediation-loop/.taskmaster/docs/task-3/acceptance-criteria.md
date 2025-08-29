# Task 3: Create Remediation Sensor and Trigger - Acceptance Criteria

## Overview
This document defines the acceptance criteria and test cases for Task 3, which implements the core remediation sensor that bridges QA feedback with automated Rex remediation. All criteria must be met for the task to be considered complete.

## Functional Acceptance Criteria

### AC-3.1: Sensor Deployment and Configuration
**Requirement**: Argo Events Sensor successfully deployed and operational

**Acceptance Criteria**:
- [ ] Sensor resource created with name `pr-comment-remediation` in `github-webhooks` namespace
- [ ] Sensor status shows `Running` state without errors
- [ ] Resource limits properly configured (128Mi memory request, 256Mi limit, 100m-200m CPU)
- [ ] ServiceAccount `argo-events-sa` properly configured and accessible
- [ ] EventBus connection established to `default` event bus
- [ ] Proper labels and annotations applied for monitoring and management

**Test Cases**:
```bash
# Verify deployment
kubectl get sensor pr-comment-remediation -n github-webhooks
kubectl describe sensor pr-comment-remediation -n github-webhooks

# Check resource usage
kubectl top pod -l app=remediation-sensor -n github-webhooks

# Verify connectivity
kubectl logs -l app=remediation-sensor -n github-webhooks --tail=50
```

**Expected Results**:
- Sensor resource exists and shows `Running` status
- Resource usage within defined limits
- No error logs related to configuration or connectivity
- EventBus connection established successfully

### AC-3.2: Event Filtering and Detection
**Requirement**: Sensor correctly identifies and processes QA feedback comments

**Acceptance Criteria**:
- [ ] Only processes `issue_comment` events with `action: created`
- [ ] Filters for comments containing `🔴 Required Changes` pattern
- [ ] Only accepts comments from authorized users (`5DLabs-Tess`, `5DLabs-Tess[bot]`)
- [ ] Validates PR context (ensures `pull_request.url` exists)
- [ ] Filters for open issues only (`issue.state == 'open'`)
- [ ] Ignores non-feedback comments and unauthorized users

**Test Cases**:
```yaml
# Test Case 1: Valid feedback comment
apiVersion: v1
kind: Event
metadata:
  name: test-feedback-comment
type: Normal
involvedObject:
  name: github-eventsource
data:
  body.action: "created"
  body.comment.body: "🔴 Required Changes: Fix the authentication bug"
  body.comment.user.login: "5DLabs-Tess"
  body.issue.state: "open"
  body.issue.pull_request.url: "https://api.github.com/repos/5dlabs/cto/pulls/123"

# Test Case 2: Should be ignored - wrong user
data:
  body.comment.user.login: "unauthorized-user"

# Test Case 3: Should be ignored - no feedback marker
data:
  body.comment.body: "Looks good to me!"
```

**Expected Results**:
- Valid feedback comments trigger sensor activation
- Invalid comments (wrong user, missing marker, closed PR) are ignored
- Sensor logs show proper filtering behavior
- No false positive triggers

### AC-3.3: Task ID Extraction
**Requirement**: Correctly extract task IDs from PR labels

**Acceptance Criteria**:
- [ ] JSONPath expression correctly parses labels array
- [ ] Extracts numeric task ID from `task-{number}` label format
- [ ] Handles missing task labels gracefully (sets to 'unknown')
- [ ] Supports multiple labels with only task-specific extraction
- [ ] Properly escapes and validates extracted task ID

**Test Cases**:
```yaml
# Test Case 1: Valid task label
body.issue.labels:
  - name: "bug"
  - name: "task-42"
  - name: "high-priority"
# Expected: task-id = "42"

# Test Case 2: No task label
body.issue.labels:
  - name: "bug"
  - name: "enhancement"
# Expected: task-id = "unknown"

# Test Case 3: Multiple task labels (edge case)
body.issue.labels:
  - name: "task-1"
  - name: "task-42"  
# Expected: task-id = "42" (last match)
```

**Expected Results**:
- Task ID "42" extracted from "task-42" label
- Unknown task scenarios handled without errors
- CodeRun resources created with correct task-id label
- Parameter mapping works correctly in all cases

### AC-3.4: CodeRun Resource Generation
**Requirement**: Generate proper CodeRun resources for Rex remediation

**Acceptance Criteria**:
- [ ] Creates CodeRun with API version `cto.5dlabs.com/v1alpha1`
- [ ] Uses `generateName: remediation-rex-` for unique naming
- [ ] Sets `github_app: "5DLabs-Rex"` for proper agent identification
- [ ] Configures `remediation_mode: true` flag
- [ ] Sets `continue_session: true` for session persistence
- [ ] Applies correct labels: `task-id`, `trigger-type: comment-feedback`, `agent-type: rex`
- [ ] Adds annotations with PR number, comment ID, and iteration info

**Test Cases**:
```bash
# Trigger remediation with test comment
curl -X POST http://webhook-endpoint/github \
  -H "Content-Type: application/json" \
  -d @test/sample-feedback-webhook.json

# Verify CodeRun creation
kubectl get coderuns -l trigger-type=comment-feedback -n agent-platform -o yaml

# Check resource structure
kubectl describe coderun remediation-rex-abc123 -n agent-platform
```

**Expected Results**:
- CodeRun resource created with correct structure
- All required fields populated from webhook parameters
- Labels and annotations properly set
- Resource creation completes without errors

### AC-3.5: Environment Variable Configuration
**Requirement**: Pass required environment variables to Rex container

**Acceptance Criteria**:
- [ ] `REMEDIATION_MODE` set to "true"
- [ ] `FEEDBACK_COMMENT_ID` populated with comment ID from webhook
- [ ] `ITERATION_COUNT` populated from state management system
- [ ] `MAX_ITERATIONS` set to "10" for limit enforcement
- [ ] Environment variables properly templated and substituted

**Test Cases**:
```bash
# Check generated CodeRun environment variables
kubectl get coderun remediation-rex-abc123 -n agent-platform -o jsonpath='{.spec.env}'

# Verify Rex container receives variables
kubectl logs -l app=rex,coderun=remediation-rex-abc123 -n agent-platform | grep -E "(REMEDIATION_MODE|FEEDBACK_COMMENT_ID)"
```

**Expected Results**:
- All environment variables present in CodeRun spec
- Rex container logs show variables are properly set
- Comment ID matches the triggering webhook comment
- Iteration count reflects current remediation cycle

### AC-3.6: State Management Integration
**Requirement**: Integrate with ConfigMap-based state tracking

**Acceptance Criteria**:
- [ ] Creates/updates ConfigMap named `task-{id}-remediation-state`
- [ ] Increments iteration counter on each remediation cycle
- [ ] Tracks `last_comment_id` to prevent duplicate processing
- [ ] Respects `max_iterations` limit (default: 10)
- [ ] Updates remediation status and timestamps
- [ ] Handles concurrent access to state ConfigMaps

**Test Cases**:
```bash
# Check state ConfigMap creation
kubectl get configmap task-42-remediation-state -n agent-platform -o yaml

# Trigger multiple remediation cycles
for i in {1..3}; do
  curl -X POST http://webhook-endpoint/github -d @test/feedback-comment-$i.json
  sleep 5
done

# Verify iteration increment
kubectl get configmap task-42-remediation-state -n agent-platform -o jsonpath='{.data.current_iteration}'
```

**Expected Results**:
- ConfigMap created with task-specific naming
- Iteration counter increments correctly (1, 2, 3...)
- Last comment ID updated to prevent duplicates
- State remains consistent across multiple triggers

### AC-3.7: Agent Cancellation Integration
**Requirement**: Coordinate with existing implementation-agent-remediation sensor

**Acceptance Criteria**:
- [ ] Cancels running Cleo and Tess CodeRuns for the same task
- [ ] Updates PR labels to reflect remediation state change
- [ ] Removes `ready-for-qa` label when remediation starts
- [ ] Adds `remediation-in-progress` and `iteration-{n}` labels
- [ ] Coordinates state transitions without race conditions

**Test Cases**:
```bash
# Start quality agents for a task
kubectl apply -f test/cleo-coderun-task-42.yaml
kubectl apply -f test/tess-coderun-task-42.yaml

# Trigger remediation
curl -X POST http://webhook-endpoint/github -d @test/feedback-comment.json

# Verify cancellation
kubectl get coderuns -l task-id=42,agent-type=cleo -n agent-platform
kubectl get coderuns -l task-id=42,agent-type=tess -n agent-platform

# Check PR label updates
gh pr view 123 --json labels
```

**Expected Results**:
- Existing Cleo/Tess CodeRuns deleted within 30 seconds
- Rex CodeRun created successfully
- PR labels updated to reflect new state
- No orphaned or conflicting agent processes

### AC-3.8: Event Deduplication
**Requirement**: Prevent duplicate processing of the same comment

**Acceptance Criteria**:
- [ ] Uses comment ID for deduplication logic
- [ ] Prevents multiple CodeRuns from same comment edit
- [ ] Handles GitHub webhook retries gracefully
- [ ] Maintains deduplication state across sensor restarts
- [ ] Provides configurable deduplication time window

**Test Cases**:
```bash
# Send identical webhook payloads multiple times
for i in {1..5}; do
  curl -X POST http://webhook-endpoint/github -d @test/same-comment-webhook.json
  sleep 1
done

# Verify only one CodeRun created
kubectl get coderuns -l pr-number=123,comment-id=987654321 -n agent-platform
```

**Expected Results**:
- Only one CodeRun created despite multiple webhook deliveries
- Duplicate events logged but not processed
- No resource conflicts or race conditions
- Deduplication state persists across sensor restarts

## Performance Acceptance Criteria

### AC-3.9: Resource Utilization
**Requirement**: Sensor operates within defined resource constraints

**Acceptance Criteria**:
- [ ] Memory usage stays below 256Mi limit under normal load
- [ ] CPU usage stays below 200m under normal load  
- [ ] Processes events within 5 seconds of reception
- [ ] Handles up to 10 concurrent PR comments without degradation
- [ ] No memory leaks during extended operation (24h+)

**Test Cases**:
```bash
# Load test with concurrent events
for i in {1..20}; do
  curl -X POST http://webhook-endpoint/github -d @test/feedback-comment-$i.json &
done
wait

# Monitor resource usage
kubectl top pod -l app=remediation-sensor -n github-webhooks
kubectl describe pod -l app=remediation-sensor -n github-webhooks | grep -A 5 "Resource"
```

**Expected Results**:
- Resource usage within defined limits
- Event processing latency < 5 seconds
- No OOMKilled or CPU throttling events
- Consistent performance across load scenarios

### AC-3.10: Error Handling and Recovery
**Requirement**: Robust error handling and failure recovery

**Acceptance Criteria**:
- [ ] Gracefully handles malformed webhook payloads
- [ ] Retries failed CodeRun creation (up to 3 attempts)
- [ ] Recovers from temporary API server unavailability
- [ ] Logs errors without crashing sensor process
- [ ] Maintains state consistency during failure scenarios

**Test Cases**:
```bash
# Test with malformed JSON payload
curl -X POST http://webhook-endpoint/github -d '{"invalid": json}}'

# Test with missing required fields
curl -X POST http://webhook-endpoint/github -d @test/incomplete-webhook.json

# Simulate API server unavailability
kubectl delete pod -l app=kube-apiserver -n kube-system
# Wait and verify recovery
```

**Expected Results**:
- Sensor continues running despite malformed inputs
- Error events logged without sensor crashes
- Failed operations retried according to policy
- Service recovers automatically when dependencies restore

## Security Acceptance Criteria

### AC-3.11: Access Control and Authorization
**Requirement**: Proper security controls and access restrictions

**Acceptance Criteria**:
- [ ] Sensor runs with minimal RBAC permissions
- [ ] Only authorized GitHub users can trigger remediation
- [ ] Webhook signature validation (if configured)
- [ ] No sensitive data logged in plain text
- [ ] ConfigMap access restricted to appropriate namespaces

**Test Cases**:
```bash
# Test unauthorized user attempt
curl -X POST http://webhook-endpoint/github \
  -d @test/unauthorized-user-comment.json

# Verify RBAC restrictions
kubectl auth can-i create coderuns --as=system:serviceaccount:github-webhooks:argo-events-sa -n agent-platform

# Check for sensitive data in logs
kubectl logs -l app=remediation-sensor -n github-webhooks | grep -i "token\|secret\|password"
```

**Expected Results**:
- Unauthorized users' comments ignored
- RBAC permissions limited to required operations
- No sensitive data exposed in logs or events
- Webhook signature validation passes (if enabled)

## Integration Acceptance Criteria

### AC-3.12: End-to-End Workflow Integration
**Requirement**: Complete integration with remediation loop workflow

**Acceptance Criteria**:
- [ ] QA feedback triggers remediation automatically
- [ ] Rex receives proper context and feedback data
- [ ] Remediation cycles complete successfully
- [ ] State transitions work correctly throughout workflow
- [ ] Human override capabilities preserved

**Test Cases**:
```bash
# Complete end-to-end test
1. Create PR with task-42 label
2. Start Cleo quality review (should complete)
3. Start Tess QA testing
4. Post feedback comment with 🔴 Required Changes
5. Verify Rex remediation starts automatically
6. Verify Cleo/Tess cancellation
7. Wait for Rex completion and new push
8. Verify cycle repeats correctly

# Monitor full workflow
kubectl get events --sort-by='.lastTimestamp' -A | grep -E "(remediation|coderun|sensor)"
```

**Expected Results**:
- Complete workflow executes without manual intervention
- Each component receives proper signals and data
- State transitions occur in correct sequence
- Workflow can handle multiple iterations
- Manual override stops automation when needed

## Monitoring and Observability Criteria

### AC-3.13: Logging and Metrics
**Requirement**: Comprehensive monitoring and observability

**Acceptance Criteria**:
- [ ] Structured JSON logging with appropriate log levels
- [ ] Metrics exposed for Prometheus scraping
- [ ] Key events tracked (triggers, successes, failures)
- [ ] Performance metrics available (latency, throughput)
- [ ] Integration with existing monitoring stack

**Test Cases**:
```bash
# Check log structure and content
kubectl logs -l app=remediation-sensor -n github-webhooks | jq '.'

# Verify metrics endpoint
kubectl port-forward svc/remediation-sensor-metrics 9090:9090 -n github-webhooks
curl http://localhost:9090/metrics | grep remediation

# Check event creation
kubectl get events -n github-webhooks --field-selector reason=RemediationTriggered
```

**Expected Results**:
- Logs formatted as valid JSON with timestamps
- Metrics endpoint accessible and provides relevant data
- Events created for important state changes
- Integration with monitoring dashboard shows sensor status

## Documentation and Maintenance Criteria

### AC-3.14: Documentation Completeness
**Requirement**: Complete and accurate documentation

**Acceptance Criteria**:
- [ ] Technical implementation guide (task.md) comprehensive
- [ ] Autonomous prompt (prompt.md) provides clear instructions
- [ ] Acceptance criteria (this document) covers all scenarios
- [ ] XML prompt (task.xml) properly structured
- [ ] Troubleshooting guide includes common issues
- [ ] Architecture diagrams show integration points

**Validation**:
- All documentation files present and complete
- Examples and code snippets accurate and functional
- Troubleshooting steps verified through testing
- Integration points clearly documented

## Final Validation Checklist

### Pre-Production Deployment
- [ ] All acceptance criteria validated in test environment
- [ ] Performance testing completed under realistic load
- [ ] Security review passed
- [ ] Integration testing with all dependent components
- [ ] Rollback procedure tested and documented
- [ ] Monitoring and alerting configured
- [ ] Team training completed on new functionality

### Production Readiness
- [ ] Gradual rollout strategy defined and approved
- [ ] Emergency contact procedures established
- [ ] Post-deployment validation plan prepared
- [ ] Success metrics defined and monitoring configured
- [ ] Documentation reviewed and approved
- [ ] Change management process completed

This comprehensive set of acceptance criteria ensures that Task 3 delivers a robust, secure, and well-integrated remediation sensor that forms the core automation bridge in the Agent Remediation Loop system.