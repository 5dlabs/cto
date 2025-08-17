# Argo Events Sensors for Multi-Agent Workflow Orchestration

## Overview

This directory contains four specialized Argo Events Sensors that enable event-driven coordination between Rex, Cleo, and Tess agents through GitHub webhook processing and workflow resumption mechanisms.

## Deployed Sensors

### 1. Multi-Agent Workflow Resume Sensor (`multi-agent-workflow-resume-sensor.yaml`)

**Purpose**: Handles PR creation events to resume workflows after Rex completes implementation.

**Triggers On**:
- GitHub pull_request events with action "opened"
- PR must have a task label (e.g., `task-3`)

**Actions**:
- Extracts task ID from PR labels
- Validates branch name matches task pattern
- Resumes workflows with labels:
  - `workflow-type=play-orchestration`
  - `task-id={extracted-id}`
  - `current-stage=waiting-pr-created`

### 2. Ready-for-QA Label Sensor (`ready-for-qa-label-sensor.yaml`)

**Purpose**: Detects when Cleo adds "ready-for-qa" label to trigger Tess testing phase.

**Triggers On**:
- GitHub pull_request events with action "labeled"
- Label name must be "ready-for-qa"
- Label must be added by `5DLabs-Cleo[bot]` or `5DLabs-Cleo`

**Actions**:
- Extracts task ID from PR labels
- Resumes workflows with labels:
  - `workflow-type=play-orchestration`
  - `task-id={extracted-id}`
  - `current-stage=waiting-ready-for-qa`

### 3. PR Approval Sensor (`pr-approval-sensor.yaml`)

**Purpose**: Processes PR review approval events to trigger workflow completion.

**Triggers On**:
- GitHub pull_request_review events with action "submitted"
- Review state must be "approved"
- Approval must come from `5DLabs-Tess[bot]` or `5DLabs-Tess`

**Actions**:
- Extracts task ID from PR metadata
- Resumes workflows with labels:
  - `workflow-type=play-orchestration`
  - `task-id={extracted-id}`
  - `current-stage=waiting-pr-approved`

### 4. Rex Remediation Sensor (`rex-remediation-sensor.yaml`)

**Purpose**: Detects Rex push events to cancel running agents and restart QA pipeline.

**Triggers On**:
- GitHub push events
- Push must be from `5DLabs-Rex[bot]` or `5DLabs-Rex`
- Branch must match pattern `task-*`

**Actions**:
- Extracts task ID from branch name
- Deletes CodeRun CRDs with labels:
  - `task-id={extracted-id}`
  - `github-app!=5DLabs-Rex`
- Would also remove "ready-for-qa" labels and reset workflow state (additional configuration needed)

## Infrastructure Dependencies

### EventSource
- **Name**: `github`
- **Namespace**: `argo`
- **Webhook Endpoint**: `http://github-eventsource-svc.argo:12000/github/webhook`
- **Organizations**: `5dlabs`

### EventBus
- **Name**: `default`
- **Namespace**: `argo`
- **Type**: NATS
- **URL**: `nats://eventbus-default-stan-svc:4222`

### Service Account
- **Name**: `argo-events-sa`
- **Namespace**: `argo`
- **Permissions**: Required for workflow operations and CRD management

## Testing

### Verification Commands

```bash
# Check sensor status
kubectl get sensors -n argo

# Check sensor pods
kubectl get pods -n argo --selector controller=sensor-controller

# View sensor logs
kubectl logs -f $(kubectl get pods -n argo -l sensor-name=multi-agent-workflow-resume -o name | head -1) -n argo

# Use test script
./test-sensors.sh
```

### Test Scenarios

1. **PR Creation Test**:
   - Create PR with `task-3` label on branch `task-3-feature`
   - Verify multi-agent-workflow-resume sensor processes event
   - Check workflow resumption

2. **Label Addition Test**:
   - Add `ready-for-qa` label as Cleo user
   - Verify ready-for-qa-label sensor processes event
   - Check workflow stage progression

3. **PR Approval Test**:
   - Approve PR as Tess user
   - Verify pr-approval sensor processes event
   - Check workflow completion trigger

4. **Rex Push Test**:
   - Push to `task-*` branch as Rex user
   - Verify rex-remediation sensor processes event
   - Check CodeRun CRD cancellation

## Webhook Payload Processing

### Task ID Extraction Pattern

```go-template
{{ range $index, $label := .Input.body.pull_request.labels }}
  {{ if hasPrefix "task-" $label.name }}
    {{ $parts := splitList "-" $label.name }}
    {{ if gt (len $parts) 1 }}
      {{ index $parts 1 }}
    {{ end }}
  {{ end }}
{{ end }}
```

### Branch Validation

```go-template
{{ $ref := .Input.body.ref }}
{{ if hasPrefix $ref "refs/heads/task-" }}
  {{ $branch := trimPrefix "refs/heads/task-" $ref }}
  {{ $parts := splitList "-" $branch }}
  {{ if gt (len $parts) 0 }}
    {{ index $parts 0 }}
  {{ end }}
{{ end }}
```

## Workflow Label Selectors

Workflows must have appropriate labels for sensor correlation:

```yaml
metadata:
  labels:
    workflow-type: play-orchestration
    task-id: "3"
    current-stage: waiting-pr-created  # or waiting-ready-for-qa, waiting-pr-approved
```

## Troubleshooting

### Common Issues

1. **Sensor Not Processing Events**:
   - Check EventSource is receiving webhooks
   - Verify webhook filters match expected values
   - Check sensor pod logs for errors

2. **Workflow Not Resuming**:
   - Verify workflow has correct labels
   - Check task ID extraction is working
   - Ensure workflow is in suspended state

3. **Actor Verification Failures**:
   - Confirm GitHub App names match exactly
   - Check webhook sender.login field
   - Verify bot suffixes ([bot]) are handled

### Debug Commands

```bash
# Check EventSource webhook delivery
kubectl logs -f $(kubectl get pods -n argo -l eventsource-name=github -o name | head -1) -n argo

# Check EventBus connectivity
kubectl logs -f $(kubectl get pods -n argo -l controller=eventbus-controller -o name | head -1) -n argo

# Describe sensor for status
kubectl describe sensor multi-agent-workflow-resume -n argo

# Get sensor configuration
kubectl get sensor multi-agent-workflow-resume -n argo -o yaml
```

## Maintenance

### Updating Sensors

1. Edit sensor YAML file
2. Apply changes: `kubectl apply -f sensor-file.yaml`
3. Verify pod restart: `kubectl get pods -n argo -w`
4. Check logs for errors

### Monitoring

- Set up alerts for sensor pod restarts
- Monitor webhook processing latency
- Track correlation success/failure rates
- Watch for GitHub API rate limiting

## Security Considerations

1. **Webhook Verification**: EventSource should verify GitHub webhook signatures
2. **Actor Validation**: Sensors verify events come from expected GitHub Apps
3. **RBAC**: Service account has minimal required permissions
4. **Namespace Isolation**: All resources deployed in `argo` namespace

## Future Enhancements

1. Add metrics for sensor processing
2. Implement webhook signature validation
3. Add support for PR comment triggers
4. Enhance error handling and retry logic
5. Add support for multiple repository configurations