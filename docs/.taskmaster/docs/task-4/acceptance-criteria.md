# Acceptance Criteria: Implement Agent-Specific PVC Naming

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

### 2. PVC Naming Pattern Implementation
- [ ] PVC names follow `workspace-{service}-{agent}` pattern
- [ ] `workspace-cto-rex` created for Rex agent on cto service
- [ ] `workspace-cto-cleo` created for Cleo agent on cto service
- [ ] `workspace-cto-tess` created for Tess agent on cto service
- [ ] Agent names properly validated for Kubernetes compliance
- [ ] PVC names truncated if necessary to meet 63-character limit

### 3. Controller Integration
- [ ] Reconciliation logic updated to use agent-specific PVC naming
- [ ] PVC creation integrated with existing controller workflow
- [ ] Pod creation uses correct agent-specific PVC for volume mounts
- [ ] Error handling implemented for agent name extraction failures
- [ ] Status updates reflect PVC creation success/failure

### 4. Backward Compatibility
- [ ] Existing workflows with legacy PVC names continue to function
- [ ] Migration logic handles transition from old to new naming
- [ ] Graceful fallback when agent extraction fails
- [ ] No disruption to currently running CodeRun instances

### 5. Workspace Isolation
- [ ] Rex agent gets dedicated `workspace-cto-rex` PVC
- [ ] Cleo agent gets dedicated `workspace-cto-cleo` PVC  
- [ ] Tess agent gets dedicated `workspace-cto-tess` PVC
- [ ] Agents cannot access other agents' workspace data
- [ ] Session continuity maintained within agent-specific workspaces

## Technical Requirements

### Code Implementation
- [ ] `extract_agent_name()` function uses regex for robust parsing
- [ ] Kubernetes naming validation prevents invalid PVC names
- [ ] Idempotent PVC creation using kube-rs API
- [ ] Proper error handling with descriptive error messages
- [ ] Comprehensive logging for troubleshooting

### Resource Management  
- [ ] PVC creation follows Kubernetes best practices
- [ ] Proper labels applied to agent-specific PVCs
- [ ] Resource limits and requests configured appropriately
- [ ] PVC cleanup handled correctly on resource deletion

### Performance
- [ ] Agent name extraction completes in <1ms
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
4. Test invalid inputs return appropriate errors
5. Test case sensitivity handling

**Expected Result**: All agent names extracted correctly with proper error handling

### Test Case 2: PVC Creation with New Naming
**Objective**: Validate PVCs created with agent-specific naming pattern

**Steps**:
1. Submit CodeRun with `github_app: "5DLabs-Rex"`
2. Verify PVC created with name `workspace-cto-rex`
3. Check PVC has correct labels and annotations
4. Confirm PVC meets Kubernetes naming constraints
5. Repeat for Cleo and Tess agents

**Expected Result**: Agent-specific PVCs created successfully with correct naming

### Test Case 3: Pod Volume Mount Integration
**Objective**: Ensure pods mount correct agent-specific PVCs

**Steps**:
1. Create CodeRun for Rex agent
2. Verify pod mounts `workspace-cto-rex` PVC
3. Check workspace directory is accessible in pod
4. Confirm agent isolation by testing multiple agents
5. Validate session continuity with agent-specific workspace

**Expected Result**: Pods correctly mount agent-specific workspaces

### Test Case 4: Controller Reconciliation
**Objective**: Test controller handles agent-specific PVC logic correctly

**Steps**:
1. Submit CodeRun and monitor reconciliation logs
2. Verify agent name extraction logged correctly  
3. Check PVC creation/retrieval process
4. Confirm pod creation with proper volume mounts
5. Validate status updates reflect PVC operations

**Expected Result**: Reconciliation completes successfully with new PVC logic

### Test Case 5: Backward Compatibility
**Objective**: Ensure existing workflows continue functioning

**Steps**:
1. Create legacy PVC with old naming pattern
2. Submit CodeRun that would create new-style PVC
3. Verify system handles both naming patterns
4. Test migration logic for existing workspaces
5. Confirm no disruption to running workflows

**Expected Result**: Legacy and new naming patterns coexist during transition

### Test Case 6: Error Handling
**Objective**: Validate proper error handling for invalid inputs

**Steps**:
1. Submit CodeRun with invalid `github_app` value
2. Verify appropriate error messages in logs
3. Check status updates reflect extraction failure
4. Test recovery after fixing invalid input
5. Validate no resource leaks on errors

**Expected Result**: Clear error messages with graceful failure handling

### Test Case 7: Workspace Isolation
**Objective**: Confirm agents cannot access other agents' workspaces

**Steps**:
1. Create CodeRuns for Rex, Cleo, and Tess simultaneously
2. Verify each gets separate PVC workspace
3. Check agents cannot read/write to other agents' PVCs
4. Validate session data isolation between agents
5. Test workspace cleanup on agent completion

**Expected Result**: Complete workspace isolation between different agents

### Test Case 8: Performance Impact
**Objective**: Measure performance impact of new PVC logic

**Steps**:
1. Benchmark controller reconciliation time before changes
2. Measure reconciliation time with agent-specific logic
3. Monitor memory usage during extended operation
4. Test concurrent CodeRun creation performance
5. Validate no degradation in controller responsiveness

**Expected Result**: Minimal performance impact from new PVC logic

## Quality Criteria

### Code Quality Standards
- [ ] Rust code follows project conventions and style guidelines
- [ ] Comprehensive error handling with specific error types
- [ ] Unit tests cover all agent name extraction scenarios
- [ ] Integration tests validate PVC creation and management
- [ ] Documentation updated with new PVC naming patterns

### Security Requirements
- [ ] Agent workspace isolation prevents cross-agent data access
- [ ] PVC permissions follow principle of least privilege
- [ ] No sensitive data exposed in PVC names or labels
- [ ] Proper RBAC permissions for controller PVC operations

### Operational Excellence
- [ ] Structured logging provides troubleshooting information
- [ ] Metrics available for monitoring PVC operations
- [ ] Clear documentation for operators and troubleshooting
- [ ] Migration guide for existing deployments

## Deliverable Checklist

- [ ] `extract_agent_name()` function implemented and tested
- [ ] PVC creation logic updated to use agent-specific naming
- [ ] Controller reconciliation modified to integrate new logic
- [ ] Backward compatibility maintained for existing workflows
- [ ] Unit tests cover all extraction and validation scenarios
- [ ] Integration tests validate end-to-end functionality
- [ ] Documentation updated with new PVC naming patterns
- [ ] Performance benchmarks confirm minimal impact

## Success Metrics

1. **Functionality**: 100% of supported GitHub App patterns extract correctly
2. **Isolation**: Each agent gets dedicated workspace PVC
3. **Compatibility**: Zero disruption to existing workflows during transition
4. **Performance**: <5% impact on controller reconciliation time
5. **Reliability**: 99.9% PVC creation success rate

## Notes

- Agent workspace isolation is critical for multi-agent coordination
- Backward compatibility ensures smooth transition for existing deployments  
- Performance monitoring important due to controller-wide impact
- Clear error messages essential for operational troubleshooting
- Consider future agent types in extraction logic design