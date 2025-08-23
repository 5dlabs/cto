# Task 14: Build Workflow Resume Operations

## Overview

Implement robust workflow resume operations for each suspension point in the multi-agent orchestration system with proper event correlation and failure handling. This task ensures reliable event-driven workflow transitions that can recover from failures and handle edge cases.

## Context

The multi-agent Play Workflow uses suspend/resume patterns to coordinate with GitHub events. Currently, workflows suspend and wait for external events (PR creation, labeling, approval) to resume processing. This task builds the robust infrastructure to reliably resume workflows at the correct suspension points.

### Current Workflow Suspension Points

1. **After Rex Implementation**: Wait for PR creation
2. **After Cleo Quality**: Wait for "ready-for-qa" label
3. **After Tess Testing**: Wait for PR approval
4. **Task Progression**: Wait for task completion events

## Technical Architecture

### Resume Operation Requirements

```yaml
# Workflow Resume API Structure
apiVersion: argoproj.io/v1alpha1
kind: WorkflowResume
metadata:
  name: resume-operation
  namespace: argo
spec:
  workflowName: "play-workflow-abc123"
  nodeFieldSelector: "templateName=suspend-for-webhook"
  correlationData:
    taskId: "5"
    eventType: "pr-created"
    githubEvent:
      pullRequestNumber: 123
      action: "opened"
      labels: ["task-5"]
  validationRules:
    - field: "task-id"
      operator: "equals"
      value: "5"
    - field: "current-stage"
      operator: "equals"
      value: "waiting-pr-created"
```

### Event Correlation System

```yaml
# Argo Events Sensor with Resume Logic
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: workflow-resume-sensor
  namespace: argo-events
spec:
  dependencies:
  - name: github-pr-created
    eventSourceName: github-webhook
    eventName: pull-request-opened
    filters:
      data:
      - path: body.pull_request.labels[].name
        type: string
        comparator: "regex"
        value: "^task-[0-9]+$"

  - name: github-pr-labeled
    eventSourceName: github-webhook
    eventName: pull-request-labeled
    filters:
      data:
      - path: body.label.name
        type: string
        value: "ready-for-qa"

  - name: github-pr-approved
    eventSourceName: github-webhook
    eventName: pull-request-review
    filters:
      data:
      - path: body.review.state
        type: string
        value: "approved"

  triggers:
  - template:
      name: resume-workflow-trigger
      argoWorkflow:
        operation: resume
        source:
          resource:
            # Dynamic workflow selection based on event correlation
            labelSelector: |
              workflow-type=play-orchestration,
              task-id={{extracted-task-id}},
              current-stage={{target-stage}}
        parameters:
        - src:
            dependencyName: github-pr-created
            dataTemplate: |
              {{jq '.body.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}
          dest: spec.arguments.parameters.extracted-task-id
        - src:
            dependencyName: github-pr-created
            dataKey: body.pull_request.number
          dest: spec.arguments.parameters.pr-number
```

## Implementation Requirements

### 1. Resume API Implementation

**Core Resume Function**:
```go
// Resume operation with validation and retry logic
func ResumeWorkflow(ctx context.Context, resumeSpec *WorkflowResumeSpec) error {
    // Step 1: Validate resume request
    if err := validateResumeRequest(resumeSpec); err != nil {
        return fmt.Errorf("resume validation failed: %w", err)
    }

    // Step 2: Find target workflow
    workflow, err := findTargetWorkflow(ctx, resumeSpec)
    if err != nil {
        return fmt.Errorf("workflow lookup failed: %w", err)
    }

    // Step 3: Validate workflow state
    if err := validateWorkflowState(workflow, resumeSpec); err != nil {
        return fmt.Errorf("workflow state validation failed: %w", err)
    }

    // Step 4: Perform resume with retry logic
    return performResumeWithRetry(ctx, workflow, resumeSpec)
}

func validateResumeRequest(spec *WorkflowResumeSpec) error {
    // Validate required fields
    if spec.WorkflowName == "" && spec.LabelSelector == "" {
        return errors.New("either workflowName or labelSelector required")
    }

    if spec.CorrelationData.TaskId == "" {
        return errors.New("task ID required for correlation")
    }

    if spec.CorrelationData.EventType == "" {
        return errors.New("event type required for validation")
    }

    return nil
}

func findTargetWorkflow(ctx context.Context, spec *WorkflowResumeSpec) (*v1alpha1.Workflow, error) {
    client := argo.NewArgoWorkflowsClient()

    if spec.WorkflowName != "" {
        // Direct workflow name lookup
        return client.Get(ctx, spec.WorkflowName, metav1.GetOptions{})
    }

    // Label-based workflow discovery
    labelSelector := fmt.Sprintf(
        "workflow-type=play-orchestration,task-id=%s,current-stage=%s",
        spec.CorrelationData.TaskId,
        spec.TargetStage,
    )

    workflows, err := client.List(ctx, metav1.ListOptions{
        LabelSelector: labelSelector,
    })
    if err != nil {
        return nil, err
    }

    if len(workflows.Items) == 0 {
        return nil, fmt.Errorf("no workflows found matching criteria: %s", labelSelector)
    }

    if len(workflows.Items) > 1 {
        return nil, fmt.Errorf("multiple workflows found matching criteria: %s", labelSelector)
    }

    return &workflows.Items[0], nil
}
```

### 2. Retry Logic with Exponential Backoff

**Robust Retry Implementation**:
```go
type RetryConfig struct {
    MaxAttempts   int           `json:"maxAttempts"`
    InitialDelay  time.Duration `json:"initialDelay"`
    MaxDelay      time.Duration `json:"maxDelay"`
    BackoffFactor float64       `json:"backoffFactor"`
}

func performResumeWithRetry(ctx context.Context, workflow *v1alpha1.Workflow, spec *WorkflowResumeSpec) error {
    retryConfig := RetryConfig{
        MaxAttempts:   3,
        InitialDelay:  time.Second * 2,
        MaxDelay:      time.Second * 30,
        BackoffFactor: 2.0,
    }

    var lastError error
    delay := retryConfig.InitialDelay

    for attempt := 1; attempt <= retryConfig.MaxAttempts; attempt++ {
        log.Printf("Resume attempt %d/%d for workflow %s", attempt, retryConfig.MaxAttempts, workflow.Name)

        err := performResume(ctx, workflow, spec)
        if err == nil {
            log.Printf("Resume successful on attempt %d for workflow %s", attempt, workflow.Name)
            return nil
        }

        lastError = err

        // Check if error is retryable
        if !isRetryableError(err) {
            log.Printf("Non-retryable error for workflow %s: %v", workflow.Name, err)
            return err
        }

        if attempt < retryConfig.MaxAttempts {
            log.Printf("Resume failed on attempt %d, retrying in %v: %v", attempt, delay, err)

            select {
            case <-ctx.Done():
                return ctx.Err()
            case <-time.After(delay):
                // Calculate next delay with exponential backoff
                delay = time.Duration(float64(delay) * retryConfig.BackoffFactor)
                if delay > retryConfig.MaxDelay {
                    delay = retryConfig.MaxDelay
                }
            }
        }
    }

    return fmt.Errorf("resume failed after %d attempts, last error: %w", retryConfig.MaxAttempts, lastError)
}

func isRetryableError(err error) bool {
    // Define retryable error conditions
    retryableErrors := []string{
        "connection refused",
        "timeout",
        "temporary failure",
        "service unavailable",
        "internal server error",
    }

    errMsg := strings.ToLower(err.Error())
    for _, retryableErr := range retryableErrors {
        if strings.Contains(errMsg, retryableErr) {
            return true
        }
    }

    return false
}
```

### 3. Event Validation and Correlation

**Comprehensive Event Validation**:
```go
type EventValidationResult struct {
    Valid         bool     `json:"valid"`
    TaskId        string   `json:"taskId"`
    EventType     string   `json:"eventType"`
    TargetStage   string   `json:"targetStage"`
    ValidationErrors []string `json:"validationErrors"`
}

func validateEventCorrelation(event *GitHubWebhookEvent, workflow *v1alpha1.Workflow) (*EventValidationResult, error) {
    result := &EventValidationResult{
        Valid: true,
        ValidationErrors: []string{},
    }

    // Extract task ID from event
    taskIdFromEvent := extractTaskIdFromEvent(event)
    if taskIdFromEvent == "" {
        result.Valid = false
        result.ValidationErrors = append(result.ValidationErrors, "no task ID found in event")
        return result, nil
    }
    result.TaskId = taskIdFromEvent

    // Extract task ID from workflow labels
    taskIdFromWorkflow := workflow.Labels["task-id"]
    if taskIdFromWorkflow == "" {
        result.Valid = false
        result.ValidationErrors = append(result.ValidationErrors, "no task ID found in workflow labels")
        return result, nil
    }

    // Validate task ID correlation
    if taskIdFromEvent != taskIdFromWorkflow {
        result.Valid = false
        result.ValidationErrors = append(result.ValidationErrors,
            fmt.Sprintf("task ID mismatch: event=%s, workflow=%s", taskIdFromEvent, taskIdFromWorkflow))
        return result, nil
    }

    // Determine event type and target stage
    result.EventType, result.TargetStage = determineEventTypeAndStage(event)

    // Validate workflow is in correct stage for this event
    currentStage := workflow.Labels["current-stage"]
    if currentStage != result.TargetStage {
        result.Valid = false
        result.ValidationErrors = append(result.ValidationErrors,
            fmt.Sprintf("workflow stage mismatch: current=%s, expected=%s", currentStage, result.TargetStage))
        return result, nil
    }

    // Validate workflow is suspended
    if !isWorkflowSuspended(workflow) {
        result.Valid = false
        result.ValidationErrors = append(result.ValidationErrors, "workflow is not in suspended state")
        return result, nil
    }

    return result, nil
}

func extractTaskIdFromEvent(event *GitHubWebhookEvent) string {
    // Extract from PR labels
    for _, label := range event.PullRequest.Labels {
        if strings.HasPrefix(label.Name, "task-") {
            parts := strings.Split(label.Name, "-")
            if len(parts) >= 2 {
                return parts[1]
            }
        }
    }

    // Extract from branch name as fallback
    branchName := event.PullRequest.Head.Ref
    if strings.HasPrefix(branchName, "task-") {
        parts := strings.Split(branchName, "-")
        if len(parts) >= 2 {
            return parts[1]
        }
    }

    return ""
}

func determineEventTypeAndStage(event *GitHubWebhookEvent) (eventType, targetStage string) {
    switch event.Action {
    case "opened":
        return "pr-created", "waiting-pr-created"
    case "labeled":
        if event.Label.Name == "ready-for-qa" {
            return "pr-labeled-ready", "waiting-ready-for-qa"
        }
        return "pr-labeled", "unknown"
    case "submitted":
        if event.Review.State == "approved" {
            return "pr-approved", "waiting-pr-approved"
        }
        return "pr-reviewed", "unknown"
    default:
        return "unknown", "unknown"
    }
}
```

### 4. Circuit Breaker Pattern

**Failure Protection**:
```go
type CircuitBreaker struct {
    MaxFailures     int           `json:"maxFailures"`
    ResetTimeout    time.Duration `json:"resetTimeout"`

    mutex           sync.RWMutex
    failures        int
    lastFailureTime time.Time
    state          CircuitBreakerState
}

type CircuitBreakerState int

const (
    StateClosed CircuitBreakerState = iota
    StateOpen
    StateHalfOpen
)

func NewCircuitBreaker(maxFailures int, resetTimeout time.Duration) *CircuitBreaker {
    return &CircuitBreaker{
        MaxFailures:  maxFailures,
        ResetTimeout: resetTimeout,
        state:       StateClosed,
    }
}

func (cb *CircuitBreaker) Call(ctx context.Context, operation func() error) error {
    cb.mutex.Lock()
    defer cb.mutex.Unlock()

    // Check if circuit breaker should transition states
    cb.updateState()

    if cb.state == StateOpen {
        return fmt.Errorf("circuit breaker is OPEN - operation blocked")
    }

    err := operation()

    if err != nil {
        cb.recordFailure()
        return err
    }

    cb.recordSuccess()
    return nil
}

func (cb *CircuitBreaker) updateState() {
    now := time.Now()

    switch cb.state {
    case StateClosed:
        if cb.failures >= cb.MaxFailures {
            cb.state = StateOpen
            cb.lastFailureTime = now
            log.Printf("Circuit breaker transitioned to OPEN state")
        }
    case StateOpen:
        if now.Sub(cb.lastFailureTime) >= cb.ResetTimeout {
            cb.state = StateHalfOpen
            log.Printf("Circuit breaker transitioned to HALF-OPEN state")
        }
    case StateHalfOpen:
        // State transitions handled in recordSuccess/recordFailure
    }
}

func (cb *CircuitBreaker) recordSuccess() {
    cb.failures = 0
    if cb.state == StateHalfOpen {
        cb.state = StateClosed
        log.Printf("Circuit breaker transitioned to CLOSED state")
    }
}

func (cb *CircuitBreaker) recordFailure() {
    cb.failures++
    cb.lastFailureTime = time.Now()

    if cb.state == StateHalfOpen {
        cb.state = StateOpen
        log.Printf("Circuit breaker transitioned back to OPEN state")
    }
}
```

## Resume Operations by Suspension Point

### 1. After Rex Implementation (PR Created)

**Resume Specification**:
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowResume
metadata:
  name: resume-after-pr-created
spec:
  labelSelector: "workflow-type=play-orchestration,current-stage=waiting-pr-created"
  correlationData:
    taskId: "{{extracted-task-id}}"
    eventType: "pr-created"
    githubEvent:
      action: "opened"
      pullRequestNumber: "{{pr-number}}"
  validationRules:
    - field: "task-id"
      operator: "equals"
      value: "{{extracted-task-id}}"
    - field: "pull_request.labels[].name"
      operator: "regex"
      value: "^task-{{extracted-task-id}}$"
  resumeParameters:
    - name: "pr-number"
      value: "{{pr-number}}"
    - name: "pr-url"
      value: "{{pr-url}}"
```

### 2. After Cleo Quality (Ready for QA)

**Resume Specification**:
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowResume
metadata:
  name: resume-after-ready-for-qa
spec:
  labelSelector: "workflow-type=play-orchestration,current-stage=waiting-ready-for-qa"
  correlationData:
    taskId: "{{extracted-task-id}}"
    eventType: "pr-labeled-ready"
    githubEvent:
      action: "labeled"
      labelName: "ready-for-qa"
  validationRules:
    - field: "task-id"
      operator: "equals"
      value: "{{extracted-task-id}}"
    - field: "label.name"
      operator: "equals"
      value: "ready-for-qa"
    - field: "sender.login"
      operator: "regex"
      value: "^5DLabs-Cleo\\[bot\\]$"
  resumeParameters:
    - name: "cleo-completion-time"
      value: "{{event-timestamp}}"
```

### 3. After Tess Testing (PR Approved)

**Resume Specification**:
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowResume
metadata:
  name: resume-after-pr-approved
spec:
  labelSelector: "workflow-type=play-orchestration,current-stage=waiting-pr-approved"
  correlationData:
    taskId: "{{extracted-task-id}}"
    eventType: "pr-approved"
    githubEvent:
      action: "submitted"
      reviewState: "approved"
  validationRules:
    - field: "task-id"
      operator: "equals"
      value: "{{extracted-task-id}}"
    - field: "review.state"
      operator: "equals"
      value: "approved"
    - field: "review.user.login"
      operator: "regex"
      value: "^5DLabs-Tess\\[bot\\]$"
  resumeParameters:
    - name: "tess-approval-time"
      value: "{{event-timestamp}}"
    - name: "review-id"
      value: "{{review-id}}"
```

## Implementation Steps

### Phase 1: Core Resume Infrastructure

1. **Resume API Development**:
   - Implement workflow resume validation
   - Add event correlation logic
   - Create retry mechanism with exponential backoff
   - Build circuit breaker protection

2. **Testing Framework**:
   - Create unit tests for resume operations
   - Build integration tests with actual workflows
   - Add performance tests for concurrent resumes
   - Create failure scenario tests

### Phase 2: Event Integration

1. **Argo Events Sensor Enhancement**:
   - Update existing sensors with resume operations
   - Add comprehensive event validation
   - Implement correlation logic
   - Add error handling and logging

2. **GitHub Webhook Processing**:
   - Enhance webhook payload parsing
   - Add task ID extraction from multiple sources
   - Implement event type determination
   - Add validation for event authenticity

### Phase 3: Monitoring and Observability

1. **Resume Operation Logging**:
   - Structured logging for all resume attempts
   - Correlation ID tracking across operations
   - Performance metrics collection
   - Error categorization and tracking

2. **Monitoring Dashboards**:
   - Resume success/failure rates
   - Event correlation accuracy
   - Retry attempt patterns
   - Circuit breaker state transitions

### Phase 4: Production Deployment

1. **Deployment Strategy**:
   - Blue-green deployment for sensor updates
   - Gradual rollout with canary testing
   - Rollback procedures for failures
   - Health check implementation

2. **Operational Procedures**:
   - Troubleshooting guides
   - Manual resume procedures
   - Alert configuration
   - Performance tuning guidelines

## Error Handling Scenarios

### Scenario 1: Workflow Not Found
```go
func handleWorkflowNotFound(taskId, eventType string) error {
    log.Printf("No workflow found for task %s, event %s", taskId, eventType)

    // Check if this is a late-arriving event
    if isEventTooLate(taskId, eventType) {
        log.Printf("Event arrived too late for task %s - workflow already completed", taskId)
        return nil // Don't retry late events
    }

    // Check if workflow creation is still in progress
    if isWorkflowCreationPending(taskId) {
        log.Printf("Workflow creation pending for task %s - will retry", taskId)
        return fmt.Errorf("workflow creation pending - retryable")
    }

    // Unknown scenario - requires investigation
    log.Printf("Unknown workflow state for task %s - manual investigation required", taskId)
    return fmt.Errorf("workflow not found and reason unknown")
}
```

### Scenario 2: Multiple Workflows Found
```go
func handleMultipleWorkflows(workflows []v1alpha1.Workflow, taskId string) error {
    log.Printf("Multiple workflows found for task %s: %d instances", taskId, len(workflows))

    // Find the most recent workflow
    var mostRecent *v1alpha1.Workflow
    for _, wf := range workflows {
        if mostRecent == nil || wf.CreationTimestamp.After(mostRecent.CreationTimestamp.Time) {
            mostRecent = &wf
        }
    }

    // Cancel older workflows
    for _, wf := range workflows {
        if wf.Name != mostRecent.Name {
            log.Printf("Canceling duplicate workflow: %s", wf.Name)
            if err := cancelWorkflow(wf.Name); err != nil {
                log.Printf("Failed to cancel duplicate workflow %s: %v", wf.Name, err)
            }
        }
    }

    // Resume the most recent workflow
    return resumeWorkflow(mostRecent)
}
```

### Scenario 3: Event Correlation Failure
```go
func handleCorrelationFailure(event *GitHubWebhookEvent, workflow *v1alpha1.Workflow, validationResult *EventValidationResult) error {
    log.Printf("Event correlation failed for workflow %s: %v", workflow.Name, validationResult.ValidationErrors)

    // Log detailed correlation data for debugging
    correlationData := map[string]interface{}{
        "workflow_name": workflow.Name,
        "workflow_task_id": workflow.Labels["task-id"],
        "workflow_stage": workflow.Labels["current-stage"],
        "event_task_id": validationResult.TaskId,
        "event_type": validationResult.EventType,
        "validation_errors": validationResult.ValidationErrors,
    }

    correlationJson, _ := json.MarshalIndent(correlationData, "", "  ")
    log.Printf("Correlation data: %s", string(correlationJson))

    // Don't retry correlation failures - they indicate data issues
    return fmt.Errorf("event correlation failed - manual investigation required")
}
```

## Testing Strategy

### Unit Testing
```go
func TestResumeWorkflowValidation(t *testing.T) {
    tests := []struct {
        name        string
        resumeSpec  *WorkflowResumeSpec
        expectError bool
    }{
        {
            name: "valid resume request",
            resumeSpec: &WorkflowResumeSpec{
                LabelSelector: "task-id=5",
                CorrelationData: CorrelationData{
                    TaskId:    "5",
                    EventType: "pr-created",
                },
            },
            expectError: false,
        },
        {
            name: "missing task ID",
            resumeSpec: &WorkflowResumeSpec{
                LabelSelector: "task-id=5",
                CorrelationData: CorrelationData{
                    EventType: "pr-created",
                },
            },
            expectError: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := validateResumeRequest(tt.resumeSpec)
            if tt.expectError && err == nil {
                t.Error("expected error but got none")
            }
            if !tt.expectError && err != nil {
                t.Errorf("unexpected error: %v", err)
            }
        })
    }
}
```

### Integration Testing
```go
func TestResumeWorkflowIntegration(t *testing.T) {
    // Setup test workflow
    workflow := createTestWorkflow("test-task-5")
    workflow.Labels = map[string]string{
        "task-id":       "5",
        "current-stage": "waiting-pr-created",
        "workflow-type": "play-orchestration",
    }

    // Submit workflow and wait for suspension
    client := fake.NewSimpleClientset(workflow)

    // Create resume specification
    resumeSpec := &WorkflowResumeSpec{
        LabelSelector: "task-id=5,current-stage=waiting-pr-created",
        CorrelationData: CorrelationData{
            TaskId:    "5",
            EventType: "pr-created",
        },
    }

    // Test resume operation
    ctx := context.Background()
    err := ResumeWorkflow(ctx, resumeSpec)

    if err != nil {
        t.Errorf("resume operation failed: %v", err)
    }

    // Verify workflow resumed
    updatedWorkflow, _ := client.Get(ctx, workflow.Name, metav1.GetOptions{})
    if updatedWorkflow.Spec.Suspend == nil || *updatedWorkflow.Spec.Suspend {
        t.Error("workflow not resumed - still suspended")
    }
}
```

## Performance Considerations

### Resume Operation Optimization
- **Parallel Processing**: Handle multiple resume requests concurrently
- **Connection Pooling**: Reuse connections to Argo API server
- **Caching**: Cache workflow lookups for recent operations
- **Batch Operations**: Group multiple resume operations when possible

### Resource Management
- **Rate Limiting**: Prevent overwhelming Argo API server
- **Memory Management**: Clean up resources after resume operations
- **Connection Limits**: Manage concurrent connections to external services
- **Timeout Management**: Appropriate timeouts for all operations

## Dependencies

- **Task 5**: Event-driven workflow coordination foundation
- **Task 7**: Comprehensive workflow event handling
- **Task 10**: Workflow suspend/resume infrastructure
- Argo Workflows API access
- Argo Events webhook processing
- Kubernetes API server connectivity

## Expected Outcomes

### Functional Outcomes
1. **Reliable Resume Operations**: 99%+ success rate for valid resume requests
2. **Event Correlation Accuracy**: Correct workflow identification from GitHub events
3. **Failure Recovery**: Automatic retry for transient failures
4. **Circuit Breaker Protection**: System protection from cascading failures

### Operational Outcomes
1. **Improved Reliability**: Reduced manual intervention for stuck workflows
2. **Better Observability**: Comprehensive logging and monitoring of resume operations
3. **Faster Recovery**: Quick identification and resolution of issues
4. **Scalable Operations**: Handle increasing workflow volume efficiently

## Future Enhancements

- **Predictive Resume**: Use ML to predict resume success probability
- **Advanced Correlation**: Support for complex event patterns
- **Resume Scheduling**: Queue and batch resume operations
- **Cross-Cluster Resume**: Support for distributed workflow systems
- **Resume Analytics**: Deep insights into workflow resume patterns
