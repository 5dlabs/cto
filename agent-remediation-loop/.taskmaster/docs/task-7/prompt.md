# Autonomous Agent Prompt: Build Label-Based Workflow Orchestration

## Your Mission
You are tasked with implementing a comprehensive PR label management system for tracking remediation state and iteration counts. This system will provide workflow orchestration through GitHub labels, enabling automated state transitions, human oversight, and clear progress visibility.

## Context
The Agent Remediation Loop needs a sophisticated state tracking and control mechanism. GitHub PR labels offer the perfect interface for both automated systems and human operators. Your implementation will create a label-based state machine that orchestrates the entire remediation workflow while providing override capabilities for human intervention.

## Required Actions

### 1. Design and Document Label Schema
Create a comprehensive label schema with clear naming conventions:
- **Task Association**: `task-{id}` labels for permanent task identification
- **Iteration Tracking**: `iteration-{n}` labels for remediation cycle counting
- **Status Labels**: Workflow state indicators (needs-remediation, remediation-in-progress, ready-for-qa, approved)
- **Override Controls**: Human intervention labels (skip-automation, manual-review-required, pause-remediation)
- Document complete lifecycle management and usage guidelines

### 2. Implement GitHub API Label Integration
Build robust GitHub API integration with comprehensive error handling:
- Create `GitHubLabelClient` with proper authentication and rate limiting
- Implement retry logic with exponential backoff for API failures
- Use GitHub's ETag headers for optimistic concurrency control
- Add batch operations support for efficient API usage
- Handle all GitHub API edge cases and error conditions

### 3. Build State Transition Logic Engine
Create the core orchestration engine with state machine implementation:
- Design `LabelOrchestrator` class with complete state transition logic
- Implement state machine with all valid workflow transitions
- Add transition condition checking (iteration limits, prerequisites)
- Build action execution system for label modifications
- Integrate with Task 4's state management for consistency

### 4. Add Skip-Automation Override Detection
Implement comprehensive override detection and handling:
- Create `OverrideDetector` with support for multiple override types
- Add automatic detection of override labels on all operations
- Implement notification system for override events
- Build bypass request system for emergency situations
- Add audit logging for all override activities

### 5. Implement Label Cleanup System
Build automated cleanup for maintaining label hygiene:
- Create `LabelCleanupManager` for completed and abandoned tasks
- Implement TTL-based cleanup with configurable retention periods
- Add selective cleanup preserving important historical labels
- Build scheduled cleanup jobs with comprehensive reporting
- Handle cleanup conflicts and edge cases gracefully

### 6. Handle Concurrent Label Updates
Implement sophisticated concurrency control mechanisms:
- Create `ConcurrentLabelManager` with per-PR locking
- Build queue-based operation batching for efficiency
- Implement atomic label operations with conflict resolution
- Add distributed locking for multi-instance deployments
- Handle race conditions and concurrent modification scenarios

## Technical Requirements

### Label Schema Structure
```typescript
interface TaskLabel {
  pattern: 'task-{id}';
  lifecycle: 'permanent';
  purpose: 'task association';
}

interface IterationLabel {
  pattern: 'iteration-{n}';
  lifecycle: 'updated-per-cycle';
  purpose: 'iteration tracking';
}

interface StatusLabel {
  values: ['needs-remediation', 'remediation-in-progress', 'ready-for-qa', 'approved', 'failed-remediation'];
  lifecycle: 'state-based';
  purpose: 'workflow status';
}
```

### State Machine Implementation
```typescript
class LabelOrchestrator {
  async transitionState(prNumber: number, taskId: string, trigger: string, context?: any): Promise<void>;
  private determineCurrentState(labels: string[]): WorkflowState;
  private findTransition(currentState: WorkflowState, trigger: string): StateTransition | null;
  private executeTransition(prNumber: number, taskId: string, transition: StateTransition): Promise<void>;
}
```

### Concurrency Control
```typescript
class ConcurrentLabelManager {
  async withLock<T>(prNumber: number, operation: () => Promise<T>): Promise<T>;
  async updateLabelsWithRetry(prNumber: number, operations: LabelOperation[]): Promise<void>;
  async batchOperations(operations: BatchOperation[]): Promise<BatchResult>;
}
```

### Integration Points
- GitHub API with proper authentication and rate limiting
- Task 4's StateManager for iteration tracking and state consistency
- Existing sensors for automated trigger detection
- Notification systems for override and status alerts
- Monitoring and alerting infrastructure

## Implementation Checklist

### Core Development
- [ ] Design complete label schema with naming conventions
- [ ] Implement GitHub API client with retry logic and rate limiting
- [ ] Build state machine with all workflow transitions
- [ ] Create override detection with multiple override types
- [ ] Implement cleanup system with TTL and selective retention
- [ ] Build concurrency control with atomic operations

### Integration
- [ ] Connect with Task 4's state management system
- [ ] Integrate with existing webhook sensors
- [ ] Add notification system integration
- [ ] Build monitoring and metrics collection
- [ ] Create operational dashboards and alerting

### Quality Assurance
- [ ] Build comprehensive unit tests for all components
- [ ] Create integration tests with real GitHub API
- [ ] Add concurrency stress tests with multiple operations
- [ ] Test all edge cases and error conditions
- [ ] Validate performance requirements under load

## Expected Outputs

1. **Label Schema Documentation**: Complete schema with naming conventions and lifecycle rules
2. **GitHub API Integration**: Robust client with comprehensive error handling
3. **State Machine Engine**: Complete orchestration logic with transition validation
4. **Override System**: Detection, handling, and notification capabilities
5. **Cleanup Manager**: Automated cleanup with configurable policies
6. **Concurrency Controller**: Atomic operations with conflict resolution
7. **Integration Layer**: Connections with existing systems and workflows
8. **Test Suite**: Comprehensive testing covering all functionality

## Success Validation

Your implementation is successful when:
1. Label schema provides clear workflow state visibility
2. State transitions work reliably for all workflow scenarios
3. Concurrent operations complete without conflicts or data corruption
4. Override system provides effective human control mechanisms
5. Cleanup system maintains label hygiene without affecting active workflows
6. Integration with existing systems functions seamlessly
7. Performance requirements met under realistic load
8. All test scenarios pass including stress and edge cases
9. Monitoring provides comprehensive visibility into operations
10. Documentation enables easy maintenance and troubleshooting

## Technical Constraints

### GitHub API Limitations
- Respect rate limits with proper backoff strategies
- Handle API failures gracefully with retry logic
- Use conditional requests to minimize API usage
- Implement proper authentication and token management

### Concurrency Requirements
- Support multiple concurrent label operations per PR
- Prevent race conditions in high-load scenarios
- Maintain data consistency across all operations
- Handle distributed deployment scenarios

### Performance Requirements
- Label operations complete within 5 seconds
- Support 100+ concurrent PRs with active label management
- Minimize GitHub API calls while maintaining functionality
- Cleanup operations complete within reasonable time bounds

### Integration Constraints
- Must not interfere with existing label usage
- Backward compatibility with current workflow patterns
- Integration with existing monitoring and alerting
- Maintain existing GitHub permissions and access patterns

## Common Pitfalls to Avoid

- Don't ignore GitHub API rate limits - implement proper throttling
- Avoid race conditions in concurrent label updates
- Don't assume label operations are atomic - implement proper conflict resolution
- Ensure cleanup doesn't remove labels from active workflows
- Test override detection thoroughly - false positives break workflows
- Validate state machine transitions - invalid transitions cause confusion
- Handle API failures gracefully - network issues shouldn't break workflow
- Don't leak sensitive information in labels or logs

## Resources and References

- GitHub REST API documentation: https://docs.github.com/en/rest/issues/labels
- GitHub GraphQL API for efficient label operations
- Octokit.js library for GitHub API integration
- State machine pattern implementation guides
- Concurrency control patterns for distributed systems

## Support and Troubleshooting

If you encounter issues:
1. Check GitHub API rate limit headers and usage
2. Verify authentication and repository permissions
3. Test state machine transitions in isolation
4. Monitor label operations for conflicts and retries
5. Validate integration with existing workflow components
6. Check override detection logic with various label combinations
7. Test cleanup operations with realistic data sets

Begin by designing the complete label schema and state machine. Understand the full workflow before implementing any components. Focus on robustness and error handling - GitHub API operations can fail in many ways. Test concurrency scenarios thoroughly as race conditions are difficult to debug in production.