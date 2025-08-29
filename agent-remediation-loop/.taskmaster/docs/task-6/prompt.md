# Autonomous Agent Prompt: Implement Agent Cancellation System

## Your Mission
You are tasked with enhancing the existing agent cancellation system to handle concurrent operations and improve race condition handling when Rex pushes remediation fixes. This system must provide robust cancellation capabilities with distributed locking, state awareness, and advanced recovery mechanisms.

## Context
The current implementation-agent-remediation sensor provides basic cancellation functionality:
- Detects Rex push events
- Deletes CodeRun resources for Cleo/Tess agents
- Manages PR labels (removes 'ready-for-qa', adds 'remediation-in-progress')

Your enhancement will add advanced concurrency control, distributed locking, state integration, and sophisticated recovery mechanisms to handle high-load scenarios and prevent race conditions.

## Required Actions

### 1. Enhance Existing Rex Push Event Sensor
Extend the current sensor in `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`:
- Add advanced filtering for concurrent push events
- Implement state validation before triggering cancellation
- Add enhanced error handling for webhook processing
- Generate correlation IDs for tracking related operations
- Configure resource limits and improved reliability features

### 2. Implement Distributed Locking for Concurrent Operations
Build lease-based distributed locking using Kubernetes coordination API:
- Create `DistributedLock` struct using `k8s.io/client-go` coordination/v1
- Implement lease acquisition with automatic renewal and timeout
- Add exponential backoff for lock contention scenarios
- Create namespace isolation per task-id to prevent conflicts
- Implement deadlock detection using lease annotations
- Ensure locks are released on process restart or crash

### 3. Enhance CodeRun Deletion with State Awareness
Improve existing deletion logic with state integration:
- Query state ConfigMap before deletion to check current status
- Verify agents haven't already completed successfully
- Implement graceful termination with SIGTERM before SIGKILL
- Add deletion confirmation via status polling
- Update remediation state after successful deletion
- Integrate with Task 4's state management system for consistency

### 4. Implement Atomic Label Transitions
Build GitHub label management with concurrency control:
- Use GitHub's ETag headers for optimistic concurrency control
- Implement compare-and-swap operations for label updates
- Add retry logic with exponential backoff for conflicts
- Batch label operations to reduce API calls and improve atomicity
- Add validation ensuring correct labels exist before/after operations
- Handle concurrent modifications from multiple sources

### 5. Build Concurrent Cancellation Coordinator
Create coordination layer for managing multiple simultaneous cancellations:
- Track all active cancellations in shared ConfigMap coordination store
- Prevent duplicate cancellation requests for same task
- Manage cancellation priority queue for ordered processing
- Implement circuit breaker pattern for cascading failure prevention
- Provide cancellation status API for monitoring and debugging
- Add comprehensive metrics for cancellation success/failure rates

### 6. Implement Advanced Recovery System
Build sophisticated recovery mechanism for partial failures:
- Detect partial failures through state inconsistency monitoring
- Implement automatic rollback for failed multi-step operations
- Add manual intervention triggers for unrecoverable states
- Create reconciliation loop running every 30 seconds for consistency
- Implement cleanup for orphaned locks and resources
- Include comprehensive observability and alerting

### 7. Create Stress Tests for Concurrent Cancellations
Develop comprehensive test suite for high-concurrency scenarios:
- Simulate 20+ concurrent Rex pushes with realistic timing
- Test with random network delays and API failures
- Verify no lost cancellations or orphaned agents under load
- Test lock contention and timeout scenarios thoroughly
- Validate state consistency after chaos testing conditions
- Measure cancellation latency under various load conditions

### 8. Integrate with State Management System
Connect enhanced cancellation with Task 4's state management:
- Update remediation state during cancellation operations
- Record cancellation history in state ConfigMap for audit trail
- Use state to determine if re-cancellation is needed
- Implement state-based recovery decisions and logic
- Add state validation before and after cancellation operations
- Ensure state consistency across distributed operations

## Technical Requirements

### Distributed Locking Implementation
```go
type DistributedLock struct {
    client      kubernetes.Interface
    namespace   string
    lockName    string
    holderName  string
    leaseDuration time.Duration
}

func (dl *DistributedLock) TryAcquire(ctx context.Context) (*Lease, error)
func (dl *DistributedLock) isLeaseExpired(lease *coordinationv1.Lease) bool
```

### State-Aware Cancellation
```go
type StateAwareCancellation struct {
    client      client.Client
    stateManager StateManager
    lockManager  *DistributedLock
}

func (sac *StateAwareCancellation) CancelAgentsWithStateCheck(ctx context.Context, taskID string, prNumber int) error
```

### Atomic Label Management
```go
type AtomicLabelManager struct {
    client *github.Client
    owner  string
    repo   string
}

func (alm *AtomicLabelManager) AtomicLabelTransition(ctx context.Context, prNumber int, transitions []LabelTransition) error
```

### Integration Points
- Existing sensor: `implementation-agent-remediation`
- State management: Integration with Task 4's StateManager
- GitHub API: Existing authentication and webhook infrastructure
- Kubernetes API: Enhanced RBAC and resource management
- Monitoring: Metrics and alerting integration

## Implementation Checklist

### Core Enhancement
- [ ] Enhance existing sensor configuration with advanced filtering
- [ ] Implement distributed locking using Kubernetes coordination API
- [ ] Create state-aware cancellation logic with consistency checks
- [ ] Build atomic label transition system with conflict resolution
- [ ] Develop concurrent cancellation coordinator with queue management
- [ ] Implement advanced recovery system with reconciliation loop

### Integration and Testing
- [ ] Integrate with Task 4's state management system
- [ ] Create comprehensive stress test suite for concurrent scenarios
- [ ] Build monitoring and alerting for cancellation operations
- [ ] Add performance profiling and optimization
- [ ] Implement comprehensive error handling and logging
- [ ] Create operational runbooks and troubleshooting guides

### Quality Assurance
- [ ] Validate no race conditions under high concurrency
- [ ] Test recovery from various failure scenarios
- [ ] Verify state consistency across all operations
- [ ] Ensure proper resource cleanup and no leaks
- [ ] Test integration with existing workflow components
- [ ] Validate performance requirements under load

## Expected Outputs

1. **Enhanced Sensor Configuration**: Updated YAML with advanced features
2. **Distributed Locking System**: Complete Kubernetes lease-based implementation
3. **State-Aware Cancellation**: Integration with state management
4. **Atomic Label Manager**: GitHub API integration with concurrency control
5. **Coordination System**: Queue-based cancellation coordination
6. **Recovery System**: Automated recovery and reconciliation
7. **Test Suite**: Comprehensive stress and integration tests
8. **Documentation**: Operational guides and troubleshooting

## Success Validation

Your implementation is successful when:
1. Enhanced sensor processes concurrent Rex pushes without conflicts
2. Distributed locks prevent race conditions in high-load scenarios
3. State-aware cancellation integrates seamlessly with Task 4
4. Atomic label transitions handle concurrent GitHub modifications
5. Coordination system manages multiple cancellations efficiently
6. Recovery system detects and repairs partial failures automatically
7. Stress tests validate performance under 20+ concurrent operations
8. No resource leaks or deadlocks under sustained load
9. Integration maintains existing functionality while adding enhancements
10. Monitoring provides comprehensive visibility into operations

## Technical Constraints

### Performance Requirements
- Cancellation operations complete within 30 seconds
- System handles 20+ concurrent cancellations
- Lock acquisition completes within 5 seconds
- Memory usage scales linearly with active cancellations
- No degradation of existing sensor performance

### Reliability Requirements
- Zero data loss during concurrent operations
- Automatic recovery from transient failures
- State consistency maintained across restarts
- Circuit breaker prevents cascading failures
- Comprehensive observability for debugging

### Integration Constraints
- Must not break existing functionality
- Backward compatibility with current sensors
- Use existing GitHub App authentication
- Respect Kubernetes RBAC policies
- Integration with existing monitoring stack

## Common Pitfalls to Avoid

- Don't ignore lock timeout scenarios - implement proper cleanup
- Avoid blocking the main sensor operation with long-running locks
- Don't assume GitHub API operations are atomic - implement proper retry
- Ensure proper resource cleanup to prevent memory leaks
- Test concurrent scenarios thoroughly - race conditions are subtle
- Don't ignore partial failure scenarios - implement proper recovery
- Validate state consistency - inconsistent state causes cascading issues

## Resources and References

- Kubernetes Coordination API: https://kubernetes.io/docs/reference/kubernetes-api/cluster-resources/lease-v1/
- GitHub API concurrency patterns and best practices
- Distributed systems patterns for concurrency control
- Circuit breaker pattern implementation guides
- Kubernetes client-go documentation and examples

## Support and Troubleshooting

If you encounter issues:
1. Check Kubernetes lease objects for lock contention
2. Validate GitHub API rate limits and quotas
3. Review sensor logs for processing errors
4. Monitor state consistency across operations
5. Check circuit breaker status and failure rates
6. Verify RBAC permissions for enhanced operations
7. Test with isolated scenarios before full integration

Begin by understanding the current sensor implementation and existing cancellation flow. Build upon this foundation with careful attention to concurrency safety and state consistency. Test thoroughly with realistic concurrent load before deployment.