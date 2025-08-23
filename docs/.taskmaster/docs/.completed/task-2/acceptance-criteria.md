# Acceptance Criteria: Setup Argo Events Infrastructure

## Functional Requirements

### 1. Multi-Agent Workflow Resume Sensor


- [ ] Sensor deployed and operational in `argo` namespace


- [ ] Correctly processes GitHub PR creation webhook events


- [ ] Extracts task ID from PR labels using pattern `task-{id}`


- [ ] Validates branch name matches task ID format `task-{id}-*`
- [ ] Resumes workflows with label selector: `workflow-type=play-orchestration,task-id={id},current-stage=waiting-pr-created`


- [ ] Handles correlation failures with appropriate error logging


- [ ] Ignores PRs without task labels or mismatched branch names

### 2. Ready-for-QA Label Sensor


- [ ] Sensor deployed and operational in `argo` namespace


- [ ] Processes GitHub PR labeling webhook events
- [ ] Filters for specific action: `labeled` with label name: `ready-for-qa`


- [ ] Verifies label was added by `5DLabs-Cleo[bot]` or Cleo GitHub App


- [ ] Extracts task ID from PR labels for workflow correlation


- [ ] Resumes workflows in `waiting-ready-for-qa` stage


- [ ] Ignores label events from unauthorized actors

### 3. PR Approval Sensor


- [ ] Sensor deployed and operational in `argo` namespace


- [ ] Processes GitHub PR review webhook events
- [ ] Filters for action: `submitted` with review state: `approved`


- [ ] Verifies approval comes from `5DLabs-Tess[bot]` or Tess GitHub App


- [ ] Extracts task ID from PR metadata for correlation


- [ ] Resumes workflows in `waiting-pr-approved` stage


- [ ] Handles multiple approvals gracefully (idempotent)

### 4. Rex Remediation Sensor


- [ ] Sensor deployed and operational in `argo` namespace


- [ ] Processes GitHub push webhook events


- [ ] Filters for pushes by `5DLabs-Rex[bot]` to branches matching `refs/heads/task-*`


- [ ] Extracts task ID from branch name for correlation
- [ ] Cancels running CodeRun CRDs with labels: `task-id={id},github-app!=5DLabs-Rex`


- [ ] Removes `ready-for-qa` labels from associated PRs


- [ ] Resets workflow stage to appropriate restart point


- [ ] Logs cancellation actions for audit trail

## Technical Requirements

### Event Source Integration


- [ ] All sensors use existing `github` EventSource


- [ ] All sensors connect to existing `argo` EventBus


- [ ] Sensor configurations follow established patterns from `github-demo-sensor.yaml`


- [ ] No modifications required to existing EventSource or EventBus

### Webhook Processing


- [ ] Robust JSON field extraction using `jq` expressions


- [ ] Handles webhook payload variations gracefully


- [ ] Validates webhook signatures (if implemented in EventSource)


- [ ] Processes events idempotently to handle duplicates

### Workflow Correlation
- [ ] Task ID extraction pattern: `.pull_request.labels[?(@.name | startswith("task-"))].name | split("-")[1]`
- [ ] Branch validation pattern: extract task ID from `refs/heads/task-{id}-*`


- [ ] Multi-method validation ensures PR labels and branch names match


- [ ] Workflow targeting uses precise label selectors

### Error Handling


- [ ] Comprehensive logging for all event processing steps


- [ ] Error alerts for correlation failures


- [ ] Graceful handling of missing workflows or resources


- [ ] Retry logic for transient failures



## Test Cases

### Test Case 1: Multi-Agent Workflow Resumption
**Objective**: Validate PR creation triggers workflow resumption

**Setup**:


1. Deploy multi-agent workflow resume sensor
2. Create suspended test workflow with labels: `workflow-type=play-orchestration,task-id=test,current-stage=waiting-pr-created`

**Steps**:


1. Create GitHub PR with label `task-test` on branch `task-test-feature`


2. Monitor sensor logs for event processing


3. Verify workflow resumes from suspended state


4. Confirm workflow stage updates correctly

**Expected Result**: Workflow resumes successfully with proper correlation

### Test Case 2: Ready-for-QA Label Processing
**Objective**: Validate Cleo completion signals trigger Tess stage

**Setup**:


1. Deploy ready-for-QA label sensor


2. Create suspended workflow in `waiting-ready-for-qa` stage


3. Create test PR with `task-test` label

**Steps**:


1. Add `ready-for-qa` label to PR (simulate Cleo action)


2. Monitor sensor processing


3. Verify workflow correlation and resumption


4. Confirm only Cleo-added labels are processed

**Expected Result**: Workflow resumes Tess stage only for authorized label additions

### Test Case 3: PR Approval Processing
**Objective**: Validate Tess approvals trigger workflow completion

**Setup**:


1. Deploy PR approval sensor


2. Create suspended workflow in `waiting-pr-approved` stage


3. Create test PR with `task-test` label

**Steps**:


1. Submit PR approval from Tess GitHub App


2. Monitor sensor event processing


3. Verify workflow resumption


4. Test unauthorized approval rejection

**Expected Result**: Workflow resumes completion stage only for Tess approvals

### Test Case 4: Rex Remediation Pipeline
**Objective**: Validate Rex push events cancel and restart QA pipeline

**Setup**:


1. Deploy Rex remediation sensor


2. Create running Cleo and Tess CodeRun CRDs with `task-id=test`


3. Create test PR with `ready-for-qa` label

**Steps**:


1. Simulate Rex push to `task-test-feature` branch


2. Monitor sensor processing and cancellation actions


3. Verify CodeRun CRDs are deleted


4. Confirm `ready-for-qa` label removal


5. Validate workflow stage reset

**Expected Result**: Running agents canceled, pipeline reset for fresh QA cycle

### Test Case 5: Event Correlation Validation
**Objective**: Test multi-method validation prevents false positives

**Setup**:


1. Deploy all sensors


2. Create test workflows and PRs

**Steps**:


1. Create PR with mismatched label (`task-1`) and branch (`task-2-feature`)


2. Verify correlation failure and no workflow resumption


3. Create PR with correct correlation


4. Verify successful processing

**Expected Result**: Only properly correlated events trigger actions

### Test Case 6: Rate Limiting and Error Handling
**Objective**: Validate sensor resilience under load

**Setup**:


1. Deploy all sensors


2. Configure monitoring and logging

**Steps**:


1. Generate high volume of webhook events


2. Monitor processing latency and success rates


3. Introduce error conditions (invalid payloads, missing workflows)


4. Verify error handling and recovery

**Expected Result**: Sensors handle load gracefully with appropriate error recovery



## Quality Criteria

### Deployment Standards


- [ ] All sensors pass `kubectl apply --dry-run=server`


- [ ] Sensor pods start successfully without errors


- [ ] Proper resource limits and requests configured


- [ ] RBAC permissions validated and minimal

### Operational Excellence


- [ ] Comprehensive structured logging for all events


- [ ] Metrics exported for monitoring (event counts, processing time)


- [ ] Alerting configured for sensor failures and correlation errors


- [ ] Documentation includes troubleshooting guide

### Security Requirements


- [ ] Webhook signature validation (if supported by EventSource)


- [ ] Actor verification prevents unauthorized event processing


- [ ] Minimal RBAC permissions for sensor operations


- [ ] Audit logging for all workflow modifications

## Performance Requirements

### Response Time


- [ ] Event processing latency < 5 seconds under normal load


- [ ] Workflow resumption triggered within 10 seconds of event


- [ ] CodeRun cancellation completes within 30 seconds



### Throughput


- [ ] Handle up to 100 webhook events per minute


- [ ] Support 10 concurrent active tasks without degradation


- [ ] Process overlapping events correctly without conflicts



### Resource Usage


- [ ] Sensor pods use < 100MB memory under normal operation


- [ ] CPU usage < 100m under typical webhook volume


- [ ] Horizontal scaling possible if needed



## Deliverable Checklist



- [ ] `multi-agent-workflow-resume-sensor.yaml` deployed


- [ ] `ready-for-qa-label-sensor.yaml` deployed


- [ ] `pr-approval-sensor.yaml` deployed


- [ ] `rex-remediation-sensor.yaml` deployed


- [ ] All sensors visible in `kubectl get sensors -n argo`


- [ ] Comprehensive test suite executed with all cases passing


- [ ] Monitoring and alerting configured


- [ ] Documentation complete with troubleshooting guide



## Success Metrics

1. **Functional Completeness**: 100% of event types processed correctly
2. **Reliability**: 99.9% event processing success rate
3. **Performance**: <5 second average event processing latency
4. **Integration**: Zero disruption to existing Argo Events infrastructure
5. **Validation**: All test cases pass consistently



## Notes



- This task builds upon existing Argo Events infrastructure without modifications


- Focus on robust correlation logic to prevent false positives


- Implement comprehensive logging for operational visibility


- Consider GitHub API rate limiting in design


- Ensure backward compatibility with existing workflows