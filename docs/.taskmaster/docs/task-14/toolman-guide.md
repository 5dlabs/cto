# Toolman Guide: Build Workflow Resume Operations

## Overview

This guide provides comprehensive instructions for implementing robust workflow resume operations in the multi-agent orchestration system. The resume system ensures reliable workflow continuation after GitHub events, with comprehensive error handling, retry logic, and failure protection.

## Tool Recommendations

### Primary Tools for Resume Operations Implementation

#### 1. Research and Design
- **brave-search_brave_web_search**: Research workflow resume patterns, retry strategies, and circuit breaker implementations
- **memory_create_entities**: Store successful resume patterns, error handling strategies, and troubleshooting solutions
- **memory_query_entities**: Retrieve stored knowledge about workflow resume operations and failure scenarios

#### 2. Implementation and Testing
- **mcp-kubernetes**: Direct Kubernetes interaction for workflow monitoring and testing
- **memory_create_entities**: Document implementation patterns and testing procedures

### Tool Usage Patterns

#### Phase 1: Research and Architecture Design

```bash
# Use brave-search_brave_web_search for research
Search: "workflow resume operations retry patterns exponential backoff"
Search: "circuit breaker pattern microservices reliability"
Search: "event correlation patterns distributed systems"
Search: "argo workflows suspend resume api best practices"
Search: "kubernetes event-driven architecture reliability patterns"
```

#### Phase 2: Implementation Knowledge Management

```bash
# Use memory_create_entities to store patterns
Store: "Resume API validation patterns and error handling strategies"
Store: "Event correlation algorithms for GitHub webhook processing"
Store: "Circuit breaker implementation with exponential backoff"
Store: "Retry logic patterns with jitter and maximum attempts"
Store: "Testing strategies for resume operations and failure scenarios"

# Use memory_query_entities to retrieve knowledge
Query: "workflow resume validation patterns"
Query: "event correlation failure handling"
Query: "circuit breaker implementation strategies"
Query: "resume operation testing procedures"
```

#### Phase 3: Testing and Validation

```bash
# Use mcp-kubernetes for direct testing
List workflows: kubectl get workflows -l workflow-type=play-orchestration
Monitor workflow status: kubectl get workflow <name> -o yaml
Test resume operations: kubectl patch workflow <name> --type='merge' -p='{"spec":{"suspend":null}}'
```

## Best Practices

### 1. Resume API Design Patterns

**Comprehensive Validation Structure**:
```go
type WorkflowResumeRequest struct {
    // Core correlation data
    TaskId       string            `json:"taskId" validate:"required,min=1"`
    EventType    string            `json:"eventType" validate:"required,oneof=pr-created pr-labeled-ready pr-approved"`

    // GitHub event context
    PRNumber     int               `json:"prNumber,omitempty"`
    EventPayload map[string]interface{} `json:"eventPayload" validate:"required"`

    // Resume targeting
    LabelSelector string            `json:"labelSelector,omitempty"`
    WorkflowName  string            `json:"workflowName,omitempty"`

    // Operation parameters
    ResumeParameters map[string]string `json:"resumeParameters,omitempty"`
    ValidationRules  []ValidationRule  `json:"validationRules,omitempty"`

    // Request metadata
    RequestId    string            `json:"requestId"`
    Timestamp    time.Time         `json:"timestamp"`
}

type ValidationRule struct {
    Field    string `json:"field"`
    Operator string `json:"operator"` // equals, regex, exists, etc.
    Value    string `json:"value"`
    Required bool   `json:"required"`
}
```

### 2. Event Correlation Best Practices

**Multi-Source Task ID Extraction**:
```go
func extractTaskIdFromEvent(event *GitHubWebhookEvent) (string, error) {
    // Priority 1: Extract from PR labels
    for _, label := range event.PullRequest.Labels {
        if strings.HasPrefix(label.Name, "task-") {
            parts := strings.Split(label.Name, "-")
            if len(parts) >= 2 {
                taskId := parts[1]
                if isValidTaskId(taskId) {
                    return taskId, nil
                }
            }
        }
    }

    // Priority 2: Extract from branch name
    branchName := event.PullRequest.Head.Ref
    if strings.HasPrefix(branchName, "task-") {
        parts := strings.Split(branchName, "-")
        if len(parts) >= 2 {
            taskId := parts[1]
            if isValidTaskId(taskId) {
                return taskId, nil
            }
        }
    }

    // Priority 3: Extract from PR title or body
    taskIdPattern := regexp.MustCompile(`task[- ]?([0-9]+)`)
    if matches := taskIdPattern.FindStringSubmatch(event.PullRequest.Title); len(matches) > 1 {
        return matches[1], nil
    }

    return "", fmt.Errorf("no task ID found in event")
}

func isValidTaskId(taskId string) bool {
    if taskId == "" {
        return false
    }

    // Validate numeric format
    if _, err := strconv.Atoi(taskId); err != nil {
        return false
    }

    // Additional validation rules
    taskNum, _ := strconv.Atoi(taskId)
    return taskNum > 0 && taskNum < 10000 // Reasonable bounds
}
```

### 3. Retry Logic with Exponential Backoff

**Advanced Retry Configuration**:
```go
type RetryConfig struct {
    MaxAttempts       int           `json:"maxAttempts"`
    InitialDelay      time.Duration `json:"initialDelay"`
    MaxDelay          time.Duration `json:"maxDelay"`
    BackoffMultiplier float64       `json:"backoffMultiplier"`
    JitterFactor      float64       `json:"jitterFactor"`
    RetryableErrors   []string      `json:"retryableErrors"`
}

func (rc *RetryConfig) CalculateDelay(attempt int) time.Duration {
    if attempt <= 1 {
        return rc.InitialDelay
    }

    // Exponential backoff
    delay := time.Duration(float64(rc.InitialDelay) * math.Pow(rc.BackoffMultiplier, float64(attempt-1)))

    // Apply maximum delay cap
    if delay > rc.MaxDelay {
        delay = rc.MaxDelay
    }

    // Add jitter to prevent thundering herd
    jitter := time.Duration(rand.Float64() * float64(delay) * rc.JitterFactor)

    return delay + jitter
}

func (rc *RetryConfig) IsRetryable(err error) bool {
    if err == nil {
        return false
    }

    errMsg := strings.ToLower(err.Error())
    for _, retryablePattern := range rc.RetryableErrors {
        if strings.Contains(errMsg, strings.ToLower(retryablePattern)) {
            return true
        }
    }

    return false
}

// Default retry configuration
func DefaultRetryConfig() *RetryConfig {
    return &RetryConfig{
        MaxAttempts:       3,
        InitialDelay:      2 * time.Second,
        MaxDelay:          30 * time.Second,
        BackoffMultiplier: 2.0,
        JitterFactor:      0.1,
        RetryableErrors: []string{
            "connection refused",
            "timeout",
            "temporary failure",
            "service unavailable",
            "internal server error",
            "too many requests",
            "network is unreachable",
        },
    }
}
```

### 4. Circuit Breaker Implementation

**Production-Ready Circuit Breaker**:
```go
type CircuitBreaker struct {
    name            string
    maxFailures     int
    resetTimeout    time.Duration
    halfOpenMaxCalls int

    mutex           sync.RWMutex
    failures        int
    successes       int
    lastFailureTime time.Time
    state          CircuitBreakerState

    // Callbacks
    onStateChange   func(from, to CircuitBreakerState)
    onFailure       func(err error)
    onSuccess       func(duration time.Duration)
}

func (cb *CircuitBreaker) Execute(operation func() error) error {
    cb.mutex.Lock()

    // Check if we can execute
    if !cb.canExecute() {
        cb.mutex.Unlock()
        return fmt.Errorf("circuit breaker %s is OPEN - operation blocked", cb.name)
    }

    cb.mutex.Unlock()

    // Execute operation with timing
    start := time.Now()
    err := operation()
    duration := time.Since(start)

    // Record result
    if err != nil {
        cb.recordFailure(err)
        return err
    }

    cb.recordSuccess(duration)
    return nil
}

func (cb *CircuitBreaker) canExecute() bool {
    cb.updateState()

    switch cb.state {
    case CircuitClosed:
        return true
    case CircuitOpen:
        return false
    case CircuitHalfOpen:
        return cb.successes + cb.failures < cb.halfOpenMaxCalls
    default:
        return false
    }
}

func (cb *CircuitBreaker) recordSuccess(duration time.Duration) {
    cb.mutex.Lock()
    defer cb.mutex.Unlock()

    oldState := cb.state

    if cb.state == CircuitHalfOpen {
        cb.successes++
        if cb.successes >= cb.halfOpenMaxCalls {
            cb.state = CircuitClosed
            cb.failures = 0
            cb.successes = 0
        }
    } else {
        cb.failures = 0
    }

    if oldState != cb.state && cb.onStateChange != nil {
        cb.onStateChange(oldState, cb.state)
    }

    if cb.onSuccess != nil {
        cb.onSuccess(duration)
    }
}
```

## Common Workflows

### Workflow 1: Complete Resume Service Implementation

1. **Research and Design Phase**
   ```bash
   # Use brave-search_brave_web_search
   Search: "workflow resume patterns microservices reliability"
   Search: "event correlation distributed systems best practices"
   Search: "circuit breaker pattern golang implementation"

   # Store research findings
   memory_create_entities("Resume Patterns", {
     "topic": "workflow-resume-reliability-patterns",
     "patterns": [
       "Idempotent resume operations",
       "Event correlation with multiple validation sources",
       "Circuit breaker with half-open state",
       "Exponential backoff with jitter"
     ],
     "best_practices": [
       "Validate event correlation before resume",
       "Use request IDs for deduplication",
       "Implement comprehensive logging",
       "Monitor circuit breaker state transitions"
     ]
   })
   ```

2. **Core Service Development**
   ```go
   // services/workflow-resume/main.go
   package main

   import (
       "context"
       "encoding/json"
       "fmt"
       "log"
       "net/http"
       "time"

       "github.com/gorilla/mux"
       "github.com/prometheus/client_golang/prometheus/promhttp"
   )

   type ResumeService struct {
       argoClient      ArgoWorkflowsClient
       circuitBreakers map[string]*CircuitBreaker
       retryConfig     *RetryConfig
       metrics         *ResumeMetrics
   }

   func (svc *ResumeService) HandleResumeRequest(w http.ResponseWriter, r *http.Request) {
       var request WorkflowResumeRequest
       if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
           http.Error(w, fmt.Sprintf("Invalid request: %v", err), http.StatusBadRequest)
           return
       }

       result, err := svc.ResumeWorkflow(r.Context(), &request)
       if err != nil {
           http.Error(w, err.Error(), http.StatusInternalServerError)
           return
       }

       w.Header().Set("Content-Type", "application/json")
       json.NewEncoder(w).Encode(result)
   }
   ```

3. **Testing and Validation**
   ```bash
   # Use mcp-kubernetes for testing
   # Create test suspended workflow
   kubectl apply -f - <<EOF
   apiVersion: argoproj.io/v1alpha1
   kind: Workflow
   metadata:
     name: test-resume-workflow
     labels:
       task-id: "100"
       current-stage: waiting-pr-created
   spec:
     suspend: {}
   EOF

   # Test resume operation
   curl -X POST http://localhost:8080/resume \
     -H "Content-Type: application/json" \
     -d '{
       "taskId": "100",
       "eventType": "pr-created",
       "eventPayload": {
         "pull_request": {
           "labels": [{"name": "task-100"}]
         }
       }
     }'

   # Verify workflow resumed
   kubectl get workflow test-resume-workflow -o jsonpath='{.status.phase}'
   ```

### Workflow 2: Error Handling and Circuit Breaker Testing

1. **Circuit Breaker Implementation**
   ```go
   // Test circuit breaker functionality
   func TestCircuitBreakerFlow(t *testing.T) {
       cb := NewCircuitBreaker("test", 3, 30*time.Second)

       // Test closed state - operations should succeed
       err := cb.Execute(func() error { return nil })
       assert.NoError(t, err)

       // Force failures to open circuit
       for i := 0; i < 3; i++ {
           cb.Execute(func() error { return fmt.Errorf("test error") })
       }

       // Test open state - operations should be blocked
       err = cb.Execute(func() error { return nil })
       assert.Error(t, err)
       assert.Contains(t, err.Error(), "circuit breaker.*OPEN")
   }
   ```

2. **Retry Logic Testing**
   ```bash
   # Test retry behavior with temporary failures
   # Use memory_create_entities to store test scenarios
   memory_create_entities("Retry Test Scenarios", {
     "transient_failures": [
       "Network timeout during API call",
       "Service temporarily unavailable",
       "Rate limit exceeded"
     ],
     "permanent_failures": [
       "Workflow not found",
       "Invalid request format",
       "Authorization failed"
     ],
     "expected_behavior": {
       "transient": "Should retry with exponential backoff",
       "permanent": "Should fail immediately without retry"
     }
   })
   ```

3. **End-to-End Integration Testing**
   ```bash
   # Deploy complete test environment
   kubectl apply -f test/resume-operations/test-environment.yaml

   # Run comprehensive test suite
   go test ./test/resume-operations/... -v -timeout 10m

   # Monitor test metrics
   curl http://workflow-resume-service/metrics | grep resume_
   ```

### Workflow 3: Production Deployment and Monitoring

1. **Deployment Configuration**
   ```yaml
   # kubernetes/workflow-resume-service.yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: workflow-resume-service
     namespace: argo
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: workflow-resume-service
     template:
       metadata:
         labels:
           app: workflow-resume-service
       spec:
         containers:
         - name: resume-service
           image: workflow-resume-service:latest
           ports:
           - containerPort: 8080
             name: http
           - containerPort: 9090
             name: metrics
           env:
           - name: ARGO_SERVER_URL
             value: "argo-server.argo.svc.cluster.local:2746"
           - name: CIRCUIT_BREAKER_MAX_FAILURES
             value: "5"
           - name: CIRCUIT_BREAKER_RESET_TIMEOUT
             value: "60s"
           livenessProbe:
             httpGet:
               path: /health
               port: 8080
           readinessProbe:
             httpGet:
               path: /ready
               port: 8080
   ```

2. **Monitoring and Alerting Setup**
   ```yaml
   # monitoring/resume-service-alerts.yaml
   apiVersion: monitoring.coreos.com/v1
   kind: PrometheusRule
   metadata:
     name: resume-service-alerts
   spec:
     groups:
     - name: resume-operations
       rules:
       - alert: HighResumeFailureRate
         expr: rate(resume_failed_total[5m]) / rate(resume_total_attempts[5m]) > 0.1
         for: 2m
         annotations:
           summary: "High failure rate for resume operations"

       - alert: CircuitBreakerOpen
         expr: circuit_breaker_state{state="open"} == 1
         for: 1m
         annotations:
           summary: "Circuit breaker is open - blocking resume operations"
   ```

## Troubleshooting Guide

### Issue 1: Event Correlation Failures
**Symptoms**: Resume requests fail with "task ID mismatch" or "workflow not found"

**Diagnosis**:
```bash
# Use memory_query_entities to get troubleshooting procedures
memory_query_entities("event correlation troubleshooting")

# Check event payload structure
curl -X POST http://workflow-resume-service/debug/event-correlation \
  -d '{"eventPayload": {...}}' | jq '.extractedTaskId'

# Verify workflow labels
kubectl get workflows -l task-id=<task-id> --show-labels

# Check GitHub webhook payload
kubectl logs -l app=argo-events-controller | grep "event correlation"
```

**Resolution**:
1. Verify PR labeling follows convention (task-X format)
2. Check branch naming includes task ID
3. Validate workflow labels are set correctly
4. Review event payload transformation in Argo Events

### Issue 2: High Resume Failure Rate
**Symptoms**: Multiple resume operations failing consistently

**Diagnosis**:
```bash
# Check circuit breaker state
curl http://workflow-resume-service/metrics | grep circuit_breaker_state

# Review failure patterns
curl http://workflow-resume-service/metrics | grep resume_failed_total

# Check Argo API server connectivity
kubectl get pods -n argo | grep argo-server
kubectl logs -n argo deployment/argo-server | grep -i error
```

**Resolution**:
1. Verify Argo Workflows API server is healthy
2. Check network connectivity between resume service and Argo
3. Review circuit breaker configuration
4. Analyze error patterns in logs

### Issue 3: Resume Operation Timeouts
**Symptoms**: Resume requests taking too long or timing out

**Diagnosis**:
```bash
# Check resume latency metrics
curl http://workflow-resume-service/metrics | grep resume_latency_seconds

# Monitor retry attempts
kubectl logs -l app=workflow-resume-service | grep "Resume attempt"

# Check resource usage
kubectl top pods -l app=workflow-resume-service
```

**Resolution**:
1. Review retry configuration and reduce if needed
2. Check Argo API server performance
3. Scale resume service replicas if needed
4. Optimize workflow query performance

### Issue 4: Duplicate Resume Operations
**Symptoms**: Multiple resume requests for same workflow causing conflicts

**Diagnosis**:
```bash
# Check for duplicate events
kubectl logs -l app=argo-events-sensor | grep "duplicate\|concurrent"

# Review resume request IDs
curl http://workflow-resume-service/debug/recent-requests | jq '.[] | {requestId, taskId, timestamp}'

# Check workflow state
kubectl get workflow <name> -o yaml | grep -A 10 status
```

**Resolution**:
1. Implement request deduplication using request IDs
2. Add idempotency checks in resume logic
3. Review event source configuration for duplicates
4. Implement workflow state validation before resume

## Tool-Specific Tips

### brave-search_brave_web_search
- Search for "golang circuit breaker pattern implementation"
- Look for "microservices reliability patterns retry exponential backoff"
- Research "event correlation distributed systems best practices"
- Find "workflow orchestration error handling strategies"

### memory_create_entities / memory_query_entities
- Store successful implementation patterns and configurations
- Document error handling strategies and troubleshooting procedures
- Keep track of testing scenarios and validation approaches
- Record performance benchmarks and optimization techniques

### mcp-kubernetes
- Use for direct workflow monitoring and testing
- Implement workflow state validation
- Test resume operations in real environments
- Monitor resource usage and performance

## Quality Checks

### Pre-Implementation Checklist
- [ ] Research completed on resume operation patterns
- [ ] Circuit breaker and retry strategies designed
- [ ] Event correlation requirements understood
- [ ] Testing strategy planned with failure scenarios

### Implementation Checklist
- [ ] Resume API with comprehensive validation implemented
- [ ] Event correlation logic handles multiple sources
- [ ] Retry logic with exponential backoff and jitter
- [ ] Circuit breaker pattern with proper state transitions
- [ ] Comprehensive error handling for all scenarios

### Post-Implementation Checklist
- [ ] End-to-end testing demonstrates reliability
- [ ] Performance acceptable under load
- [ ] Circuit breaker protects against failures
- [ ] Monitoring and alerting configured
- [ ] Integration with Argo Events works correctly

## Success Indicators

1. **Reliability Success**:
   - 99%+ resume success rate for valid requests
   - Event correlation accuracy >95%
   - Circuit breaker effectively protects against failures

2. **Performance Success**:
   - Resume operations complete within 10 seconds
   - Service handles concurrent requests efficiently
   - Resource usage remains bounded under load

3. **Integration Success**:
   - Seamless integration with Argo Events
   - Multi-agent workflow compatibility maintained
   - GitHub webhook processing works reliably

## Performance Optimization

### Resume Service Optimization
- Implement connection pooling to Argo API
- Cache workflow lookups for recent operations
- Use efficient label selectors for workflow queries
- Optimize event correlation algorithms

### Circuit Breaker Optimization
- Tune failure thresholds based on actual patterns
- Implement adaptive timeout periods
- Use health checks for faster recovery detection
- Monitor and adjust based on operational data

### Retry Logic Optimization
- Analyze failure patterns to optimize retry conditions
- Implement intelligent jitter based on load
- Use context-aware timeouts
- Add request prioritization for critical operations

This guide provides the foundation for building robust workflow resume operations that ensure high reliability and availability in the multi-agent orchestration system.
