# Task 2: Setup Argo Events Infrastructure



## Overview

Create and configure specialized Argo Events Sensors for multi-agent workflow orchestration, enabling event-driven coordination between Rex, Cleo, and Tess agents through GitHub webhook processing and workflow resumption.

## Technical Context

The existing Argo Events infrastructure (EventBus, EventSource) is already deployed and functional. This task focuses on creating four specialized Sensors that handle the complex event correlation and workflow state transitions required for the multi-agent orchestration system.

## Implementation Guide

### Phase 1: Multi-Agent Workflow Resume Sensor



1. **Create PR Creation Event Sensor**
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: multi-agent-workflow-resume
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





```



2. **Implement Workflow Correlation Logic**
   - Extract task ID from PR labels using jq: `.pull_request.labels[?(@.name | startswith("task-"))].name | split("-")[1]`
   - Target workflows with label selector: `workflow-type=play-orchestration,task-id={{extracted-task-id}},current-stage=waiting-pr-created`


   - Use Argo Workflow resume operation to continue suspended workflows

### Phase 2: Ready-for-QA Label Sensor



1. **Create PR Labeling Event Sensor**
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: ready-for-qa-sensor
     namespace: argo
   spec:
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



2. **Configure Workflow Targeting**


   - Ensure label was added by Cleo (5DLabs-Cleo[bot])


   - Target workflows in `waiting-ready-for-qa` stage


   - Resume Tess stage after successful correlation

### Phase 3: PR Approval Sensor



1. **Create PR Review Event Sensor**
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: pr-approval-sensor
     namespace: argo
   spec:
     dependencies:
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





```



2. **Verify Tess Approval**


   - Confirm reviewer is 5DLabs-Tess[bot]


   - Extract task ID from PR labels


   - Resume workflow completion stage

### Phase 4: Rex Remediation Sensor



1. **Create Push Event Detection**
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: rex-remediation-sensor
     namespace: argo
   spec:
     dependencies:
     - name: rex-push-event
       eventSourceName: github
       eventName: push
       filters:
         data:
         - path: sender.login
           type: string
           value: "5DLabs-Rex[bot]"
         - path: ref
           type: string
           comparator: "="
           value: "refs/heads/task-.*"





```



2. **Implement QA Pipeline Restart Logic**
   ```yaml
   triggers:
   - template:
       name: restart-qa-pipeline
       k8s:
         operation: delete
         source:
           resource:
             apiVersion: agents.platform/v1
             kind: CodeRun
             metadata:
               labelSelector: "task-id={{extracted-task-id}},github-app!=5DLabs-Rex"





```



## Code Examples

### Event Correlation Pattern



```yaml
# Standard pattern for task ID extraction and workflow targeting
- src:
    dependencyName: github-pr-created
    dataTemplate: |
      {{jq '.pull_request.labels[?(@.name | startswith("task-"))].name | split("-")[1]'}}
  dest: spec.arguments.parameters.task-id

# Workflow label selector for precise targeting
labelSelector: |
  workflow-type=play-orchestration,
  task-id={{task-id}},
  current-stage={{target-stage}}






```

### Webhook Field Extraction



```bash


# Extract task ID from PR labels
TASK_ID=$(echo '$webhook_payload' | jq -r '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]')

# Verify branch naming matches
BRANCH_TASK=$(echo '$webhook_payload' | jq -r '.pull_request.head.ref' | grep -o '^task-[0-9]*' | cut -d'-' -f2)

# Validation: both must match
if [ "$TASK_ID" != "$BRANCH_TASK" ]; then
  echo "ERROR: Task association mismatch"
  exit 1
fi






```

### Workflow Resume Configuration



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

## Architecture Patterns

### Event-Driven State Machine
The sensor configuration implements a distributed state machine where:
1. **Suspended Workflows**: Wait indefinitely for specific GitHub events
2. **Event Correlation**: GitHub webhooks are processed and correlated with task IDs
3. **Selective Resumption**: Only workflows matching exact criteria are resumed
4. **Stage Progression**: Workflows update their stage labels after resumption

### Multi-Method Validation
Sensors implement multi-method validation to prevent false positives:
- **Primary**: PR labels (`task-3`) for webhook detection
- **Secondary**: Branch naming (`task-3-feature`) for human readability
- **Validation**: Both methods must agree or processing fails

### Remediation Patterns
Rex remediation follows a specific cancellation and restart pattern:
1. **Detection**: Push event from 5DLabs-Rex[bot] to task branch
2. **Cancellation**: Delete running Cleo/Tess CodeRun CRDs
3. **State Reset**: Remove "ready-for-qa" labels and reset workflow stage
4. **Restart**: Resume workflow from Cleo stage with fresh code

## Key Implementation Details

### Sensor Dependencies
All sensors depend on:
- **EventSource**: Existing `github` EventSource for webhook processing
- **EventBus**: Existing `argo` EventBus for event distribution
- **RBAC**: Proper permissions for workflow resume operations

### Webhook Processing
- **Rate Limiting**: GitHub webhooks may be rate limited, implement appropriate handling
- **Payload Validation**: Verify webhook signatures and source authenticity
- **Field Extraction**: Use jq for reliable JSON field extraction from payloads

### Error Handling
- **Correlation Failures**: Log and alert when task association fails
- **Missing Workflows**: Handle cases where target workflow doesn't exist
- **Permission Errors**: Ensure sensors have proper RBAC for operations

## Testing Strategy

### Unit Testing
1. **Deploy Each Sensor**: Verify with `kubectl get sensors -n argo`
2. **Monitor Logs**: Use `kubectl logs -f sensor-pod -n argo`
3. **Webhook Processing**: Trigger test GitHub events
4. **Correlation Validation**: Confirm task ID extraction works

### Integration Testing
1. **End-to-End Flow**: Create suspended test workflows
2. **Event Triggering**: Use actual GitHub PRs to test event flow
3. **Workflow Resumption**: Verify correct workflows are resumed
4. **Remediation Testing**: Test Rex push cancellation logic

### Operational Validation
1. **Rate Limiting**: Monitor for GitHub API rate limit issues
2. **Event Processing Latency**: Measure webhook → resumption delays
3. **Resource Usage**: Monitor sensor pod resource consumption
4. **Error Rates**: Track correlation failures and retry logic

## References

- [Argo Events Sensor Documentation](https://argoproj.github.io/argo-events/concepts/sensor/)
- [GitHub Webhook Payloads](https://docs.github.com/en/developers/webhooks-and-events/webhooks/webhook-events-and-payloads)
- [Argo Workflows Resume Operations](https://argoproj.github.io/argo-workflows/rest-api/#operation/WorkflowServiceResumeWorkflow)


- [Multi-Agent Architecture](.taskmaster/docs/architecture.md)


- [Product Requirements](.taskmaster/docs/prd.txt)