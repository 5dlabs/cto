# Acceptance Criteria: Task Association Validation System

## Primary Validation Requirements

### ✅ PR Label Extraction
- [ ] System extracts task ID from PR labels using pattern `task-{id}`
- [ ] JQ expression correctly processes webhook payload: `.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]`
- [ ] Returns first matching label when multiple task labels exist
- [ ] Returns empty string when no task labels found
- [ ] Handles malformed labels gracefully (e.g., `task-abc`, `task-`, `task`)

### ✅ Branch Name Parsing
- [ ] Regex `^(?:feature/)?task-(\d+)(?:-.*)?$` correctly extracts task IDs
- [ ] Supports pattern `task-26-description` → extracts `26`
- [ ] Supports pattern `feature/task-26` → extracts `26`
- [ ] Supports pattern `task-26` → extracts `26`
- [ ] Rejects invalid patterns: `feature/task`, `task-abc-def`, `bugfix/task-26`
- [ ] Extracts from `pull_request.head.ref` field in webhook payload

### ✅ Marker File Processing
- [ ] Reads `docs/.taskmaster/current-task.json` successfully
- [ ] Extracts `task_id` field from valid JSON structure
- [ ] Handles missing marker file without failing workflow
- [ ] Handles corrupted JSON gracefully
- [ ] Validates JSON schema before extraction

## Validation Logic Requirements

### ✅ Agreement Enforcement
- [ ] Workflow fails if any two non-empty task IDs disagree
- [ ] Workflow succeeds when all non-empty task IDs match
- [ ] Workflow fails if all three methods return empty/invalid IDs
- [ ] Validation passes with single method if others are empty
- [ ] System logs detailed comparison results

### ✅ Test Scenarios
**All Methods Agree (PASS)**
- [ ] Label: `task-26`, Branch: `task-26-fix-bug`, Marker: `{"task_id": "26"}` → ✅ Pass
- [ ] Label: `task-15`, Branch: `feature/task-15`, Marker: `{"task_id": "15"}` → ✅ Pass

**Method Disagreement (FAIL)**
- [ ] Label: `task-26`, Branch: `task-27-fix`, Marker: `{"task_id": "26"}` → ❌ Fail
- [ ] Label: `task-10`, Branch: `task-10-update`, Marker: `{"task_id": "12"}` → ❌ Fail

**Missing Methods (Conditional)**
- [ ] Label: `task-26`, Branch: `invalid-branch`, Marker: missing → ✅ Pass (label only)
- [ ] Label: none, Branch: `task-15-fix`, Marker: `{"task_id": "15"}` → ✅ Pass (branch+marker)
- [ ] Label: none, Branch: `invalid-branch`, Marker: missing → ❌ Fail (no valid IDs)

## Error Handling Requirements

### ✅ GitHub Integration
- [ ] Creates GitHub PR comment on validation failure
- [ ] Comment includes specific error details and required actions
- [ ] Comment template provides clear fix instructions
- [ ] Uses proper GitHub API authentication (token from secret)
- [ ] Handles API failures gracefully (doesn't fail workflow)

### ✅ Retry Logic
- [ ] Retries failed validations up to 3 times
- [ ] Uses exponential backoff: 30s, 1m, 2m
- [ ] Distinguishes between retryable and permanent failures
- [ ] Logs retry attempts with structured data
- [ ] Fails permanently after max retries

### ✅ Error Messages
- [ ] Provides specific details for each validation failure type
- [ ] Includes all extracted task IDs in error reports
- [ ] Suggests concrete remediation steps
- [ ] Uses consistent error message format
- [ ] Logs errors with appropriate severity levels

## Marker File Management

### ✅ File Creation
- [ ] Creates marker file with correct JSON schema
- [ ] Includes required fields: `task_id`, `started_at`, `agent`, `workflow_id`, `commit_sha`
- [ ] Uses ISO-8601 timestamp format for `started_at`
- [ ] Commits marker file to repository with descriptive message
- [ ] Creates file only after successful validation

### ✅ File Schema Validation
```json
{
  "task_id": "string (required)",
  "started_at": "ISO-8601 timestamp (required)",
  "agent": "string (required)",
  "workflow_id": "string (required)", 
  "commit_sha": "string (required)",
  "pr_number": "string (optional)"
}
```
- [ ] All required fields present and properly typed
- [ ] Timestamp validates as ISO-8601 format
- [ ] Task ID matches validation result
- [ ] File permissions restrict write access

## Integration Requirements

### ✅ Argo Workflows Integration
- [ ] Sensor correctly triggers on PR webhook events
- [ ] WorkflowTemplate executes validation pipeline
- [ ] Parameters passed correctly between workflow steps
- [ ] Conditional step execution based on validation results
- [ ] Workflow status updates reflect validation state

### ✅ Monitoring Integration
- [ ] Emits Prometheus metrics for validation attempts
- [ ] Metrics include status labels: `success`, `failure`, `retry`
- [ ] Tracks validation latency and error rates
- [ ] Integrates with existing Grafana dashboards
- [ ] Provides alerting for validation failures

## Performance Requirements

### ✅ Response Times
- [ ] Validation completes within 30 seconds for normal cases
- [ ] Network timeouts configured appropriately (5s for API calls)
- [ ] Workflow fails fast on permanent errors
- [ ] Resource usage remains within defined limits

### ✅ Scalability
- [ ] Handles concurrent validation requests
- [ ] Scales with cluster auto-scaling policies
- [ ] Maintains performance under load (100+ concurrent workflows)
- [ ] Implements proper resource requests/limits

## Security Requirements

### ✅ Authentication & Authorization
- [ ] GitHub token stored in Kubernetes secret
- [ ] Service account uses least-privilege permissions
- [ ] No sensitive data logged in plain text
- [ ] Input sanitization prevents injection attacks

### ✅ Data Protection
- [ ] Marker file permissions prevent unauthorized access
- [ ] Audit logs capture all validation operations
- [ ] No secrets exposed in workflow annotations
- [ ] Secure handling of webhook payloads

## Testing Requirements

### ✅ Unit Tests
- [ ] Test task ID extraction from all three methods
- [ ] Test validation logic with various ID combinations
- [ ] Test error handling paths
- [ ] Test marker file creation and schema validation
- [ ] Achieve 95% code coverage

### ✅ Integration Tests
- [ ] Submit test PRs with various label/branch combinations
- [ ] Verify end-to-end workflow execution
- [ ] Test GitHub API integration
- [ ] Validate monitoring metric collection
- [ ] Test concurrent workflow scenarios

### ✅ Load Testing
- [ ] Process 100 concurrent validation requests
- [ ] Maintain <30s response time under load
- [ ] Verify no resource leaks or memory issues
- [ ] Test error recovery under stress

## Documentation Requirements

### ✅ Technical Documentation
- [ ] Architecture diagrams show validation flow
- [ ] API documentation for webhook integration
- [ ] Configuration examples for all components
- [ ] Troubleshooting guide for common issues

### ✅ Operational Documentation
- [ ] Monitoring and alerting setup guide
- [ ] Incident response playbooks
- [ ] Performance tuning recommendations
- [ ] Security configuration checklist

## Deployment Requirements

### ✅ Production Readiness
- [ ] All components deployed via GitOps
- [ ] Monitoring and alerting configured
- [ ] Backup and recovery procedures documented
- [ ] Security scanning passes (container and code)
- [ ] Load testing completed successfully

### ✅ Rollback Plan
- [ ] Rollback procedure documented and tested
- [ ] Feature flags enable quick disable if needed
- [ ] Monitoring detects issues requiring rollback
- [ ] Database migrations are reversible (if applicable)