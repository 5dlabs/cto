# Toolman Guide: Workflow Stage Transitions

## Overview

This task implements atomic workflow stage management for multi-agent orchestration. You'll create label update mechanisms that coordinate agent handoffs through event-driven state transitions.

Scope: Enhance the existing workflow. Stage labels at suspend points and correlation sensors are already implemented. Add explicit post-agent stage transition steps and atomic label updates; do not modify correlation sensors beyond verification.

## Tool Selection Strategy

### Primary Development Tools

**filesystem** - Essential for workflow template development
- Create and modify Argo Workflows template files
- Read existing workflow patterns and structures
- Implement stage transition templates and scripts
- Organize workflow template files logically

**kubernetes** - Critical for Kubernetes resource management
- Test kubectl patch operations for label updates
- Validate workflow label updates work correctly
- Debug workflow state and label issues
- Apply and test Argo Events sensor configurations

### Research and Documentation Tools

**memory_create_entities** - Store implementation knowledge
- Document stage transition patterns and requirements
- Track label update strategies and approaches
- Remember workflow template structures and dependencies
- Store testing scenarios and validation methods

**brave_web_search** - Supplemental research tool
- Research Argo Workflows label management patterns
- Find Kubernetes JSON patch operation examples
- Research event-driven workflow coordination patterns
- Lookup Argo Events sensor configuration best practices

## Implementation Workflow

### Phase 1: Research and Planning
```
Tools: filesystem, memory_create_entities, brave_web_search
```

1. **Analyze Existing Workflow Structure**
   - Use `filesystem` to examine current workflow templates
   - Study existing suspend/resume patterns
   - Document current label usage and patterns

2. **Research Label Update Mechanisms**
   - Use `brave_web_search` for Kubernetes patch operation examples
   - Research atomic update patterns and race condition prevention
   - Find Argo Workflows resource template examples

3. **Plan Stage Transition Architecture**
   - Use `memory_create_entities` to document stage progression flow
   - Define label structure and naming conventions
   - Plan integration with existing Argo Events sensors

### Phase 2: Template Development
```
Tools: filesystem, memory_create_entities
```

1. **Create Stage Update Template**
   ```yaml
   # Focus areas for update-workflow-stage template
   - Atomic label update using resource template
   - Input parameter validation
   - Error handling and verification
   - Idempotent operation design
   ```

2. **Implement Stage Transition Logic**
   ```yaml
   # Integration points in main workflow template
   - Add stage updates after each agent completion
   - Integrate suspend points between stages
   - Add proper dependency chains
   - Include error handling for failed updates
   ```

3. **Create Verification Scripts**
   ```bash
   # Shell script templates for stage validation
   - Verify label updates succeeded
   - Check workflow state consistency
   - Validate stage progression logic
   - Monitor for race conditions
   ```

### Phase 3: Kubernetes Integration
```
Tools: kubernetes, filesystem, memory_create_entities
```

1. **Test Label Update Operations**
   - Use `kubernetes` to test kubectl patch commands
   - Validate JSON merge patch operations work correctly
   - Test concurrent update scenarios
   - Verify label selector targeting works

2. **Validate Workflow State Management**
   ```bash
   # Test workflow label updates
   kubectl patch workflow test-workflow \
     --type='merge' \
     --patch='{"metadata":{"labels":{"current-stage":"waiting-pr-created"}}}'
   
   # Verify update succeeded
   kubectl get workflow test-workflow -o jsonpath='{.metadata.labels.current-stage}'
   ```

3. **Test Argo Events Integration**
   - Create test sensors with stage-aware label selectors
   - Validate event correlation logic works correctly
   - Test workflow resumption at correct stages

### Phase 4: Testing and Validation
```
Tools: kubernetes, filesystem, memory_create_entities
```

1. **Create Test Scenarios**
   - Test single workflow progressing through all stages
   - Test multiple concurrent workflows with different stages
   - Test error scenarios and recovery mechanisms

2. **Performance and Concurrency Testing**
   - Test high-frequency label updates
   - Validate no race conditions in concurrent scenarios
   - Monitor resource usage during stage transitions

3. **Integration Testing**
   - Test end-to-end workflow with GitHub events
   - Validate agent handoffs work correctly
   - Test error recovery and rollback scenarios

## Best Practices

### Atomic Operations
- **JSON Merge Patches**: Use merge patches for atomic label updates
- **Resource Versioning**: Include resource versions to prevent concurrent modification
- **Verification**: Always verify updates succeeded before proceeding
- **Idempotent Design**: Ensure all operations are safe to retry

### Error Handling
- **Clear Error Messages**: Provide detailed error information for debugging
- **Graceful Failures**: Fail workflows cleanly when critical updates fail
- **Recovery Mechanisms**: Provide ways to recover from failed stage transitions
- **Monitoring Integration**: Log all stage transitions for observability

### Performance Optimization
- **Minimal API Calls**: Use efficient patch operations instead of full updates
- **Batch Operations**: Group related label updates when possible
- **Async Operations**: Don't block workflow progression on non-critical updates
- **Resource Limits**: Set appropriate timeouts and resource limits

## Tool Usage Examples

### Reading Existing Workflow Templates
```bash
# Use filesystem to analyze current structures
filesystem.read_file("infra/argo-workflows/templates/coderun-template.yaml")
filesystem.list_directory("infra/argo-workflows/")
```

### Testing Kubernetes Operations
```bash
# Use kubernetes for testing label operations
kubernetes.kubectl_patch("workflow", "test-workflow", 
  '{"metadata":{"labels":{"current-stage":"new-stage"}}}')
kubernetes.kubectl_get("workflow", "test-workflow", "-o", "jsonpath='{.metadata.labels}'")
```

### Creating New Templates
```bash
# Use filesystem to create workflow templates
filesystem.write_file("infra/argo-workflows/templates/stage-transitions.yaml", template_content)
filesystem.write_file("scripts/validate-stage-transition.sh", script_content)
```

### Research and Documentation
```bash
# Use memory_create_entities to document findings
memory_create_entities([{
  "name": "Stage Transition Pattern",
  "description": "Atomic label update using Kubernetes resource template",
  "implementation": "JSON merge patch with verification"
}])
```

## Common Pitfalls to Avoid

1. **Race Conditions**: Always use atomic operations for concurrent label updates
2. **Resource Versioning**: Include resource versions to prevent concurrent modification conflicts
3. **Event Correlation**: Ensure label selectors precisely target intended workflows
4. **Error Propagation**: Don't silently ignore label update failures
5. **Performance Impact**: Minimize overhead from frequent label updates
6. **State Consistency**: Always verify stage transitions completed successfully

## Workflow Template Patterns

### Stage Update Template Structure
```yaml
- name: update-workflow-stage
  inputs:
    parameters:
    - name: new-stage
  resource:
    action: patch
    source:
      resource:
        apiVersion: argoproj.io/v1alpha1
        kind: Workflow
        metadata:
          name: "{{workflow.name}}"
    patch: |
      {"metadata":{"labels":{"current-stage":"{{inputs.parameters.new-stage}}"}}}
    patchType: "application/merge-patch+json"
```

### DAG Integration Pattern
```yaml
dag:
  tasks:
  - name: agent-work
    template: agent-coderun
  - name: update-stage
    dependencies: [agent-work]
    template: update-workflow-stage
    arguments:
      parameters:
      - name: new-stage
        value: "next-stage"
  - name: wait-for-event
    dependencies: [update-stage]
    template: suspend-for-webhook
```

### Event Sensor Pattern
```yaml
triggers:
- template:
    name: resume-workflow
    argoWorkflow:
      operation: resume
      source:
        resource:
          labelSelector: |
            workflow-type=play-orchestration,
            current-stage=waiting-pr-created,
            task-id={{extracted-task-id}}
```

## Success Validation

### Template Quality Checks
- [ ] All workflow templates render without errors
- [ ] Stage update templates perform atomic operations
- [ ] Label updates include proper verification
- [ ] Error handling covers all failure scenarios

### Integration Quality Checks
- [ ] Argo Events sensors target workflows correctly
- [ ] Label selectors precisely match intended workflows
- [ ] Event correlation extracts task IDs correctly
- [ ] Workflow resumption works at all stages

### Performance Quality Checks
- [ ] Stage transitions complete within acceptable time
- [ ] Concurrent workflows don't interfere with each other
- [ ] High-frequency updates don't cause resource issues
- [ ] System scales to handle multiple concurrent workflows

This implementation requires careful attention to atomic operations and event-driven coordination. Focus on creating reliable state management that prevents race conditions while enabling precise workflow coordination through label-based event correlation.