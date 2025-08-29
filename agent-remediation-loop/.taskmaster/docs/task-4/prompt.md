# Autonomous Agent Prompt: Implement State Management System

## Your Mission
You are tasked with building a comprehensive ConfigMap-based state management system for the Agent Remediation Loop. This system must track remediation iterations, store feedback history, and provide atomic operations with recovery capabilities, all integrated into the existing Rust controller.

## Context
The Agent Remediation Loop requires persistent state management to track:
- Iteration counts across remediation cycles
- Feedback history from QA reviews
- System metadata and operational status
- Recovery state after controller restarts

Your implementation will enhance the existing Rust controller with a new `remediation::state` module that uses Kubernetes ConfigMaps as the storage backend.

## Required Actions

### 1. Design ConfigMap Schema and Data Structure
Create comprehensive data structures in Rust:
- Design `RemediationState` struct with all required fields
- Implement `FeedbackEntry` with proper metadata
- Create enums for status, severity, and issue types
- Define naming convention: `task-{id}-state`
- Implement serde serialization for JSON storage

### 2. Build StateManager in Controller
Create `controller/src/remediation/state.rs` module:
- Implement `StateManager` struct with Kube client integration
- Create CRUD operations: get_state, create_or_update_state
- Use server-side apply for atomic operations
- Implement proper error handling with custom error types
- Add structured logging with tracing crate

### 3. Implement Atomic Iteration Counter
Build thread-safe iteration tracking:
- Create `increment_iteration` method with atomicity guarantees
- Implement MAX_ITERATIONS limit (10) with enforcement
- Add `get_current_iteration` for reading current state
- Use optimistic locking to handle concurrent modifications
- Track iteration history with timestamps

### 4. Create Feedback History System
Implement JSON-based feedback storage:
- Build `append_feedback` method with automatic serialization
- Create `get_feedback_history` for retrieving entries
- Implement compression for large feedback arrays
- Handle ConfigMap 1MB size limit with smart truncation
- Add pagination support for large histories

### 5. Build State Recovery System
Create robust recovery mechanisms:
- Implement `recover_state` method for controller restart scenarios
- Add state validation and consistency checking
- Create automatic repair for corrupted state data
- Integrate with controller's reconciliation loop
- Add health check endpoints for monitoring

### 6. Implement TTL Cleanup System
Build automated cleanup with Tokio:
- Create background task running every 6 hours
- Implement `cleanup_old_states` with configurable TTL (7 days)
- Use ConfigMap labels for efficient cleanup queries
- Add soft delete with grace period
- Integrate cleanup metrics and monitoring

## Technical Requirements

### Core Data Structures
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationState {
    pub task_id: String,
    pub iteration: u32,
    pub status: RemediationStatus,
    pub feedback_history: Vec<FeedbackEntry>,
    pub last_update: DateTime<Utc>,
    pub error_messages: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub version: String,
}
```

### StateManager Interface
```rust
impl StateManager {
    pub async fn increment_iteration(&self, task_id: &str) -> Result<u32, StateError>;
    pub async fn append_feedback(&self, task_id: &str, feedback: FeedbackEntry) -> Result<(), StateError>;
    pub async fn recover_state(&self, task_id: &str) -> Result<RemediationState, StateError>;
    pub async fn cleanup_old_states(&self) -> Result<(), StateError>;
}
```

### Integration Points
- Use existing Kube client from controller
- Integrate with controller's error handling patterns
- Follow controller's logging and metrics conventions
- Reuse controller's namespace and service account
- Compatible with existing RBAC configuration

## Implementation Checklist

### Core Implementation
- [ ] Create `controller/src/remediation/state.rs` module
- [ ] Define `RemediationState` and related data structures
- [ ] Implement `StateManager` with Kube client integration
- [ ] Create atomic `increment_iteration` method
- [ ] Build feedback history management
- [ ] Implement state recovery and validation
- [ ] Create TTL cleanup background task

### Integration
- [ ] Add state module to controller's mod.rs
- [ ] Integrate with existing controller reconciliation
- [ ] Add proper error handling and logging
- [ ] Implement health check integration
- [ ] Add metrics and monitoring support

### Testing
- [ ] Unit tests for all StateManager methods
- [ ] Integration tests with real Kubernetes cluster
- [ ] Test concurrent access scenarios
- [ ] Validate recovery after controller restart
- [ ] Test cleanup operations and TTL handling

## Expected Outputs

1. **State Module**: Complete `controller/src/remediation/state.rs` implementation
2. **Data Structures**: Rust structs with serde serialization
3. **CRUD Operations**: All state management functionality
4. **Atomic Operations**: Thread-safe iteration counter
5. **Recovery System**: Robust state recovery mechanisms
6. **Cleanup System**: Automated TTL-based cleanup
7. **Integration**: Full controller integration
8. **Tests**: Comprehensive test coverage

## Success Validation

Your implementation is successful when:
1. StateManager successfully creates and retrieves state from ConfigMaps
2. Iteration counter increments atomically without race conditions
3. Feedback history stores and retrieves data correctly
4. State recovery works after controller restart
5. Cleanup system removes old state data on schedule
6. All operations handle errors gracefully
7. Integration with controller doesn't break existing functionality
8. Performance meets requirements (sub-second operations)

## Technical Constraints

### Kubernetes Integration
- Use existing Kube client and service account
- Follow ConfigMap best practices and size limits
- Implement proper RBAC for state operations
- Handle network failures and API rate limiting

### Performance Requirements
- State operations complete within 1 second
- Memory usage scales reasonably with feedback history
- Cleanup operations don't impact controller performance
- Concurrent access doesn't cause data corruption

### Error Handling
- All operations return proper Result types
- Implement custom StateError enum
- Add structured logging for all operations
- Provide clear error messages for debugging

## Common Pitfalls to Avoid

- Don't ignore ConfigMap size limits (1MB)
- Avoid blocking the controller's main reconciliation loop
- Don't store sensitive data in ConfigMaps
- Ensure proper JSON schema evolution
- Handle partial failures in cleanup operations
- Test concurrent access thoroughly
- Validate all user input before storage

## Resources and References

- kube-rs documentation: https://docs.rs/kube/latest/kube/
- Kubernetes ConfigMap API reference
- Serde JSON serialization guide
- Tokio async programming patterns
- Controller reconciliation loop patterns

Begin by examining the existing controller structure to understand patterns and conventions. Create comprehensive data structures first, then implement the StateManager with proper error handling and logging. Test thoroughly with both unit and integration tests before integration with the main controller.