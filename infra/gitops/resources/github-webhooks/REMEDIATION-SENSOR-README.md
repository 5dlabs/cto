# Remediation Feedback Sensor

This document describes the remediation feedback sensor that automatically triggers CodeRun resources when QA feedback is posted on pull requests.

## Overview

The `remediation-feedback-sensor` is an Argo Events Sensor that:

1. **Monitors** GitHub webhook events for PR comments
2. **Filters** for comments containing '🔴 Required Changes' format
3. **Extracts** task IDs from PR labels (format: `task-{number}`)
4. **Creates** CodeRun resources with remediation configuration
5. **Triggers** Rex agent to perform automated remediation

## Architecture

```
GitHub PR Comment
        ↓
  Webhook Event
        ↓
Argo Events Sensor (remediation-feedback-sensor)
        ↓
   Argo Workflow (task ID extraction)
        ↓
    CodeRun Resource
        ↓
     Rex Agent (remediation mode)
```

## Event Filtering

The sensor filters for events that meet ALL of the following criteria:

- **Event Type**: `issue_comment`
- **Action**: `created`
- **Comment Body**: Contains `🔴 Required Changes`
- **Issue Type**: Pull request (not regular issue)
- **Authorized Author**: `5DLabs-Tess` or `5DLabs-Tess[bot]` (optional)

## Task ID Extraction

Task IDs are extracted from PR labels using the following logic:

1. Find all labels starting with `task-`
2. Extract the numeric portion after `task-`
3. Use the first matching label found

Example labels that work:
- `task-1` → Task ID: `1`
- `task-42` → Task ID: `42`

## Generated CodeRun Configuration

When a feedback comment is detected, the sensor creates a CodeRun with:

```yaml
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  generateName: rex-remediation-
  namespace: agent-platform
  labels:
    task-id: "{extracted-task-id}"
    pr-number: "{pr-number}"
    trigger-type: "comment-feedback"
    remediation-iteration: "{iteration}"
spec:
  taskId: {extracted-task-id}
  service: "task{extracted-task-id}"
  githubApp: "5DLabs-Rex"
  continueSession: true
  env:
    REMEDIATION_MODE: "true"
    FEEDBACK_COMMENT_ID: "{comment-id}"
    ITERATION_COUNT: "{iteration}"
    FEEDBACK_AUTHOR: "{comment-author}"
    PR_NUMBER: "{pr-number}"
```

## Deployment

The sensor is deployed as part of the GitHub webhooks kustomization:

```bash
kubectl apply -k infra/gitops/resources/github-webhooks/
```

## Monitoring

### Check Sensor Status
```bash
kubectl get sensors -n argo
kubectl get pods -n argo | grep remediation
```

### View Sensor Logs
```bash
kubectl logs -n argo deployment/remediation-feedback-sensor
```

### Check Created Resources
```bash
# View remediation workflows
kubectl get workflows -n agent-platform -l type=remediation-workflow

# View remediation CodeRuns
kubectl get coderuns -n agent-platform -l trigger-type=comment-feedback
```

## Testing

Use the provided test script to simulate feedback comments:

```bash
./infra/gitops/resources/github-webhooks/test-remediation-sensor.sh
```

This will:
1. Create a test webhook payload
2. Send it to the sensor
3. Verify the sensor processes it correctly

## Troubleshooting

### Sensor Not Triggering

1. **Check webhook delivery**:
   ```bash
   # View recent webhook deliveries in GitHub
   # Settings → Webhooks → Recent Deliveries
   ```

2. **Verify sensor logs**:
   ```bash
   kubectl logs -n argo deployment/remediation-feedback-sensor
   ```

3. **Test with simplified payload**:
   ```bash
   ./test-remediation-sensor.sh
   ```

### CodeRun Creation Fails

1. **Check RBAC permissions**:
   ```bash
   kubectl auth can-i create coderuns --as=system:serviceaccount:argo:argo-events-sa -n agent-platform
   ```

2. **Verify workflow execution**:
   ```bash
   kubectl get workflows -n agent-platform -l type=remediation-workflow
   kubectl logs -n agent-platform <workflow-pod>
   ```

### Task ID Extraction Issues

1. **Check PR labels**:
   ```bash
   # Get PR details from GitHub API
   curl https://api.github.com/repos/5dlabs/cto/pulls/{pr-number}
   ```

2. **Verify label format**:
   - Labels must start with `task-`
   - Must be followed by numeric digits only
   - Example: `task-1`, `task-42`

## Configuration

### Environment Variables
- `REMEDIATION_MODE=true`: Enables remediation mode in Rex agent
- `FEEDBACK_COMMENT_ID`: Comment ID for context reference
- `ITERATION_COUNT`: Tracks remediation iteration cycles
- `FEEDBACK_AUTHOR`: Original feedback author
- `PR_NUMBER`: Associated pull request number

### Resource Limits
- **Memory**: 128Mi limit, 64Mi request
- **CPU**: 200m limit, 100m request

### Retry Strategy
- **Steps**: 2 retry attempts
- **Duration**: 5 seconds base
- **Factor**: 2x exponential backoff
- **Jitter**: 10% randomization

## Integration Points

### Existing Infrastructure
- **EventSource**: `github-eventsource` (existing)
- **Service Account**: `argo-events-sa` (existing)
- **RBAC**: Extended with CodeRun permissions
- **Namespace**: `agent-platform` (existing)

### Related Components
- **Rex Agent**: Consumes CodeRun resources
- **GitHub Webhooks**: Provides event source
- **Argo Workflows**: Executes remediation logic
- **CodeRun Controller**: Manages CodeRun lifecycle
