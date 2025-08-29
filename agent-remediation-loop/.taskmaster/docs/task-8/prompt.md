# Autonomous Agent Prompt: Implement Escalation and Termination Logic

## Your Mission
You are tasked with building a comprehensive escalation system for maximum iterations and implementing various termination conditions. This system must provide intelligent automated escalation when remediation limits are reached, handle timeout scenarios, detect success criteria, and manage manual intervention requests with graceful termination procedures.

## Context
The Agent Remediation Loop requires sophisticated termination logic to prevent infinite loops, escalate complex issues to humans, and automatically recognize successful completion. Your implementation will serve as both the safety net and success detector for the entire remediation process, ensuring efficient resource usage and timely human intervention when needed.

## Required Actions

### 1. Implement Iteration Limit Checking
Create robust iteration tracking and enforcement system:
- Build `EscalationManager` with configurable max iterations (default 10)
- Implement `check_iteration_limit()` method with proper validation
- Add warning system at 70% threshold (7 iterations)
- Create escalation triggers when maximum reached
- Track iteration history with timestamps and outcomes
- Integrate with Task 4's state management for consistency

### 2. Implement Timeout Detection System
Build comprehensive timeout monitoring with 4-hour limits:
- Create `check_timeout()` method monitoring elapsed time from start
- Implement warning system at 75% of timeout duration (3 hours)
- Add configurable timeout periods with environment overrides
- Handle timezone considerations using UTC timestamps
- Build escalation triggers for timeout exceeded scenarios
- Include grace period handling for long-running operations

### 3. Build Critical Error Detection
Create intelligent error classification and handling system:
- Design `CriticalError` enum with comprehensive error types
- Implement `handle_error()` method with automatic classification
- Add pattern matching for known critical error signatures
- Create escalation triggers for authentication, infrastructure, and data corruption errors
- Build retry logic for recoverable errors (rate limits, temporary failures)
- Preserve error context for debugging and analysis

### 4. Implement Manual Override Detection
Create system to detect and respond to human intervention:
- Build `check_manual_override()` method checking PR labels and comments
- Support multiple override types: skip-automation, manual-review-required, pause-remediation
- Add detection of intervention comments (@platform-team mentions)
- Create bypass request system for emergency situations
- Implement proper authorization checks for override actions
- Add comprehensive audit logging for all override events

### 5. Create Escalation Notification System
Build multi-channel notification system for escalation events:
- Implement `NotificationChannel` trait with GitHub, Slack, Email, PagerDuty support
- Create `post_escalation_comment()` for structured GitHub comments
- Build notification routing with appropriate team mentions (@platform-team, @cto)
- Add escalation reason formatting with context and history
- Implement retry logic for notification delivery failures
- Create escalation templates for different scenarios

### 6. Implement PR Comment Posting for Escalations
Build robust GitHub integration for escalation visibility:
- Create `GitHubClient` with proper authentication and error handling
- Build structured comment formatting for different escalation types
- Add markdown formatting for readability and action items
- Include escalation metadata: timestamps, iteration counts, duration
- Create team mention system for appropriate notification
- Handle GitHub API failures with retry logic and fallbacks

### 7. Implement Success Criteria Detection
Create intelligent success recognition system:
- Build `check_success_criteria()` method with multiple validation checks
- Check feedback resolution status from remediation history
- Validate PR approval status from GitHub reviews
- Monitor CI/CD status and required check completion
- Detect explicit success signals in PR comments (Tess approvals)
- Handle implicit success when all criteria met without explicit signal

### 8. Build Graceful Termination Procedures
Create comprehensive termination system with cleanup:
- Implement `terminate_remediation()` method with state preservation
- Build cleanup procedures for CodeRun resources and temporary data
- Create state archival system for post-mortem analysis
- Update PR labels based on termination reason
- Record comprehensive metrics for monitoring and analysis
- Send final notifications with termination summary

## Technical Requirements

### Core EscalationManager Structure
```rust
pub struct EscalationManager {
    max_iterations: u8,
    timeout_duration: Duration,
    warning_threshold: u8,
    notification_channels: Vec<NotificationChannel>,
    github_client: GitHubClient,
}

impl EscalationManager {
    pub async fn check_iteration_limit(&self, state: &RemediationState) -> Result<IterationStatus, EscalationError>;
    pub async fn check_timeout(&self, state: &RemediationState) -> Result<TimeoutStatus, EscalationError>;
    pub async fn handle_error(&self, error: &dyn std::error::Error, context: ErrorContext) -> Result<ErrorHandlingAction, EscalationError>;
    pub async fn check_success_criteria(&self, state: &RemediationState, pr_number: u32) -> Result<SuccessStatus, EscalationError>;
    pub async fn terminate_remediation(&self, state: &mut RemediationState, reason: TerminationReason, pr_number: u32) -> Result<TerminationResult, EscalationError>;
}
```

### Error Classification System
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CriticalErrorType {
    SystemFailure,
    AuthenticationError,
    RateLimitExceeded,
    RepositoryAccessDenied,
    InfrastructureFailure,
    DataCorruption,
    InvalidConfiguration,
    ExternalServiceFailure,
}

#[derive(Debug, Clone)]
pub enum ErrorHandlingAction {
    Continue,
    Retry,
    Escalate,
}
```

### Success Detection Framework
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SuccessStatus {
    InProgress,
    PendingFeedback(usize),
    PendingApproval,
    PendingCI(Vec<String>),
    Success(Vec<SuccessSignal>),
    ImplicitSuccess,
}
```

### Integration Points
- Task 4's StateManager for iteration tracking and state updates
- Task 7's label orchestration for final label management
- GitHub API for PR operations, comments, and status checks
- Existing notification infrastructure for alerts and escalations
- Monitoring systems for metrics collection and alerting

## Implementation Checklist

### Core Development
- [ ] Build EscalationManager with configurable limits and timeouts
- [ ] Implement iteration limit checking with warning thresholds
- [ ] Create timeout detection with grace periods and warnings
- [ ] Build critical error classification with pattern matching
- [ ] Implement manual override detection with multiple override types
- [ ] Create multi-channel notification system with retry logic

### GitHub Integration
- [ ] Build GitHubClient with proper authentication and error handling
- [ ] Implement escalation comment posting with structured formatting
- [ ] Add PR label and status checking for success criteria
- [ ] Create team mention system for appropriate notifications
- [ ] Handle GitHub API rate limits and failures gracefully

### Termination System
- [ ] Implement graceful termination with comprehensive cleanup
- [ ] Build state archival system for post-mortem analysis
- [ ] Create metrics collection for monitoring and alerting
- [ ] Add final notification system with termination summaries
- [ ] Ensure idempotent termination operations

### Testing and Quality
- [ ] Build comprehensive unit tests for all escalation scenarios
- [ ] Create integration tests with mock GitHub API
- [ ] Add stress tests for concurrent escalation handling
- [ ] Test all termination paths including failure scenarios
- [ ] Validate error handling and recovery mechanisms

## Expected Outputs

1. **EscalationManager**: Complete system with all escalation triggers
2. **Error Classification**: Intelligent error handling with proper routing
3. **Success Detection**: Automated recognition of completion criteria
4. **Notification System**: Multi-channel alerting with proper formatting
5. **GitHub Integration**: Robust API client with escalation comments
6. **Termination System**: Graceful cleanup with state preservation
7. **Test Suite**: Comprehensive coverage of all escalation scenarios
8. **Monitoring Integration**: Metrics and alerting for operational visibility

## Success Validation

Your implementation is successful when:
1. Iteration limits enforced correctly with escalation at 10 cycles
2. Timeout detection prevents runaway processes after 4 hours
3. Critical errors classified and escalated appropriately
4. Manual overrides detected and respected consistently
5. Success criteria recognized automatically for completed tasks
6. Escalation notifications reach appropriate teams reliably
7. Termination procedures complete with proper cleanup
8. All test scenarios pass including edge cases and failures
9. Integration with existing systems maintains functionality
10. Monitoring provides comprehensive visibility into escalations

## Technical Constraints

### Performance Requirements
- Escalation checks complete within 5 seconds
- Timeout detection runs efficiently without blocking operations  
- Error classification handles high-frequency error scenarios
- Success criteria detection scales with feedback volume
- Termination procedures complete within 30 seconds

### Reliability Requirements
- System handles GitHub API failures gracefully
- Escalation notifications have backup delivery mechanisms
- State preservation maintains consistency across failures
- Error handling prevents escalation system crashes
- Recovery mechanisms handle partial escalation failures

### Integration Constraints
- Must integrate seamlessly with Task 4's state management
- Notification channels respect existing infrastructure
- GitHub API usage follows rate limiting and authentication patterns
- Monitoring integration uses existing metrics collection
- Termination coordinates with other system components

## Common Pitfalls to Avoid

- Don't ignore GitHub API rate limits during escalation posting
- Avoid blocking operations during escalation checks
- Don't assume network connectivity for notification delivery
- Ensure proper error handling in escalation notification chains
- Test timeout scenarios thoroughly - timing is critical
- Validate iteration counting accuracy across system restarts
- Handle concurrent escalation attempts gracefully
- Don't leak sensitive information in escalation comments

## Resources and References

- GitHub REST API documentation for comments and status checks
- Rust error handling best practices with thiserror crate
- Notification system integration patterns
- Distributed systems timeout and retry patterns
- State machine implementation for escalation flows

## Support and Troubleshooting

If you encounter issues:
1. Validate GitHub API authentication and permissions
2. Check state management integration and data consistency
3. Test notification delivery with various failure scenarios
4. Verify timeout calculations and timezone handling
5. Monitor error classification accuracy with real errors
6. Test escalation comment formatting and team mentions
7. Validate termination cleanup with realistic resource sets

Begin by understanding the complete escalation flow and designing the state machine for all possible termination scenarios. Focus on reliability and comprehensive error handling - escalation failures can leave the system in unclear states. Test thoroughly with realistic timing and concurrency patterns.