# Task 1: Setup GitHub Webhook Infrastructure

## Overview
Create and configure an Argo Events Sensor resource for detecting and processing PR comments with '🔴 Required Changes' format. This sensor will leverage the existing GitHub webhook infrastructure to trigger automated remediation workflows when QA feedback is received.

## Technical Context
The platform already has a working GitHub webhook EventSource and several active sensors. This task involves creating a new remediation-specific sensor that integrates seamlessly with the existing infrastructure while adding the capability to detect and respond to structured QA feedback.

### Existing Infrastructure
- **GitHub EventSource**: Located at `infra/gitops/resources/github-webhooks/eventsource.yaml`
- **Working Sensors**: `play-workflow-sensors.yaml`, `implementation-agent-remediation`
- **HTTPRoute**: Configured for webhook ingress
- **RBAC**: Service accounts and permissions already established
- **GitHub Webhook Secret**: Configured and functional

## Implementation Guide

### Step 1: Create Remediation Sensor YAML Configuration

#### 1.1 Sensor Resource Structure
Create a new file `infra/gitops/resources/github-webhooks/remediation-feedback-sensor.yaml`:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: pr-comment-remediation
  namespace: argo-events
spec:
  template:
    serviceAccountName: argo-events-sa
  dependencies:
    - name: feedback-comment
      eventSourceName: github-eventsource
      eventName: org
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["issue_comment"]
          - path: body.action
            type: string
            value: ["created"]
          - path: body.comment.body
            type: string
            comparator: "~"
            value: ".*🔴 Required Changes.*"
```

#### 1.2 Task ID Extraction
Add JSONPath expressions to extract task ID from PR labels:

```yaml
  parameters:
    - src:
        dependencyName: feedback-comment
        dataKey: body.issue.labels
      dest: labels
    - src:
        dependencyName: feedback-comment
        dataTemplate: '{{ .labels | toJson | contains "task-" | extractTaskId }}'
      dest: taskId
```

### Step 2: Implement Event Filtering

#### 2.1 Comment Validation
Add comprehensive filtering to ensure only valid feedback triggers remediation:

```yaml
filters:
  data:
    - path: body.issue.pull_request
      type: string
      comparator: "!="
      value: ["null"]
    - path: body.comment.user.login
      type: string
      value: ["5DLabs-Tess", "5DLabs-Tess[bot]", "authorized-reviewer"]
  exprs:
    - expr: commentBody =~ '🔴 Required Changes' && issue.state == 'open'
      fields:
        - name: commentBody
          path: body.comment.body
        - name: issue.state
          path: body.issue.state
```

### Step 3: Configure CodeRun Resource Generation

#### 3.1 Trigger Configuration
Set up the Kubernetes resource trigger to create CodeRun CRDs:

```yaml
triggers:
  - template:
      name: trigger-remediation
      k8s:
        operation: create
        source:
          resource:
            apiVersion: platform.5dlabs.com/v1
            kind: CodeRun
            metadata:
              generateName: rex-remediation-
              namespace: agent-platform
              labels:
                task-id: "{{.taskId}}"
                pr-number: "{{.prNumber}}"
                trigger-type: "comment-feedback"
                iteration: "{{.iteration}}"
            spec:
              service: "task{{.taskId}}"
              github_app: "5DLabs-Rex"
              pr_number: "{{.prNumber}}"
              pr_comment_id: "{{.commentId}}"
              continue_session: true
              env:
                - name: REMEDIATION_MODE
                  value: "true"
                - name: FEEDBACK_COMMENT_ID
                  value: "{{.commentId}}"
                - name: ITERATION_COUNT
                  value: "{{.iteration}}"
        parameters:
          - src:
              dependencyName: feedback-comment
              dataKey: body.issue.number
            dest: prNumber
          - src:
              dependencyName: feedback-comment
              dataKey: body.comment.id
            dest: commentId
```

### Step 4: Deploy and Integration

#### 4.1 Deployment Steps
1. **Apply the Sensor**: `kubectl apply -f infra/gitops/resources/github-webhooks/remediation-feedback-sensor.yaml`
2. **Verify Pod Status**: `kubectl get pods -n argo-events | grep remediation`
3. **Check Event Source Connection**: `kubectl logs -n argo-events deployment/pr-comment-remediation-sensor`
4. **Validate RBAC**: Ensure sensor can create CodeRun resources in agent-platform namespace

#### 4.2 Integration Verification
- Confirm sensor appears in Argo Events controller logs
- Test event flow from GitHub webhook to sensor
- Verify no conflicts with existing sensors
- Check resource creation permissions

### Step 5: Testing Strategy

#### 5.1 Unit Testing
- Test event filtering logic with sample webhook payloads
- Verify JSONPath expressions for task ID extraction
- Validate CodeRun resource generation templates

#### 5.2 Integration Testing
1. Create test PR with task label (e.g., `task-42`)
2. Post comment with feedback format:
```markdown
## 🔴 Required Changes

**Issue Type**: Bug
**Severity**: High

### Description
Test feedback for remediation flow
```
3. Verify CodeRun creation with correct parameters
4. Check environment variables include `REMEDIATION_MODE=true`

#### 5.3 End-to-End Testing
- Test concurrent feedback on multiple PRs
- Validate iteration tracking
- Test edge cases (malformed comments, missing labels)
- Verify integration with Rex agent workflow

## Code Examples

### Complete Sensor Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: pr-comment-remediation
  namespace: argo-events
  labels:
    app: remediation-loop
    component: feedback-sensor
spec:
  template:
    serviceAccountName: argo-events-sa
    container:
      resources:
        requests:
          memory: "64Mi"
          cpu: "100m"
        limits:
          memory: "128Mi"
          cpu: "200m"
  dependencies:
    - name: feedback-comment
      eventSourceName: github-eventsource
      eventName: org
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["issue_comment"]
          - path: body.action
            type: string
            value: ["created"]
          - path: body.comment.body
            type: string
            comparator: "~"
            value: ".*🔴 Required Changes.*"
          - path: body.issue.pull_request
            type: string
            comparator: "!="
            value: ["null"]
  triggers:
    - template:
        name: trigger-remediation
        conditions: feedback-comment
        k8s:
          operation: create
          source:
            resource: |
              apiVersion: platform.5dlabs.com/v1
              kind: CodeRun
              metadata:
                generateName: rex-remediation-
                namespace: agent-platform
                labels:
                  task-id: "{{.taskId}}"
                  pr-number: "{{.prNumber}}"
                  trigger-type: "comment-feedback"
                  remediation-iteration: "{{.iteration}}"
              spec:
                service: "task{{.taskId}}"
                github_app: "5DLabs-Rex"
                pr_number: {{.prNumber}}
                pr_comment_id: "{{.commentId}}"
                continue_session: true
                env:
                  - name: REMEDIATION_MODE
                    value: "true"
                  - name: FEEDBACK_COMMENT_ID
                    value: "{{.commentId}}"
                  - name: ITERATION_COUNT
                    value: "{{.iteration}}"
                  - name: FEEDBACK_AUTHOR
                    value: "{{.commentAuthor}}"
          parameters:
            - src:
                dependencyName: feedback-comment
                dataKey: body.issue.number
              dest: prNumber
            - src:
                dependencyName: feedback-comment
                dataKey: body.comment.id
              dest: commentId
            - src:
                dependencyName: feedback-comment
                dataKey: body.comment.user.login
              dest: commentAuthor
            - src:
                dependencyName: feedback-comment
                dataTemplate: '{{ .body.issue.labels | extractTaskId }}'
              dest: taskId
            - src:
                dependencyName: feedback-comment
                dataTemplate: '{{ .iteration | default "1" }}'
              dest: iteration
```

## Monitoring and Observability

### Metrics to Track
- Sensor trigger count by task ID
- CodeRun creation success/failure rate
- Event processing latency
- Filter match rate

### Logging
Ensure structured logging for:
- Incoming webhook events
- Filter evaluation results
- CodeRun creation attempts
- Error conditions

### Alerts
Set up alerts for:
- Sensor pod restarts
- CodeRun creation failures
- Event processing errors
- High latency in event processing

## Troubleshooting Guide

### Common Issues

1. **Sensor Not Triggering**
   - Check event source connection
   - Verify webhook delivery in GitHub settings
   - Review sensor logs for filter mismatches

2. **CodeRun Creation Fails**
   - Verify RBAC permissions
   - Check namespace existence
   - Validate resource template syntax

3. **Task ID Extraction Issues**
   - Review PR labels format
   - Test JSONPath expressions
   - Check for special characters in labels

4. **Event Filtering Problems**
   - Validate regex patterns
   - Check comment format
   - Review user authorization list

## Success Criteria
- Sensor successfully deployed and running
- Feedback comments trigger CodeRun creation
- Correct parameters passed to Rex agent
- Integration with existing infrastructure verified
- End-to-end remediation flow tested