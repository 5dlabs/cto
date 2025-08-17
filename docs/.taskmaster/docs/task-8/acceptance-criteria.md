# Acceptance Criteria: Rex Remediation Sensor

## Core Event Detection Requirements

### ✅ EventSource Configuration
- [ ] **Rex Push EventSource**: EventSource created to detect GitHub push events from 5DLabs-Rex[bot]
- [ ] **Event Filtering**: Events filtered to only task branches (refs/heads/task-*) 
- [ ] **Sender Validation**: Only events from 5DLabs-Rex[bot] processed
- [ ] **Repository Filtering**: Events limited to target repository (5dlabs/cto)
- [ ] **Webhook Setup**: Webhook endpoint configured and accessible

### ✅ Task ID Extraction
- [ ] **Branch Name Parsing**: Task ID correctly extracted from branch names (task-3-*, task-15-*)
- [ ] **Regex Pattern**: Regex pattern reliably matches task branch naming convention
- [ ] **Validation**: Invalid task IDs rejected with clear error messages
- [ ] **Edge Cases**: Handles malformed branch names gracefully
- [ ] **Correlation**: Task ID used for downstream agent targeting

## Agent Cancellation Requirements

### ✅ CodeRun Deletion Logic
- [ ] **Label Selector Targeting**: Uses precise label selectors to target only affected agents
- [ ] **Cleo Agent Cancellation**: All Cleo CodeRun CRDs for task cancelled correctly
- [ ] **Tess Agent Cancellation**: All Tess CodeRun CRDs for task cancelled correctly 
- [ ] **Rex Agent Preservation**: Rex agents not cancelled by remediation process
- [ ] **Selective Targeting**: Only agents with matching task-id cancelled

### ✅ Cancellation Safety
- [ ] **Existence Verification**: Checks if agents exist before attempting deletion
- [ ] **Graceful Handling**: Non-existent agents handled without errors
- [ ] **Completion Verification**: Verifies all targeted agents cancelled successfully
- [ ] **Cleanup Validation**: No orphaned resources left after cancellation
- [ ] **Concurrent Safety**: Multiple cancellation requests don't cause conflicts

## GitHub Integration Requirements

### ✅ Label Management
- [ ] **PR Discovery**: Correctly finds PRs associated with task ID
- [ ] **Ready-for-QA Removal**: Removes ready-for-qa label when present
- [ ] **Label Validation**: Checks label existence before removal attempt
- [ ] **API Authentication**: GitHub API calls properly authenticated
- [ ] **Error Handling**: GitHub API failures handled gracefully

### ✅ PR Correlation  
- [ ] **Task Label Matching**: Finds PRs using task-{id} label correlation
- [ ] **Multiple PR Handling**: Handles scenarios with multiple PRs for same task
- [ ] **Missing PR Handling**: Gracefully handles when no PR found for task
- [ ] **PR State Validation**: Validates PR is in correct state for label removal
- [ ] **Repository Targeting**: Operations limited to correct repository

## Workflow State Management Requirements

### ✅ Stage Reset Logic
- [ ] **Main Workflow Discovery**: Correctly identifies main play workflow for task
- [ ] **Stage Update**: Updates workflow stage to waiting-pr-created
- [ ] **Atomic Operations**: Stage updates performed atomically
- [ ] **State Verification**: Verifies stage update completed successfully
- [ ] **Multiple Workflow Handling**: Handles scenarios with multiple workflows per task

### ✅ Workflow Resumption
- [ ] **Resume Operation**: Successfully resumes suspended main workflow
- [ ] **Resume Verification**: Confirms workflow resumed and progressing
- [ ] **Stage Coordination**: Workflow resumes at correct stage (quality-work)
- [ ] **Timing Coordination**: Resume happens after all cleanup completed
- [ ] **Error Recovery**: Failed resume attempts handled with retries

## Sensor Implementation Requirements

### ✅ Remediation Workflow Structure
- [ ] **DAG Organization**: Remediation steps organized in proper dependency order
- [ ] **Step Dependencies**: Each step waits for previous step completion
- [ ] **Parameter Passing**: Task ID and context passed between workflow steps
- [ ] **Template Organization**: Reusable templates for common operations
- [ ] **Error Propagation**: Failures in any step cause workflow failure

### ✅ Validation and Safety
- [ ] **Event Validation**: All incoming events validated before processing
- [ ] **Sender Verification**: Confirms event actually from 5DLabs-Rex[bot]
- [ ] **Task ID Validation**: Ensures task ID is valid numeric value
- [ ] **Branch Validation**: Validates branch matches expected task pattern
- [ ] **Repository Validation**: Confirms event from correct repository

## Idempotency and Reliability Requirements

### ✅ Duplicate Event Handling
- [ ] **Idempotent Operations**: All remediation steps safe to retry and duplicate
- [ ] **Event Deduplication**: Duplicate GitHub webhook events handled gracefully
- [ ] **State Consistency**: Repeated operations don't cause inconsistent state
- [ ] **Concurrent Execution**: Multiple remediation workflows don't interfere
- [ ] **Race Condition Prevention**: Proper locking/sequencing for critical operations

### ✅ Error Recovery
- [ ] **Partial Failure Recovery**: System recovers from partial remediation failures
- [ ] **Retry Logic**: Failed operations retry with appropriate backoff
- [ ] **Manual Recovery**: Failed remediation can be manually completed
- [ ] **State Repair**: Inconsistent state can be detected and corrected
- [ ] **Rollback Capability**: Critical failures can be rolled back safely

## Performance Requirements

### ✅ Response Times  
- [ ] **Event Processing**: Rex push events processed within 30 seconds
- [ ] **Agent Cancellation**: Agent cancellation completes within 60 seconds
- [ ] **Workflow Resumption**: Main workflow resumed within 2 minutes of push
- [ ] **End-to-End Latency**: Complete remediation process within 5 minutes
- [ ] **GitHub API Calls**: Label operations complete within 10 seconds

### ✅ Scalability
- [ ] **Concurrent Tasks**: Handles remediation for multiple tasks simultaneously
- [ ] **High Frequency Events**: Processes rapid sequential pushes without issues
- [ ] **Resource Usage**: Remediation workflow uses minimal cluster resources
- [ ] **Event Volume**: Handles high-volume GitHub webhook events efficiently
- [ ] **Agent Scale**: Cancellation scales to large numbers of running agents

## Testing Requirements

### ✅ Event Simulation Testing
- [ ] **Push Event Simulation**: Test with simulated Rex push webhook events
- [ ] **Branch Pattern Testing**: Test task ID extraction with various branch patterns
- [ ] **Sender Validation Testing**: Test filtering with different sender values
- [ ] **Malformed Event Testing**: Test handling of malformed webhook payloads
- [ ] **Repository Filtering Testing**: Test events from different repositories

### ✅ Agent Cancellation Testing
- [ ] **Running Agent Tests**: Test cancellation with actual running CodeRun CRDs
- [ ] **Label Selector Testing**: Verify only targeted agents cancelled
- [ ] **Multiple Agent Testing**: Test with multiple Cleo/Tess agents per task
- [ ] **Concurrent Cancellation**: Test cancellation during agent execution
- [ ] **Resource Cleanup Testing**: Verify all related resources cleaned up

### ✅ Integration Testing
- [ ] **End-to-End Flow**: Complete Rex push → cancellation → resume cycle
- [ ] **GitHub API Integration**: Test label removal with real GitHub PRs
- [ ] **Workflow Coordination**: Test interaction with main play workflows
- [ ] **Multi-Task Scenarios**: Test with multiple concurrent tasks
- [ ] **Error Scenario Testing**: Test various failure modes and recovery

## Security Requirements

### ✅ Access Control
- [ ] **RBAC Configuration**: Sensor has minimal required permissions for operations
- [ ] **Service Account Security**: Dedicated service accounts with restricted permissions
- [ ] **GitHub Token Security**: GitHub tokens properly secured and rotated
- [ ] **Webhook Authentication**: GitHub webhook events properly authenticated
- [ ] **Cross-Namespace Security**: Operations don't affect resources in other namespaces

### ✅ Validation and Filtering
- [ ] **Input Validation**: All webhook payloads validated before processing
- [ ] **Malicious Event Protection**: Malformed or malicious events can't cause damage
- [ ] **Authorization Checks**: Only authorized senders can trigger remediation
- [ ] **Audit Trail**: All remediation operations logged for security auditing
- [ ] **Rate Limiting**: Protection against webhook flooding attacks

## Monitoring and Observability Requirements

### ✅ Logging and Metrics
- [ ] **Event Processing Logs**: All Rex push events logged with details
- [ ] **Remediation Step Logs**: Each remediation step logged with outcomes
- [ ] **Error Logging**: All failures logged with sufficient detail for debugging
- [ ] **Performance Metrics**: Remediation timing and success rates tracked
- [ ] **Agent Metrics**: Agent cancellation counts and timing tracked

### ✅ Alerting and Monitoring
- [ ] **Failure Alerts**: Alerts configured for remediation failures
- [ ] **Performance Alerts**: Alerts for remediation taking longer than expected
- [ ] **State Consistency Alerts**: Alerts for inconsistent workflow states
- [ ] **Health Monitoring**: Sensor health monitored and reported
- [ ] **Dashboard Integration**: Remediation status visible in monitoring dashboards

## Documentation Requirements

### ✅ Technical Documentation
- [ ] **Implementation Guide**: Complete remediation sensor implementation documented
- [ ] **Architecture Overview**: Rex remediation system architecture documented
- [ ] **Configuration Reference**: All sensor and workflow configuration options documented
- [ ] **API Documentation**: GitHub integration and webhook handling documented
- [ ] **Troubleshooting Guide**: Common issues and resolution steps documented

### ✅ Operational Documentation
- [ ] **Deployment Guide**: How to deploy and configure rex remediation sensor
- [ ] **Monitoring Guide**: How to monitor remediation system health and performance
- [ ] **Maintenance Procedures**: How to maintain and update remediation components
- [ ] **Emergency Procedures**: How to handle remediation failures and incidents
- [ ] **Security Procedures**: Security best practices and incident response documented

## Validation Checklist

Before marking this task complete, verify:

1. **Event Detection**: Rex push events detected and processed correctly
2. **Agent Cancellation**: All targeted downstream agents cancelled reliably
3. **State Management**: Workflow stages reset and workflows resumed correctly
4. **Safety**: No unintended cancellations or false positive detections
5. **Performance**: System meets all response time and scalability requirements
6. **Testing**: All unit, integration, and load tests pass consistently
7. **Security**: All security requirements met and validated
8. **Documentation**: All technical and operational documentation complete

## Success Metrics

- **Event Detection Accuracy**: >99% of Rex push events detected and processed
- **Cancellation Success Rate**: >99% of targeted agents cancelled successfully
- **False Positive Rate**: <0.1% of non-Rex events trigger remediation
- **End-to-End Latency**: <5 minutes average for complete remediation process
- **System Availability**: Rex remediation system maintains >99.9% availability
- **Error Recovery Rate**: >95% of failures recover automatically or with minimal intervention