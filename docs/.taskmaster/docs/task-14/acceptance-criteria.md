# Acceptance Criteria: Build Workflow Resume Operations



## Overview

This document defines the specific, testable criteria that must be met to consider Task 14 (Build Workflow Resume Operations) complete. All criteria must pass before the task can be approved and merged.

## Functional Requirements

### FR-1: Resume Operation Validation
**Requirement**: Resume requests are thoroughly validated before execution

**Test Cases**:


- [ ] Resume request with valid task ID and event type passes validation


- [ ] Resume request missing required fields fails validation with clear error


- [ ] Resume request with invalid correlation data fails validation


- [ ] Validation includes event payload structure verification


- [ ] Validation rules are configurable and extensible

**Verification**:



```bash


# Test valid resume request
curl -X POST http://workflow-resume-service/resume \
  -H "Content-Type: application/json" \


  -d '{
    "taskId": "5",
    "eventType": "pr-created",
    "prNumber": 123,
    "eventPayload": {
      "action": "opened",
      "pull_request": {
        "number": 123,
        "labels": [{"name": "task-5"}]
      }
    }
  }' | jq '.success'

# Test invalid resume request (missing taskId)
curl -X POST http://workflow-resume-service/resume \
  -H "Content-Type: application/json" \


  -d '{
    "eventType": "pr-created",
    "prNumber": 123
  }' | jq '.validationErrors[]'

# Should return validation error about missing taskId






```

### FR-2: Event Correlation Accuracy
**Requirement**: GitHub events correctly correlate with target workflows

**Test Cases**:


- [ ] Task ID extracted correctly from PR labels


- [ ] Task ID extracted from branch name as fallback


- [ ] Event type determination matches GitHub action types


- [ ] Workflow stage validation ensures correct suspend point


- [ ] Multiple correlation methods validated for consistency

**Verification**:



```bash


# Create test workflow with specific labels
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-workflow-task-5
  namespace: argo
  labels:
    workflow-type: play-orchestration
    task-id: "5"
    current-stage: waiting-pr-created
spec:
  suspend: {}
EOF

# Test event correlation with PR label
curl -X POST http://workflow-resume-service/resume \
  -H "Content-Type: application/json" \


  -d '{
    "taskId": "5",
    "eventType": "pr-created",
    "eventPayload": {
      "pull_request": {
        "labels": [{"name": "task-5"}],
        "head": {"ref": "task-5-implement-feature"}
      }
    }
  }'

# Verify workflow was found and resumed
kubectl get workflow test-workflow-task-5 -o jsonpath='{.status.phase}' | grep -v Suspended

# Test correlation failure with mismatched task ID
curl -X POST http://workflow-resume-service/resume \
  -H "Content-Type: application/json" \


  -d '{
    "taskId": "6",
    "eventType": "pr-created",
    "eventPayload": {
      "pull_request": {
        "labels": [{"name": "task-5"}]
      }
    }
  }' | jq '.validationErrors[]' | grep "task ID mismatch"






```

### FR-3: Workflow State Validation
**Requirement**: Resume operations only proceed on workflows in correct state

**Test Cases**:


- [ ] Suspended workflows can be resumed


- [ ] Non-suspended workflows reject resume attempts


- [ ] Workflow stage matches expected stage for event type


- [ ] Completed workflows cannot be resumed


- [ ] Failed workflows handled appropriately

**Verification**:



```bash
# Test suspended workflow resume
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-suspended-workflow
  labels:
    task-id: "10"
    current-stage: waiting-pr-created
spec:
  suspend: {}
EOF



# Should succeed
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "10", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" | jq '.success'

# Test running workflow resume attempt
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-running-workflow
  labels:
    task-id: "11"
    current-stage: waiting-pr-created
status:
  phase: Running
EOF

# Should fail with validation error
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "11", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" | jq '.validationErrors[]' | grep "not suspended"






```

### FR-4: Resume Operation Execution
**Requirement**: Resume operations successfully resume suspended workflows

**Test Cases**:


- [ ] Suspended workflow resumes and continues execution


- [ ] Resume parameters passed correctly to workflow


- [ ] Workflow labels updated after resume


- [ ] Resume timestamp recorded


- [ ] Workflow events generated for resume operation

**Verification**:



```bash
# Create suspended workflow
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-resume-execution
  labels:
    task-id: "12"
    current-stage: waiting-pr-created
spec:
  suspend: {}
  templates:
  - name: test-template
    script:
      image: alpine
      command: [sh]
      source: echo "Workflow resumed"
EOF

# Execute resume operation
curl -X POST http://workflow-resume-service/resume \


  -d '{
    "taskId": "12",
    "eventType": "pr-created",
    "resumeParameters": {
      "pr-number": "123",
      "pr-url": "https://github.com/repo/pull/123"
    }
  }' -H "Content-Type: application/json"



# Wait for workflow to resume
sleep 5

# Verify workflow is no longer suspended
kubectl get workflow test-resume-execution -o jsonpath='{.status.phase}' | grep -v Suspended



# Verify resume parameters were passed
kubectl get workflow test-resume-execution -o yaml | grep -A 5 "parameters"






```

## Error Handling Requirements

### EH-1: Retry Logic Implementation
**Requirement**: Transient failures trigger retry with exponential backoff

**Test Cases**:


- [ ] Network timeout errors trigger retry


- [ ] Service unavailable errors trigger retry


- [ ] Maximum retry count prevents infinite loops


- [ ] Exponential backoff increases delay between retries


- [ ] Jitter prevents thundering herd issues

**Verification**:



```bash
# Simulate network timeout by making API server temporarily unavailable
kubectl scale deployment argo-server --replicas=0 -n argo

# Attempt resume operation
start_time=$(date +%s)
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "13", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" > /tmp/resume_result.json
end_time=$(date +%s)



# Restore API server
kubectl scale deployment argo-server --replicas=1 -n argo



# Verify retry attempts were made
jq '.attemptCount' /tmp/resume_result.json | grep -E '^[2-3]$'  # Should be 2 or 3 attempts

# Verify total time indicates retries with backoff
echo $((end_time - start_time)) | grep -E '^[5-9][0-9]*$'  # Should be more than 5 seconds

# Check retry delays in response
jq '.retryDelays[]' /tmp/resume_result.json






```

### EH-2: Circuit Breaker Protection
**Requirement**: Circuit breaker protects against cascading failures

**Test Cases**:


- [ ] Circuit breaker opens after maximum failures reached


- [ ] Open circuit breaker blocks operations immediately


- [ ] Circuit breaker transitions to half-open after timeout


- [ ] Successful operations in half-open state close circuit


- [ ] Failed operations in half-open state reopen circuit

**Verification**:



```bash
# Force circuit breaker to open by causing multiple failures
for i in {1..5}; do
  curl -X POST http://workflow-resume-service/resume \
    -d '{"taskId": "999", "eventType": "invalid-event"}' \
    -H "Content-Type: application/json" > /dev/null
done

# Attempt operation with open circuit breaker
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "14", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" | jq '.validationErrors[]' | grep "circuit breaker.*OPEN"



# Wait for circuit breaker reset timeout
sleep 35

# Verify circuit breaker allows operations after timeout
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "14", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" | jq '.success'






```

### EH-3: Non-Retryable Error Handling
**Requirement**: Non-retryable errors fail immediately without retry

**Test Cases**:


- [ ] Validation errors are not retried


- [ ] Authorization errors are not retried


- [ ] Resource not found errors are not retried


- [ ] Malformed request errors are not retried


- [ ] Clear error messages provided for non-retryable errors

**Verification**:



```bash
# Test non-retryable validation error
start_time=$(date +%s)
curl -X POST http://workflow-resume-service/resume \
  -d '{"invalidField": "value"}' \
  -H "Content-Type: application/json" > /tmp/validation_error.json
end_time=$(date +%s)

# Verify no retries (should fail quickly)
echo $((end_time - start_time)) | grep -E '^[0-4]$'  # Should be less than 5 seconds

# Verify single attempt
jq '.attemptCount' /tmp/validation_error.json | grep '^1$'



# Verify clear error message
jq '.validationErrors[]' /tmp/validation_error.json | grep -i validation






```

## Performance Requirements

### PR-1: Resume Operation Latency
**Requirement**: Resume operations complete within acceptable time limits

**Test Cases**:


- [ ] Successful resume operations complete within 10 seconds


- [ ] Failed resume operations fail within 5 seconds (for immediate failures)


- [ ] Retry operations complete within 60 seconds total


- [ ] Circuit breaker blocks operations within 1 second


- [ ] Concurrent resume operations don't significantly impact latency

**Verification**:



```bash
# Test successful resume latency
kubectl apply -f test-suspended-workflow.yaml

start_time=$(date +%s%3N)  # Milliseconds
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "15", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" > /tmp/resume_timing.json
end_time=$(date +%s%3N)

latency=$((end_time - start_time))
echo "Resume latency: ${latency}ms"

# Verify latency is under 10 seconds (10000ms)
test $latency -lt 10000

# Test concurrent resume operations
for i in {1..10}; do
  kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: concurrent-test-$i
  labels:
    task-id: "$((100+i))"
    current-stage: waiting-pr-created
spec:
  suspend: {}
EOF

  curl -X POST http://workflow-resume-service/resume \
    -d '{"taskId": "'$((100+i))'", "eventType": "pr-created"}' \
    -H "Content-Type: application/json" &
done

wait

# Verify all operations completed successfully
for i in {1..10}; do
  kubectl get workflow concurrent-test-$i -o jsonpath='{.status.phase}' | grep -v Suspended
done






```



### PR-2: Resource Usage
**Requirement**: Resume service uses resources efficiently

**Test Cases**:


- [ ] Memory usage remains stable under load


- [ ] CPU usage scales appropriately with request volume


- [ ] No memory leaks in long-running operations


- [ ] Connection pooling prevents resource exhaustion


- [ ] Graceful degradation under resource pressure

**Verification**:



```bash
# Monitor resource usage during load test
kubectl top pods -l app=workflow-resume-service &

# Generate load
for i in {1..100}; do
  kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: load-test-$i
  labels:
    task-id: "$((200+i))"
    current-stage: waiting-pr-created
spec:
  suspend: {}
EOF

  curl -X POST http://workflow-resume-service/resume \
    -d '{"taskId": "'$((200+i))'", "eventType": "pr-created"}' \
    -H "Content-Type: application/json" &

  if [ $((i % 10)) -eq 0 ]; then
    wait
    echo "Completed $i requests"
    kubectl top pods -l app=workflow-resume-service
  fi
done

wait

# Verify service remains responsive
curl -X GET http://workflow-resume-service/health | jq '.status' | grep "healthy"






```

## Integration Requirements

### IR-1: Argo Events Integration
**Requirement**: Resume service integrates seamlessly with Argo Events

**Test Cases**:


- [ ] Argo Events sensor triggers resume operations correctly


- [ ] Event payload transformation works as expected


- [ ] Resume service receives properly formatted requests


- [ ] Error responses from resume service handled by Argo Events


- [ ] Event correlation IDs maintained through pipeline

**Verification**:



```bash
# Deploy test Argo Events sensor
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: test-resume-sensor
  namespace: argo-events
spec:
  dependencies:
  - name: test-webhook
    eventSourceName: test-webhook-source
    eventName: test-event
  triggers:
  - template:
      name: test-resume
      http:
        url: http://workflow-resume-service.argo.svc.cluster.local/resume
        method: POST
        payload:
        - src:
            dependencyName: test-webhook
            dataTemplate: |
              {
                "taskId": "16",
                "eventType": "pr-created",
                "eventPayload": {{toJson .}}
              }
          dest: payload
EOF



# Create test workflow
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-argo-events-integration
  labels:
    task-id: "16"
    current-stage: waiting-pr-created
spec:
  suspend: {}
EOF

# Trigger event through Argo Events
kubectl apply -f - <<EOF
apiVersion: v1
kind: Event
metadata:
  name: test-resume-event
  namespace: argo-events
type: Normal
reason: TestEvent
message: '{"action": "opened", "task_id": "16"}'
EOF

# Wait for processing
sleep 10



# Verify workflow was resumed
kubectl get workflow test-argo-events-integration -o jsonpath='{.status.phase}' | grep -v Suspended

# Check sensor logs for successful processing
kubectl logs -l sensor-name=test-resume-sensor -n argo-events | grep "resume.*success"






```

### IR-2: Multi-Agent Workflow Compatibility
**Requirement**: Resume operations work correctly with existing multi-agent workflows

**Test Cases**:


- [ ] Rex implementation suspend/resume cycle works


- [ ] Cleo quality suspend/resume cycle works


- [ ] Tess testing suspend/resume cycle works


- [ ] Task progression suspend/resume cycle works


- [ ] Resume parameters flow correctly to next workflow steps

**Verification**:



```bash
# Test complete multi-agent workflow with resume operations
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: multi-agent-resume-test
  labels:
    workflow-type: play-orchestration
    task-id: "17"
    current-stage: waiting-pr-created
spec:
  entrypoint: main
  templates:
  - name: main
    dag:
      tasks:
      - name: implementation-work
        template: suspend-for-webhook
      - name: quality-work
        dependencies: [implementation-work]
        template: suspend-for-webhook
      - name: testing-work
        dependencies: [quality-work]
        template: suspend-for-webhook

  - name: suspend-for-webhook
    suspend: {}
EOF

# Test each resume point
# 1. Resume after implementation
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "17", "eventType": "pr-created"}' \
  -H "Content-Type: application/json"

sleep 5

# Update workflow stage for next test
kubectl label workflow multi-agent-resume-test current-stage=waiting-ready-for-qa --overwrite



# 2. Resume after quality
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "17", "eventType": "pr-labeled-ready"}' \
  -H "Content-Type: application/json"

sleep 5

# Update workflow stage for final test
kubectl label workflow multi-agent-resume-test current-stage=waiting-pr-approved --overwrite

# 3. Resume after testing
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "17", "eventType": "pr-approved"}' \
  -H "Content-Type: application/json"



# Verify workflow completed all stages
kubectl get workflow multi-agent-resume-test -o jsonpath='{.status.phase}' | grep "Succeeded"






```

## Monitoring and Observability Requirements

### MO-1: Resume Operation Logging
**Requirement**: All resume operations are comprehensively logged

**Test Cases**:


- [ ] Successful resume operations logged with details


- [ ] Failed resume operations logged with error details


- [ ] Retry attempts logged with attempt count and delay


- [ ] Circuit breaker state changes logged


- [ ] Event correlation results logged

**Verification**:



```bash
# Test successful resume logging
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "18", "eventType": "pr-created"}' \
  -H "Content-Type: application/json"

# Check logs for success entry
kubectl logs -l app=workflow-resume-service | grep "Resume successful.*workflow.*task.*18"

# Test failed resume logging
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "999", "eventType": "pr-created"}' \
  -H "Content-Type: application/json"

# Check logs for failure entry
kubectl logs -l app=workflow-resume-service | grep "Resume failed.*workflow.*task.*999"

# Test retry logging by causing temporary failure
kubectl scale deployment argo-server --replicas=0 -n argo

curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "19", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" &

sleep 10
kubectl scale deployment argo-server --replicas=1 -n argo
wait



# Check logs for retry attempts
kubectl logs -l app=workflow-resume-service | grep "Resume failed on attempt.*retrying in"
kubectl logs -l app=workflow-resume-service | grep "Resume attempt [0-9]*/[0-9]*"






```

### MO-2: Resume Operation Metrics
**Requirement**: Resume operations generate metrics for monitoring

**Test Cases**:


- [ ] Total resume attempts counter increments


- [ ] Successful resume counter increments


- [ ] Failed resume counter increments


- [ ] Resume latency histogram records durations


- [ ] Circuit breaker state gauge updates

**Verification**:



```bash
# Get baseline metrics
curl -s http://workflow-resume-service/metrics | grep "resume_total_attempts" | awk '{print $2}' > /tmp/baseline_total
curl -s http://workflow-resume-service/metrics | grep "resume_successful" | awk '{print $2}' > /tmp/baseline_success

# Perform resume operation
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "20", "eventType": "pr-created"}' \
  -H "Content-Type: application/json"



# Get updated metrics
curl -s http://workflow-resume-service/metrics | grep "resume_total_attempts" | awk '{print $2}' > /tmp/updated_total
curl -s http://workflow-resume-service/metrics | grep "resume_successful" | awk '{print $2}' > /tmp/updated_success

# Verify metrics incremented
baseline_total=$(cat /tmp/baseline_total)
updated_total=$(cat /tmp/updated_total)
test $updated_total -gt $baseline_total

baseline_success=$(cat /tmp/baseline_success)
updated_success=$(cat /tmp/updated_success)
test $updated_success -gt $baseline_success

# Check latency metrics
curl -s http://workflow-resume-service/metrics | grep "resume_latency_seconds"
curl -s http://workflow-resume-service/metrics | grep "resume_latency_seconds_bucket"






```

## Edge Case Requirements

### EC-1: Concurrent Resume Requests
**Requirement**: Multiple resume requests for the same workflow handled correctly

**Test Cases**:


- [ ] Concurrent resume requests don't cause race conditions


- [ ] Only one resume operation succeeds per workflow


- [ ] Duplicate resume attempts handled gracefully


- [ ] Workflow state consistency maintained


- [ ] Proper error messages for duplicate attempts

**Verification**:



```bash


# Create test workflow
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: concurrent-resume-test
  labels:
    task-id: "21"
    current-stage: waiting-pr-created
spec:
  suspend: {}
EOF

# Submit multiple concurrent resume requests
for i in {1..5}; do
  curl -X POST http://workflow-resume-service/resume \
    -d '{"taskId": "21", "eventType": "pr-created"}' \
    -H "Content-Type: application/json" > /tmp/concurrent_result_$i.json &
done

wait

# Count successful operations (should be 1)
success_count=0
for i in {1..5}; do
  if jq -r '.success' /tmp/concurrent_result_$i.json | grep -q "true"; then
    success_count=$((success_count + 1))
  fi
done

test $success_count -eq 1

# Verify workflow was resumed only once
kubectl get workflow concurrent-resume-test -o jsonpath='{.status.phase}' | grep -v Suspended






```

### EC-2: Late-Arriving Events
**Requirement**: Events arriving after workflow completion handled gracefully

**Test Cases**:


- [ ] Events for completed workflows are ignored


- [ ] Events for non-existent workflows handled appropriately


- [ ] Late events don't cause errors or resource leaks


- [ ] Proper logging for late-arriving events


- [ ] Event correlation prevents incorrect resume attempts

**Verification**:



```bash
# Create and complete a workflow
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: completed-workflow-test
  labels:
    task-id: "22"
status:
  phase: Succeeded
EOF



# Attempt to resume completed workflow
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "22", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" > /tmp/late_event_result.json

# Verify operation was ignored gracefully
jq -r '.success' /tmp/late_event_result.json | grep "false"
jq -r '.validationErrors[]' /tmp/late_event_result.json | grep -i "completed\|late"

# Check logs for appropriate handling
kubectl logs -l app=workflow-resume-service | grep "Event.*arrived.*after.*completion\|workflow.*already.*completed"

# Attempt resume for non-existent workflow
curl -X POST http://workflow-resume-service/resume \
  -d '{"taskId": "999", "eventType": "pr-created"}' \
  -H "Content-Type: application/json" > /tmp/nonexistent_result.json

# Verify handled gracefully
jq -r '.success' /tmp/nonexistent_result.json | grep "false"
jq -r '.validationErrors[]' /tmp/nonexistent_result.json | grep -i "not found\|missing"






```

## Final Validation Checklist

Before considering Task 14 complete:



- [ ] All functional requirements (FR-1 through FR-4) pass


- [ ] All error handling requirements (EH-1 through EH-3) pass


- [ ] All performance requirements (PR-1 through PR-2) pass


- [ ] All integration requirements (IR-1 through IR-2) pass


- [ ] All monitoring requirements (MO-1 through MO-2) pass


- [ ] All edge case requirements (EC-1 through EC-2) pass


- [ ] End-to-end multi-agent workflow with resume operations works


- [ ] Load testing demonstrates acceptable performance


- [ ] Circuit breaker protects against failures


- [ ] Retry logic handles transient failures correctly


- [ ] Code review completed and approved


- [ ] Documentation updated and reviewed


- [ ] Changes tested in isolated environment


- [ ] Ready for production deployment



## Success Metrics



1. **99%+ resume success rate** - For valid resume requests


2. **<10 second resume latency** - For successful operations


3. **<5 second failure latency** - For immediate validation failures


4. **Zero race conditions** - In concurrent resume scenarios


5. **100% event correlation accuracy** - For properly labeled events


6. **Circuit breaker effectiveness** - Protection during failures


7. **Comprehensive error handling** - All edge cases handled gracefully

## Post-Deployment Monitoring

After Task 14 completion, monitor these key indicators:



- **Resume success rate** - Percentage of successful resume operations


- **Resume latency distribution** - P50, P95, P99 latencies


- **Event correlation accuracy** - Correct workflow targeting rate


- **Retry frequency** - How often retries are needed


- **Circuit breaker activations** - Frequency and duration of breaker openings


- **Error rate by category** - Breakdown of failure types


- **Resource utilization** - CPU and memory usage under load

## Troubleshooting Scenarios

### Common Issues and Solutions



1. **High Resume Failure Rate**


   - Check Argo API server connectivity


   - Verify workflow label accuracy


   - Review event correlation logic


   - Monitor circuit breaker state



2. **High Resume Latency**


   - Check network latency to Argo API


   - Review retry configuration


   - Monitor resource usage


   - Verify connection pooling



3. **Event Correlation Failures**


   - Validate GitHub webhook payload format


   - Check task ID extraction logic


   - Verify workflow labeling consistency


   - Review stage transition logic

When all acceptance criteria are met, Task 14 successfully implements robust workflow resume operations that ensure reliable event-driven coordination in the multi-agent orchestration system.
