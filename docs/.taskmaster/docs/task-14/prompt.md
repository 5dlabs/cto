# Autonomous Agent Prompt: Build Workflow Resume Operations

## 🚨 CRITICAL: Argo Events Reference Documentation

**BEFORE implementing ANY Argo Events sensors/triggers, MUST review official examples:**
- **Location:** [docs/references/argo-events/](../../../references/argo-events/)
- **Key Files:**
  - `github.yaml` - GitHub webhook sensor patterns
  - `complete-trigger-parameterization.yaml` - Dynamic parameter extraction  
  - `special-workflow-trigger.yaml` - ArgoWorkflow operations (submit/resume)
  - `trigger-standard-k8s-resource.yaml` - K8s resource creation patterns

**❌ UNSUPPORTED Operations (will cause deployment failures):**
- `operation: delete` ❌
- `operation: patch` ❌  
- `operation: update` ❌
- Template variables in `labelSelector` ❌

**✅ SUPPORTED Operations:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows)
- `dest: metadata.name` (dynamic targeting)

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!


## Mission

You are tasked with implementing robust workflow resume operations for the multi-agent orchestration system. This system must reliably resume suspended workflows at the correct points based on GitHub events, with comprehensive error handling, retry logic, and failure protection.

## Context

**System Architecture**: Multi-agent Play Workflow with event-driven suspend/resume patterns
- **Rex/Blaze Implementation** → Suspend → **GitHub PR Created** → Resume → **Cleo Quality**
- **Cleo Quality** → Suspend → **GitHub PR Labeled "ready-for-qa"** → Resume → **Tess Testing** 
- **Tess Testing** → Suspend → **GitHub PR Approved** → Resume → **Task Completion**

**Your Role**: Reliability engineer building bullet-proof workflow resume infrastructure

**Critical Problem**: Current suspend/resume operations are fragile and fail under various conditions:
- Network timeouts during resume operations
- Multiple workflows matching the same event
- Events arriving for non-existent workflows
- Event correlation failures between GitHub and workflow state

## Primary Objectives

### 1. Reliable Resume API
Implement resume operations with proper validation, event correlation, and comprehensive error handling.

### 2. Exponential Backoff Retry Logic
Add intelligent retry mechanisms that handle transient failures without overwhelming the system.

### 3. Circuit Breaker Pattern
Implement protection against cascading failures when resume operations consistently fail.

### 4. Comprehensive Event Correlation
Ensure GitHub webhook events correctly match and resume the appropriate workflow instances.

## Technical Implementation

### Phase 1: Core Resume Infrastructure

**Resume Operation Structure**:
```go
// Core resume function with full error handling
func ResumeWorkflow(ctx context.Context, resumeRequest *WorkflowResumeRequest) (*ResumeResult, error) {
    // Step 1: Validate the resume request
    if err := validateResumeRequest(resumeRequest); err != nil {
        return nil, fmt.Errorf("resume validation failed: %w", err)
    }
    
    // Step 2: Find the target workflow using correlation
    workflow, err := findTargetWorkflowWithCorrelation(ctx, resumeRequest)
    if err != nil {
        return nil, fmt.Errorf("workflow correlation failed: %w", err)
    }
    
    // Step 3: Validate workflow state for resume
    if err := validateWorkflowStateForResume(workflow, resumeRequest); err != nil {
        return nil, fmt.Errorf("workflow state validation failed: %w", err)
    }
    
    // Step 4: Execute resume with retry logic and circuit breaker
    return executeResumeWithProtection(ctx, workflow, resumeRequest)
}

type WorkflowResumeRequest struct {
    // Correlation data from GitHub event
    TaskId       string            `json:"taskId"`
    EventType    string            `json:"eventType"`
    PRNumber     int               `json:"prNumber"`
    EventPayload map[string]interface{} `json:"eventPayload"`
    
    // Workflow targeting
    LabelSelector string            `json:"labelSelector,omitempty"`
    WorkflowName  string            `json:"workflowName,omitempty"`
    
    // Resume parameters
    ResumeParameters map[string]string `json:"resumeParameters"`
    
    // Validation rules
    ValidationRules []ValidationRule  `json:"validationRules"`
}

type ResumeResult struct {
    Success          bool              `json:"success"`
    WorkflowName     string            `json:"workflowName"`
    ResumeTime       time.Time         `json:"resumeTime"`
    AttemptCount     int               `json:"attemptCount"`
    ValidationErrors []string          `json:"validationErrors,omitempty"`
    RetryDelays      []time.Duration   `json:"retryDelays,omitempty"`
}
```

### Phase 2: Event Correlation and Validation

**Comprehensive Event Correlation**:
```go
func findTargetWorkflowWithCorrelation(ctx context.Context, request *WorkflowResumeRequest) (*v1alpha1.Workflow, error) {
    argoClient := getArgoWorkflowsClient()
    
    // Build label selector for workflow targeting
    labelSelector := buildWorkflowLabelSelector(request)
    
    // Query workflows matching the criteria
    workflowList, err := argoClient.List(ctx, metav1.ListOptions{
        LabelSelector: labelSelector,
    })
    if err != nil {
        return nil, fmt.Errorf("workflow query failed: %w", err)
    }
    
    // Handle different scenarios
    switch len(workflowList.Items) {
    case 0:
        return handleNoWorkflowFound(request)
    case 1:
        return &workflowList.Items[0], nil
    default:
        return handleMultipleWorkflowsFound(workflowList.Items, request)
    }
}

func buildWorkflowLabelSelector(request *WorkflowResumeRequest) string {
    selectors := []string{
        "workflow-type=play-orchestration",
        fmt.Sprintf("task-id=%s", request.TaskId),
    }
    
    // Add event-specific stage selector
    switch request.EventType {
    case "pr-created":
        selectors = append(selectors, "current-stage=waiting-pr-created")
    case "pr-labeled-ready":
        selectors = append(selectors, "current-stage=waiting-ready-for-qa")
    case "pr-approved":
        selectors = append(selectors, "current-stage=waiting-pr-approved")
    }
    
    return strings.Join(selectors, ",")
}

func validateWorkflowStateForResume(workflow *v1alpha1.Workflow, request *WorkflowResumeRequest) error {
    // Check if workflow is actually suspended
    if !isWorkflowSuspended(workflow) {
        return fmt.Errorf("workflow %s is not suspended (current phase: %s)", 
            workflow.Name, workflow.Status.Phase)
    }
    
    // Validate task ID correlation
    workflowTaskId := workflow.Labels["task-id"]
    if workflowTaskId != request.TaskId {
        return fmt.Errorf("task ID mismatch: workflow=%s, event=%s", 
            workflowTaskId, request.TaskId)
    }
    
    // Validate workflow stage matches event expectation
    currentStage := workflow.Labels["current-stage"]
    expectedStage := getExpectedStageForEvent(request.EventType)
    if currentStage != expectedStage {
        return fmt.Errorf("stage mismatch: current=%s, expected=%s", 
            currentStage, expectedStage)
    }
    
    // Run custom validation rules
    return runValidationRules(workflow, request)
}
```

### Phase 3: Retry Logic with Exponential Backoff

**Robust Retry Implementation**:
```go
type RetryConfig struct {
    MaxAttempts      int           `json:"maxAttempts"`
    InitialDelay     time.Duration `json:"initialDelay"`
    MaxDelay         time.Duration `json:"maxDelay"`
    BackoffMultiplier float64      `json:"backoffMultiplier"`
    JitterFactor     float64      `json:"jitterFactor"`
}

func executeResumeWithProtection(ctx context.Context, workflow *v1alpha1.Workflow, request *WorkflowResumeRequest) (*ResumeResult, error) {
    // Initialize retry configuration
    retryConfig := RetryConfig{
        MaxAttempts:      3,
        InitialDelay:     2 * time.Second,
        MaxDelay:         30 * time.Second,
        BackoffMultiplier: 2.0,
        JitterFactor:     0.1,
    }
    
    // Use circuit breaker to protect against cascading failures
    circuitBreaker := getCircuitBreakerForWorkflow(workflow.Name)
    
    result := &ResumeResult{
        WorkflowName: workflow.Name,
        ResumeTime:   time.Now(),
    }
    
    var lastError error
    delay := retryConfig.InitialDelay
    
    for attempt := 1; attempt <= retryConfig.MaxAttempts; attempt++ {
        result.AttemptCount = attempt
        
        log.Printf("Resume attempt %d/%d for workflow %s (task %s)", 
            attempt, retryConfig.MaxAttempts, workflow.Name, request.TaskId)
        
        // Execute resume through circuit breaker
        err := circuitBreaker.Execute(func() error {
            return performActualResume(ctx, workflow, request)
        })
        
        if err == nil {
            result.Success = true
            log.Printf("Resume successful on attempt %d for workflow %s", 
                attempt, workflow.Name)
            return result, nil
        }
        
        lastError = err
        
        // Check if error is retryable
        if !isRetryableError(err) {
            result.ValidationErrors = []string{err.Error()}
            log.Printf("Non-retryable error for workflow %s: %v", 
                workflow.Name, err)
            break
        }
        
        // Circuit breaker may block retries
        if isCircuitBreakerOpen(err) {
            result.ValidationErrors = []string{"circuit breaker open - retry blocked"}
            log.Printf("Circuit breaker blocked retry for workflow %s", 
                workflow.Name)
            break
        }
        
        if attempt < retryConfig.MaxAttempts {
            // Add jitter to prevent thundering herd
            jitter := time.Duration(rand.Float64() * float64(delay) * retryConfig.JitterFactor)
            actualDelay := delay + jitter
            result.RetryDelays = append(result.RetryDelays, actualDelay)
            
            log.Printf("Resume failed on attempt %d, retrying in %v: %v", 
                attempt, actualDelay, err)
            
            select {
            case <-ctx.Done():
                return result, ctx.Err()
            case <-time.After(actualDelay):
                // Calculate next delay with exponential backoff
                delay = time.Duration(float64(delay) * retryConfig.BackoffMultiplier)
                if delay > retryConfig.MaxDelay {
                    delay = retryConfig.MaxDelay
                }
            }
        }
    }
    
    result.Success = false
    result.ValidationErrors = append(result.ValidationErrors, 
        fmt.Sprintf("resume failed after %d attempts: %v", retryConfig.MaxAttempts, lastError))
    
    return result, lastError
}

func isRetryableError(err error) bool {
    retryablePatterns := []string{
        "connection refused",
        "timeout",
        "temporary failure",
        "service unavailable",
        "internal server error",
        "too many requests",
        "network is unreachable",
    }
    
    errMsg := strings.ToLower(err.Error())
    for _, pattern := range retryablePatterns {
        if strings.Contains(errMsg, pattern) {
            return true
        }
    }
    
    return false
}
```

### Phase 4: Circuit Breaker Implementation

**Circuit Breaker Pattern**:
```go
type CircuitBreaker struct {
    name            string
    maxFailures     int
    resetTimeout    time.Duration
    
    mutex           sync.RWMutex
    failures        int
    lastFailureTime time.Time
    state          CircuitBreakerState
    onStateChange   func(from, to CircuitBreakerState)
}

type CircuitBreakerState int

const (
    CircuitClosed CircuitBreakerState = iota
    CircuitOpen
    CircuitHalfOpen
)

func NewCircuitBreaker(name string, maxFailures int, resetTimeout time.Duration) *CircuitBreaker {
    return &CircuitBreaker{
        name:         name,
        maxFailures:  maxFailures,
        resetTimeout: resetTimeout,
        state:       CircuitClosed,
        onStateChange: func(from, to CircuitBreakerState) {
            log.Printf("Circuit breaker %s: %s -> %s", name, 
                stateToString(from), stateToString(to))
        },
    }
}

func (cb *CircuitBreaker) Execute(operation func() error) error {
    cb.mutex.Lock()
    defer cb.mutex.Unlock()
    
    // Update circuit breaker state based on time and failures
    cb.updateState()
    
    if cb.state == CircuitOpen {
        return fmt.Errorf("circuit breaker %s is OPEN - operation blocked", cb.name)
    }
    
    // Execute the operation
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
    oldState := cb.state
    
    switch cb.state {
    case CircuitClosed:
        if cb.failures >= cb.maxFailures {
            cb.state = CircuitOpen
            cb.lastFailureTime = now
        }
    case CircuitOpen:
        if now.Sub(cb.lastFailureTime) >= cb.resetTimeout {
            cb.state = CircuitHalfOpen
        }
    case CircuitHalfOpen:
        // State managed by recordSuccess/recordFailure
    }
    
    if oldState != cb.state && cb.onStateChange != nil {
        cb.onStateChange(oldState, cb.state)
    }
}

func (cb *CircuitBreaker) recordSuccess() {
    oldState := cb.state
    cb.failures = 0
    
    if cb.state == CircuitHalfOpen {
        cb.state = CircuitClosed
    }
    
    if oldState != cb.state && cb.onStateChange != nil {
        cb.onStateChange(oldState, cb.state)
    }
}

func (cb *CircuitBreaker) recordFailure() {
    oldState := cb.state
    cb.failures++
    cb.lastFailureTime = time.Now()
    
    if cb.state == CircuitHalfOpen {
        cb.state = CircuitOpen
    }
    
    if oldState != cb.state && cb.onStateChange != nil {
        cb.onStateChange(oldState, cb.state)
    }
}
```

## Critical Success Criteria

### 1. Resume Operation Reliability
- [ ] 99%+ success rate for valid resume requests
- [ ] Proper correlation between GitHub events and workflow instances
- [ ] Comprehensive validation prevents invalid resume attempts
- [ ] Graceful handling of edge cases (multiple workflows, missing workflows)

### 2. Error Handling and Recovery
- [ ] Exponential backoff retry logic implemented correctly
- [ ] Circuit breaker protects against cascading failures
- [ ] Non-retryable errors handled without unnecessary retries
- [ ] Comprehensive error logging for troubleshooting

### 3. Event Correlation Accuracy
- [ ] Task ID extraction from GitHub events works reliably
- [ ] Workflow stage validation matches event types correctly
- [ ] Multiple event sources (PR labels, branch names) supported
- [ ] Event timestamp and ordering handled properly

### 4. Performance and Scalability
- [ ] Resume operations complete within 10 seconds for normal cases
- [ ] System handles concurrent resume requests efficiently
- [ ] Circuit breaker prevents system overload
- [ ] Resource usage remains bounded under load

## Implementation Strategy

### Step 1: Core Resume API Development
```go
// Create comprehensive resume service
type WorkflowResumeService struct {
    argoClient      argoclient.Interface
    circuitBreakers map[string]*CircuitBreaker
    retryConfig     RetryConfig
    logger          *log.Logger
    metrics         *ResumeMetrics
}

// Implement main resume operation
func (svc *WorkflowResumeService) ResumeWorkflow(ctx context.Context, request *WorkflowResumeRequest) (*ResumeResult, error) {
    // Implementation as detailed above
}

// Add comprehensive testing
func TestResumeWorkflowService(t *testing.T) {
    // Unit tests for all resume scenarios
}
```

### Step 2: Argo Events Integration
```yaml
# Enhanced Argo Events sensor for resume operations
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: workflow-resume-sensor
  namespace: argo-events
spec:
  dependencies:
  - name: github-pr-events
    eventSourceName: github-webhook
    eventName: pull-request
    filters:
      # Comprehensive event filtering
      data:
      - path: body.action
        type: string
        comparator: "regex"
        value: "^(opened|labeled|review_requested|submitted)$"
      - path: body.pull_request.labels[].name
        type: string
        comparator: "regex"
        value: "^task-[0-9]+$"
        
  triggers:
  - template:
      name: resume-workflow
      http:
        url: http://workflow-resume-service.argo.svc.cluster.local/resume
        method: POST
        headers:
          Content-Type: "application/json"
        payload:
        - src:
            dependencyName: github-pr-events
            dataTemplate: |
              {
                "taskId": "{{jq '.body.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]'}}",
                "eventType": "{{if eq .body.action "opened"}}pr-created{{else if eq .body.action "labeled"}}pr-labeled-ready{{else if eq .body.action "submitted"}}pr-approved{{end}}",
                "prNumber": {{.body.pull_request.number}},
                "eventPayload": {{toJson .body}}
              }
          dest: payload
```

### Step 3: Monitoring and Observability
```go
type ResumeMetrics struct {
    TotalResumeAttempts   prometheus.Counter
    SuccessfulResumes     prometheus.Counter
    FailedResumes         prometheus.Counter
    ResumeLatency        prometheus.Histogram
    CircuitBreakerStates prometheus.GaugeVec
    RetryAttempts        prometheus.Histogram
}

func (svc *WorkflowResumeService) recordMetrics(result *ResumeResult, duration time.Duration) {
    svc.metrics.TotalResumeAttempts.Inc()
    svc.metrics.ResumeLatency.Observe(duration.Seconds())
    
    if result.Success {
        svc.metrics.SuccessfulResumes.Inc()
    } else {
        svc.metrics.FailedResumes.Inc()
    }
    
    svc.metrics.RetryAttempts.Observe(float64(result.AttemptCount))
}
```

### Step 4: End-to-End Testing
```bash
# Create comprehensive test scenarios
# Test 1: Normal resume operation
kubectl apply -f test-workflow-suspended.yaml

# Simulate GitHub event
curl -X POST http://workflow-resume-service/resume \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "123",
    "eventType": "pr-created",
    "prNumber": 456,
    "eventPayload": {...}
  }'

# Verify workflow resumed
argo get test-workflow-123

# Test 2: Retry logic with temporary failures
# Simulate network issues during resume

# Test 3: Circuit breaker protection
# Simulate consistent failures to trigger circuit breaker

# Test 4: Event correlation edge cases
# Test multiple workflows, missing workflows, etc.
```

## Key Files to Create/Modify

### New Components
- `services/workflow-resume/main.go` - Resume service implementation
- `services/workflow-resume/circuit-breaker.go` - Circuit breaker implementation
- `services/workflow-resume/retry.go` - Retry logic with exponential backoff
- `services/workflow-resume/correlation.go` - Event correlation logic
- `services/workflow-resume/metrics.go` - Monitoring and metrics

### Enhanced Components
- `argo-events/sensors/workflow-resume-sensor.yaml` - Enhanced event processing
- `argo-events/event-sources/github-webhook.yaml` - Improved webhook handling

### Testing
- `test/resume-operations/` - Comprehensive test suite
- `test/integration/resume-e2e-test.yaml` - End-to-end integration tests

## Error Scenarios and Handling

### Scenario 1: Network Timeout During Resume
```go
func handleNetworkTimeout(ctx context.Context, workflow *v1alpha1.Workflow, request *WorkflowResumeRequest) error {
    log.Printf("Network timeout during resume for workflow %s - will retry", workflow.Name)
    
    // This is retryable - let retry logic handle it
    return fmt.Errorf("network timeout - retryable error")
}
```

### Scenario 2: Multiple Workflows Match Event
```go
func handleMultipleWorkflows(workflows []v1alpha1.Workflow, request *WorkflowResumeRequest) (*v1alpha1.Workflow, error) {
    log.Printf("Found %d workflows matching task %s - determining correct target", 
        len(workflows), request.TaskId)
    
    // Find most recent suspended workflow
    var target *v1alpha1.Workflow
    for _, wf := range workflows {
        if isWorkflowSuspended(&wf) && (target == nil || wf.CreationTimestamp.After(target.CreationTimestamp.Time)) {
            target = &wf
        }
    }
    
    if target == nil {
        return nil, fmt.Errorf("no suspended workflows found among %d candidates", len(workflows))
    }
    
    // Cancel older/duplicate workflows
    for _, wf := range workflows {
        if wf.Name != target.Name {
            log.Printf("Canceling duplicate workflow: %s", wf.Name)
            cancelWorkflow(wf.Name)
        }
    }
    
    return target, nil
}
```

### Scenario 3: Event for Non-existent Workflow
```go
func handleMissingWorkflow(request *WorkflowResumeRequest) error {
    log.Printf("No workflow found for task %s, event type %s", 
        request.TaskId, request.EventType)
    
    // Check if event arrived too late (workflow already completed)
    if isWorkflowAlreadyCompleted(request.TaskId) {
        log.Printf("Event for task %s arrived after workflow completion - ignoring", 
            request.TaskId)
        return nil // Don't retry late events
    }
    
    // Check if workflow creation is still pending
    if isWorkflowCreationPending(request.TaskId) {
        log.Printf("Workflow creation for task %s still pending - will retry", 
            request.TaskId)
        return fmt.Errorf("workflow creation pending - retryable")
    }
    
    // Unknown state - requires manual investigation
    return fmt.Errorf("workflow not found for unknown reason - manual investigation required")
}
```

## Testing Commands

### Unit Testing
```bash
# Test resume service components
go test ./services/workflow-resume/... -v

# Test circuit breaker implementation
go test ./services/workflow-resume/circuit-breaker_test.go -v

# Test retry logic
go test ./services/workflow-resume/retry_test.go -v

# Test event correlation
go test ./services/workflow-resume/correlation_test.go -v
```

### Integration Testing
```bash
# Deploy test environment
kubectl apply -f test/resume-operations/test-environment.yaml

# Run end-to-end resume tests
kubectl apply -f test/resume-operations/e2e-test-suite.yaml

# Monitor test results
kubectl logs -l app=resume-test-runner -f
```

## Expected Deliverables

1. **Resume Service Implementation**: Complete workflow resume service with API
2. **Retry Logic**: Exponential backoff retry mechanism with jitter
3. **Circuit Breaker**: Protection against cascading failures
4. **Event Correlation**: Robust GitHub event to workflow correlation
5. **Comprehensive Testing**: Unit, integration, and end-to-end tests
6. **Monitoring Integration**: Metrics, logging, and observability
7. **Documentation**: Operation guides and troubleshooting procedures

## Dependencies & Prerequisites

- **Task 5**: Event-driven workflow coordination functional
- **Task 7**: Workflow suspend/resume infrastructure
- **Task 10**: Event correlation foundation
- **Argo Workflows API**: Access for resume operations
- **Argo Events**: Webhook processing and event triggering
- **Kubernetes Cluster**: Sufficient resources for resume service

## Constraints

- **API Rate Limits**: Respect Argo API server rate limits
- **Resource Usage**: Bounded memory and CPU usage
- **Response Times**: Resume operations complete within reasonable time
- **Concurrent Safety**: Handle multiple resume requests safely

## Quality Gates

Before marking complete:
- [ ] Resume service handles all event types correctly
- [ ] Retry logic with exponential backoff implemented
- [ ] Circuit breaker protects against failures
- [ ] Event correlation accuracy >95%
- [ ] All error scenarios tested and handled
- [ ] Performance under load acceptable
- [ ] Monitoring and alerting configured
- [ ] Integration with existing multi-agent system verified

This implementation establishes reliable workflow resume operations that ensure the multi-agent orchestration system can handle failures gracefully and maintain high availability.