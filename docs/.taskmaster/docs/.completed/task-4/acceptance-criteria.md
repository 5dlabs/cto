# Acceptance Criteria: Implement Conditional Agent-Specific PVC Naming

## Functional Requirements

### 1. Agent Name Extraction Logic
- [ ] `extract_agent_name()` function implemented in `controller/src/tasks/code/resources.rs`
- [ ] Correctly parses `5DLabs-Rex` to `rex`
- [ ] Correctly parses `5DLabs-Cleo[bot]` to `cleo`
- [ ] Correctly parses `5DLabs-Tess` to `tess`
- [ ] Correctly parses `5DLabs-Blaze` to `blaze`
- [ ] Handles case-insensitive input
- [ ] Returns appropriate errors for invalid GitHub App names
- [ ] Validates extracted names against Kubernetes naming constraints

### 2. Agent Classification Logic
- [ ] `is_implementation_agent()` function identifies Rex and Blaze as implementation agents
- [ ] `requires_isolated_workspace()` function returns false for implementation agents
- [ ] Implementation agents continue using `workspace-{service}` pattern
- [ ] Non-implementation agents use `workspace-{service}-{agent}` pattern
- [ ] Agent classification is extensible for future agent types

### 3. Conditional PVC Naming Implementation
- [ ] Rex agent continues using `workspace-cto` (shared workspace)
- [ ] Blaze agent continues using `workspace-cto` (shared workspace)
- [ ] Cleo agent gets `workspace-cto-cleo` (isolated workspace)
- [ ] Tess agent gets `workspace-cto-tess` (isolated workspace)
- [ ] Agent names properly validated for Kubernetes compliance
- [ ] PVC names truncated if necessary to meet 63-character limit

### 4. Controller Integration
- [ ] Reconciliation logic updated to use conditional PVC naming
- [ ] PVC creation integrated with existing controller workflow
- [ ] Pod creation uses correct PVC for volume mounts
- [ ] Error handling implemented for agent name extraction failures
- [ ] Status updates reflect PVC creation success/failure

### 5. Backward Compatibility
- [ ] **CRITICAL**: Implementation agents (Rex, Blaze) continue using existing `workspace-{service}` pattern
- [ ] No disruption to currently running CodeRun instances
- [ ] Existing PVCs remain accessible to implementation agents
- [ ] Graceful fallback when agent extraction fails

### 6. Workspace Strategy
- [ ] Implementation agents share workspace for collaborative development
- [ ] Non-implementation agents get isolated workspaces for independent operation
- [ ] Session continuity maintained within appropriate workspace type
- [ ] Knowledge accumulation occurs in correct workspace context

## Technical Requirements

### Code Implementation
- [ ] `extract_agent_name()` function uses regex for robust parsing
- [ ] `AgentClassifier` struct manages agent type classification
- [ ] Kubernetes naming validation prevents invalid PVC names
- [ ] Idempotent PVC creation using kube-rs API
- [ ] Proper error handling with descriptive error messages
- [ ] Comprehensive logging for troubleshooting

### Resource Management
- [ ] PVC creation follows Kubernetes best practices
- [ ] Proper labels applied to PVCs (shared vs isolated)
- [ ] Resource limits and requests configured appropriately
- [ ] PVC cleanup handled correctly on resource deletion

### Performance
- [ ] Agent name extraction completes in <1ms
- [ ] Agent classification completes in <1ms
- [ ] PVC creation doesn't significantly impact reconciliation time
- [ ] Controller memory usage remains stable with new logic
- [ ] No performance degradation for existing functionality

## Test Cases

### Test Case 1: Agent Name Extraction
**Objective**: Verify agent name extraction from various GitHub App formats

**Steps**:
1. Test `extract_agent_name("5DLabs-Rex")` returns `"rex"`
2. Test `extract_agent_name("5DLabs-Cleo[bot]")` returns `"cleo"`
3. Test `extract_agent_name("5DLabs-Tess")` returns `"tess"`
4. Test `extract_agent_name("5DLabs-Blaze")` returns `"blaze"`
5. Test invalid inputs return appropriate errors
6. Test case sensitivity handling

**Expected Result**: All agent names extracted correctly with proper error handling

### Test Case 2: Agent Classification
**Objective**: Verify correct classification of implementation vs non-implementation agents

**Steps**:
1. Test `is_implementation_agent("rex")` returns `true`
2. Test `is_implementation_agent("blaze")` returns `true`
3. Test `is_implementation_agent("cleo")` returns `false`
4. Test `is_implementation_agent("tess")` returns `false`
5. Test `requires_isolated_workspace()` returns opposite of `is_implementation_agent()`

**Expected Result**: Implementation agents correctly identified, non-implementation agents flagged for isolation

### Test Case 3: Conditional PVC Creation
**Objective**: Validate PVCs created with appropriate naming based on agent type

**Steps**:
1. Submit CodeRun with `github_app: "5DLabs-Rex"`
2. Verify PVC created with name `workspace-cto` (shared)
3. Submit CodeRun with `github_app: "5DLabs-Cleo"`
4. Verify PVC created with name `workspace-cto-cleo` (isolated)
5. Check PVC has correct labels and annotations
6. Confirm PVC meets Kubernetes naming constraints

**Expected Result**: PVCs created with correct naming pattern based on agent classification

### Test Case 4: Pod Volume Mount Integration
**Objective**: Ensure pods mount correct PVCs based on agent type

**Steps**:
1. Create CodeRun for Rex agent
2. Verify pod mounts `workspace-cto` PVC (shared)
3. Create CodeRun for Cleo agent
4. Verify pod mounts `workspace-cto-cleo` PVC (isolated)
5. Check workspace directory is accessible in pod
6. Validate session continuity with appropriate workspace

**Expected Result**: Pods correctly mount workspaces based on agent classification

### Test Case 5: Controller Reconciliation
**Objective**: Test controller handles conditional PVC logic correctly

**Steps**:
1. Submit CodeRun and monitor reconciliation logs
2. Verify agent name extraction logged correctly
3. Check agent classification logged correctly
4. Confirm PVC creation/retrieval process
5. Validate status updates reflect PVC operations

**Expected Result**: Reconciliation completes successfully with conditional PVC logic

### Test Case 6: Backward Compatibility
**Objective**: Ensure implementation agents continue using existing workspace pattern

**Steps**:
1. Create CodeRun for Rex agent
2. Verify continues using existing `workspace-cto` PVC
3. Create CodeRun for Blaze agent
4. Verify continues using existing `workspace-cto` PVC
5. Confirm no new PVCs created for implementation agents
6. Test existing workflows continue functioning

**Expected Result**: Implementation agents continue using shared workspace without disruption

### Test Case 7: Error Handling
**Objective**: Validate proper error handling for invalid inputs

**Steps**:
1. Submit CodeRun with invalid `github_app` value
2. Verify appropriate error messages in logs
3. Check status updates reflect extraction failure
4. Test recovery after fixing invalid input
5. Validate no resource leaks on errors

**Expected Result**: Clear error messages with graceful failure handling

### Test Case 8: Workspace Isolation vs Sharing
**Objective**: Confirm correct workspace strategy for different agent types

**Steps**:
1. Create CodeRuns for Rex and Blaze simultaneously
2. Verify both use same `workspace-cto` PVC (shared)
3. Create CodeRun for Cleo agent
4. Verify gets separate `workspace-cto-cleo` PVC (isolated)
5. Test workspace data isolation between different agent types
6. Validate workspace sharing between implementation agents

**Expected Result**: Correct workspace strategy applied based on agent classification

### Test Case 9: Performance Impact
**Objective**: Measure performance impact of new conditional logic

**Steps**:
1. Benchmark controller reconciliation time before changes
2. Measure reconciliation time with conditional PVC logic
3. Monitor memory usage during extended operation
4. Test concurrent CodeRun creation performance
5. Validate no degradation in controller responsiveness

**Expected Result**: Minimal performance impact from new conditional PVC logic

## Quality Criteria

### Code Quality Standards
- [ ] Rust code follows project conventions and style guidelines
- [ ] Comprehensive error handling with specific error types
- [ ] Unit tests cover all agent name extraction scenarios
- [ ] Unit tests cover all agent classification scenarios
- [ ] Integration tests validate PVC creation and management
- [ ] Documentation updated with new conditional PVC naming patterns

### Security Requirements
- [ ] Agent workspace isolation prevents cross-agent data access where required
- [ ] Shared workspace access properly controlled for implementation agents
- [ ] PVC permissions follow principle of least privilege
- [ ] No sensitive data exposed in PVC names or labels
- [ ] Proper RBAC permissions for controller PVC operations

### Operational Excellence
- [ ] Structured logging provides troubleshooting information
- [ ] Metrics available for monitoring PVC operations
- [ ] Clear documentation for operators and troubleshooting
- [ ] No migration required for existing deployments

## Deliverable Checklist

- [ ] `extract_agent_name()` function implemented and tested
- [ ] `AgentClassifier` struct implemented and tested
- [ ] Conditional PVC creation logic implemented
- [ ] Controller reconciliation modified to integrate new logic
- [ ] Backward compatibility maintained for implementation agents
- [ ] Unit tests cover all extraction and classification scenarios
- [ ] Integration tests validate end-to-end functionality
- [ ] Documentation updated with conditional PVC naming patterns
- [ ] Performance benchmarks confirm minimal impact

## Success Metrics

1. **Functionality**: 100% of supported GitHub App patterns extract correctly
2. **Classification**: Implementation agents correctly identified and use shared workspace
3. **Isolation**: Non-implementation agents get dedicated workspace PVCs
4. **Compatibility**: Zero disruption to existing implementation agent workflows
5. **Performance**: <5% impact on controller reconciliation time
6. **Reliability**: 99.9% PVC creation success rate

## Notes

- **CRITICAL**: Implementation agents (Rex, Blaze) must continue using shared workspace
- Backward compatibility ensures smooth operation for existing deployments
- Performance monitoring important due to controller-wide impact
- Clear error messages essential for operational troubleshooting
- Agent classification system designed for future extensibility
- No migration required - implementation agents continue using existing pattern
