# Task 3: Create Remediation Sensor and Trigger

## Overview
Deploy an Argo Events Sensor to process feedback comments and trigger Rex remediation CodeRuns. This sensor integrates with the existing play workflow infrastructure to enable automated remediation when QA feedback is received. It builds upon the webhook infrastructure from Task 1 and feedback processing capabilities from Task 2.

## Technical Context
This sensor represents the core automation component of the remediation loop, bridging feedback detection with automated fixes. It leverages existing Kubernetes infrastructure while adding remediation-specific logic to trigger Rex when Tess (QA agent) posts feedback comments.

### Dependencies
- **Task 1**: GitHub webhook infrastructure and initial sensor setup
- **Task 2**: Feedback comment processing and validation logic
- **Existing Infrastructure**: 
  - GitHub EventSource configured in `github-webhooks` namespace
  - Play workflow sensors for reference patterns
  - CodeRun CRD for agent execution
  - ConfigMap-based state management system

## Implementation Guide

### Step 1: Create Remediation Sensor Configuration

#### 1.1 Sensor Resource Structure
Create the main sensor file `infra/gitops/resources/github-webhooks/pr-comment-remediation-sensor.yaml`:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: pr-comment-remediation
  namespace: github-webhooks
  labels:
    app: remediation-sensor
    component: webhook-processing
    version: v1
spec:
  template:
    serviceAccountName: argo-events-sa
    container:
      resources:
        requests:
          memory: "128Mi"
          cpu: "100m"
        limits:
          memory: "256Mi"
          cpu: "200m"
  eventBusName: default
  dependencies:
    - name: feedback-comment
      eventSourceName: github
      eventName: issue_comment
      filters:
        data:
          # Only process comment creation events
          - path: body.action
            type: string
            value: ["created"]
          # Ensure this is a PR (has pull_request field)
          - path: body.issue.pull_request.url
            type: string
            comparator: "!="
            value: [""]
          # Only QA feedback comments
          - path: body.comment.body
            type: string
            comparator: "~"
            value: ".*🔴 Required Changes.*"
          # Only from authorized users
          - path: body.comment.user.login
            type: string
            value: ["5DLabs-Tess", "5DLabs-Tess[bot]"]
        exprs:
          # Additional validation expressions
          - expr: "issue.state == 'open'"
            fields:
              - name: issue.state
                path: body.issue.state
```

#### 1.2 Parameter Extraction
Configure parameter extraction for CodeRun generation:

```yaml
  triggers:
    - template:
        name: create-remediation-coderun
        k8s:
          operation: create
          source:
            resource:
              apiVersion: cto.5dlabs.com/v1alpha1
              kind: CodeRun
              metadata:
                generateName: remediation-rex-
                namespace: agent-platform
                labels:
                  task-id: ""  # Will be populated via parameter
                  trigger-type: "comment-feedback"
                  agent-type: "rex"
                annotations:
                  remediation.5dlabs.com/pr-number: ""
                  remediation.5dlabs.com/comment-id: ""
                  remediation.5dlabs.com/iteration: ""
              spec:
                github_app: "5DLabs-Rex"
                remediation_mode: true
                continue_session: true
                env:
                  REMEDIATION_MODE: "true"
                  FEEDBACK_COMMENT_ID: ""  # From parameter
                  ITERATION_COUNT: ""      # From state lookup
                  MAX_ITERATIONS: "10"
        parameters:
          # Extract PR number
          - src:
              dependencyName: feedback-comment
              dataKey: body.issue.number
            dest: spec.pr_number
          # Extract comment ID
          - src:
              dependencyName: feedback-comment
              dataKey: body.comment.id
            dest: spec.pr_comment_id
          # Extract comment ID for environment
          - src:
              dependencyName: feedback-comment
              dataKey: body.comment.id
            dest: spec.env.FEEDBACK_COMMENT_ID
          # Extract task ID from labels
          - src:
              dependencyName: feedback-comment
              dataTemplate: |
                {{- range .body.issue.labels -}}
                  {{- if regexMatch "^task-\\d+$" .name -}}
                    {{ regexReplaceAll "^task-" .name "" }}
                  {{- end -}}
                {{- end -}}
            dest: metadata.labels.task-id
          # Set annotations
          - src:
              dependencyName: feedback-comment
              dataKey: body.issue.number
            dest: metadata.annotations["remediation.5dlabs.com/pr-number"]
          - src:
              dependencyName: feedback-comment
              dataKey: body.comment.id
            dest: metadata.annotations["remediation.5dlabs.com/comment-id"]
```

### Step 2: Implement Task ID Extraction Logic

#### 2.1 JSONPath for Label Processing
The sensor uses JSONPath expressions to extract task IDs from PR labels:

```yaml
# Task ID extraction from PR labels array
- src:
    dependencyName: feedback-comment
    dataTemplate: |
      {{- $taskId := "" -}}
      {{- range .body.issue.labels -}}
        {{- if regexMatch "^task-\\d+$" .name -}}
          {{- $taskId = regexReplaceAll "^task-" .name "" -}}
        {{- end -}}
      {{- end -}}
      {{- if eq $taskId "" -}}
        unknown
      {{- else -}}
        {{ $taskId }}
      {{- end -}}
  dest: metadata.labels["task-id"]
```

#### 2.2 Label Validation
Add validation to ensure task ID is properly extracted:

```yaml
# Conditional trigger - only if task ID found
        condition: "taskId != 'unknown'"
        parameters:
          - src:
              dependencyName: feedback-comment
              dataTemplate: |
                {{- range .body.issue.labels -}}
                  {{- if regexMatch "^task-\\d+$" .name -}}
                    {{ regexReplaceAll "^task-" .name "" }}
                  {{- end -}}
                {{- end -}}
            dest: taskId
```

### Step 3: Configure State Management Integration

#### 3.1 Iteration Counter Logic
The sensor integrates with ConfigMap-based state tracking:

```yaml
# Pre-trigger to check/update iteration counter
triggers:
  - template:
      name: check-iteration-state
      k8s:
        operation: patch
        source:
          resource:
            apiVersion: v1
            kind: ConfigMap
            metadata:
              name: ""  # Will be task-{id}-remediation-state
              namespace: agent-platform
            data:
              last_comment_id: ""
              current_iteration: ""
        patchStrategy: merge
      parameters:
        - src:
            dependencyName: feedback-comment
            dataTemplate: "task-{{ .taskId }}-remediation-state"
          dest: metadata.name
        - src:
            dependencyName: feedback-comment
            dataKey: body.comment.id
          dest: data.last_comment_id
  # Main CodeRun creation trigger follows
```

#### 3.2 State ConfigMap Template
Each task gets a dedicated state ConfigMap:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: task-{id}-remediation-state
  namespace: agent-platform
  labels:
    app: remediation-loop
    task-id: "{id}"
data:
  current_iteration: "1"
  max_iterations: "10" 
  start_time: ""
  last_comment_id: ""
  status: "active"
  feedback_history: "[]"  # JSON array of feedback records
```

### Step 4: Implement Event Deduplication

#### 4.1 Deduplication Strategy
Use Argo Events' built-in deduplication with custom logic:

```yaml
spec:
  eventBusName: default
  replicas: 1  # Single replica to prevent race conditions
  dependencies:
    - name: feedback-comment
      eventSourceName: github
      eventName: issue_comment
      filters:
        # Existing filters...
        exprs:
          # Prevent duplicate processing of same comment
          - expr: "commentId != lastProcessedCommentId"
            fields:
              - name: commentId
                path: body.comment.id
              - name: lastProcessedCommentId
                # This would need to be populated from ConfigMap
                path: body.comment.id  # Placeholder - actual implementation needs state lookup
```

#### 4.2 Idempotent Resource Creation
Configure CodeRun creation to be idempotent:

```yaml
spec:
  # Use deterministic naming based on PR and comment
  generateName: "rex-remediation-"
  labels:
    pr-number: ""
    comment-id: ""
    deduplication-key: ""  # Combination of PR + comment ID
```

### Step 5: Configure Integration with Existing Sensors

#### 5.1 Cancellation Trigger
The sensor works with the implementation-agent-remediation sensor for cancellation:

```yaml
# Additional trigger to cancel existing quality agents
triggers:
  - template:
      name: cancel-quality-agents
      k8s:
        operation: delete
        source:
          resource:
            apiVersion: cto.5dlabs.com/v1alpha1
            kind: CodeRun
            metadata:
              labels:
                task-id: ""  # From parameter
                agent-type: "cleo,tess"
                status: "running"
        parameters:
          - src:
              dependencyName: feedback-comment
              dataTemplate: "{{ .taskId }}"
            dest: metadata.labels["task-id"]
```

#### 5.2 Label Management
Update PR labels to reflect remediation state:

```yaml
# Update PR labels to indicate remediation started
- template:
    name: update-pr-labels
    http:
      url: "https://api.github.com/repos/5dlabs/cto/issues/{{.PRNumber}}/labels"
      method: POST
      headers:
        Authorization: "token {{.GitHubToken}}"
        Content-Type: "application/json"
      payload:
        - "remediation-in-progress"
        - "iteration-{{.Iteration}}"
      # Remove ready-for-qa label via separate DELETE request
  parameters:
    - src:
        dependencyName: feedback-comment
        dataKey: body.issue.number
      dest: PRNumber
    - src:
        dependencyName: feedback-comment
        dataTemplate: "{{ .iteration }}"  # From state lookup
      dest: Iteration
```

### Step 6: Error Handling and Retry Logic

#### 6.1 Retry Configuration
Configure exponential backoff for failed triggers:

```yaml
spec:
  template:
    container:
      env:
        - name: RETRY_ATTEMPTS
          value: "3"
        - name: RETRY_BACKOFF
          value: "exponential"
  triggers:
    - template:
        name: create-remediation-coderun
        retryStrategy:
          steps: 3
          duration: "10s"
          factor: 2.0
          jitter: 0.1
```

#### 6.2 Failure Handling
Handle common failure scenarios:

```yaml
# Fallback trigger for state management failures
triggers:
  - template:
      name: remediation-fallback
      conditions: "failed_attempts > 2"
      k8s:
        operation: create
        source:
          resource:
            apiVersion: v1
            kind: Event
            metadata:
              generateName: remediation-failure-
              namespace: agent-platform
            type: Warning
            reason: RemediationTriggerFailed
            message: "Failed to trigger remediation for task {{ .taskId }}"
```

### Step 7: Monitoring and Observability

#### 7.1 Metrics Configuration
Add monitoring labels and annotations:

```yaml
metadata:
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
    prometheus.io/path: "/metrics"
  labels:
    monitoring.5dlabs.com/component: "remediation-sensor"
```

#### 7.2 Logging Configuration
Configure structured logging:

```yaml
spec:
  template:
    container:
      env:
        - name: LOG_LEVEL
          value: "INFO"
        - name: LOG_FORMAT
          value: "json"
      volumeMounts:
        - name: log-config
          mountPath: /etc/logging
    volumes:
      - name: log-config
        configMap:
          name: sensor-logging-config
```

## Testing Strategy

### Unit Testing
1. **JSONPath Expression Validation**
   - Test task ID extraction with various label formats
   - Validate parameter mapping with sample webhook payloads
   - Test regex patterns for comment filtering

2. **Resource Template Testing**
   - Verify CodeRun resource structure
   - Test parameter substitution
   - Validate resource limits and annotations

### Integration Testing
1. **End-to-End Workflow**
   - Create test PR with task label
   - Post feedback comment from authorized user
   - Verify sensor activation and CodeRun creation
   - Test cancellation of existing agents

2. **State Management**
   - Test ConfigMap creation and updates
   - Verify iteration counter functionality
   - Test maximum iteration limits

### Performance Testing
1. **Concurrent Processing**
   - Multiple PRs receiving feedback simultaneously
   - High volume comment processing
   - Resource usage monitoring

2. **Deduplication**
   - Rapid comment edits
   - Duplicate webhook deliveries
   - Race condition handling

## Deployment

### Prerequisites
- Argo Events v1.9+ installed
- GitHub EventSource configured
- RBAC permissions for CodeRun creation
- ConfigMap creation permissions

### Deployment Steps
1. Apply the sensor configuration:
   ```bash
   kubectl apply -f infra/gitops/resources/github-webhooks/pr-comment-remediation-sensor.yaml
   ```

2. Verify deployment:
   ```bash
   kubectl get sensor pr-comment-remediation -n github-webhooks
   kubectl logs -l app=remediation-sensor -n github-webhooks
   ```

3. Test with sample event:
   ```bash
   kubectl apply -f test/sample-feedback-event.yaml
   ```

## Troubleshooting

### Common Issues
1. **Sensor Not Triggering**
   - Check event filters match webhook payload structure
   - Verify EventSource is receiving events
   - Check user authorization filters

2. **Parameter Extraction Failures**
   - Validate JSONPath expressions with actual data
   - Check for missing fields in webhook payload
   - Verify dataTemplate syntax

3. **CodeRun Creation Failures**
   - Check RBAC permissions
   - Verify namespace exists
   - Check resource quota limits

### Debug Commands
```bash
# Check sensor status
kubectl describe sensor pr-comment-remediation -n github-webhooks

# View sensor logs
kubectl logs -l app=remediation-sensor -n github-webhooks -f

# Check EventSource events
kubectl get events -n github-webhooks --sort-by='.lastTimestamp'

# Verify CodeRun creation
kubectl get coderuns -l trigger-type=comment-feedback -n agent-platform
```

## Security Considerations

### Access Control
- Sensor runs with minimal RBAC permissions
- Only authorized GitHub users can trigger remediation
- ConfigMap access restricted to agent-platform namespace

### Data Validation
- Input validation on all extracted parameters
- Sanitization of user-provided comment content
- Rate limiting via Kubernetes resource quotas

### Audit Trail
- All remediation triggers logged
- State changes tracked in ConfigMaps
- GitHub audit log integration

## Future Enhancements

### Planned Improvements
1. **Advanced Filtering**
   - ML-based feedback classification
   - Priority-based processing
   - Custom reviewer authorization

2. **Enhanced State Management**
   - Persistent volume state storage
   - Cross-cluster state replication
   - Advanced iteration strategies

3. **Observability**
   - Custom metrics dashboards
   - Alerting rules
   - Performance analytics

This implementation provides a robust, production-ready remediation sensor that integrates seamlessly with existing infrastructure while providing the automation needed for the feedback loop system.