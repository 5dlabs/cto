# Toolman Guide: Rex Remediation Sensor



## Overview

This task implements a dedicated Argo Events Sensor to detect Rex push events and automatically restart the QA pipeline with downstream agent cancellation. You'll create event-driven remediation that ensures clean pipeline restarts when Rex addresses PR feedback.

## Tool Selection Strategy

### Primary Development Tools



**filesystem** - Essential for sensor and workflow template creation


- Create Argo Events EventSource and Sensor YAML configurations


- Develop remediation workflow templates with proper step sequencing


- Read existing Argo Events configurations for pattern consistency


- Organize event handling and workflow files logically



**kubernetes** - Critical for Kubernetes operations and testing


- Test kubectl commands for agent cancellation using label selectors


- Apply and validate Argo Events configurations in cluster


- Debug CodeRun CRD deletion and workflow state management


- Test event-driven workflow triggers and resumption



**git** - Required for change tracking and validation


- Track sensor configuration changes and template development


- Review existing event handling patterns in codebase


- Commit incremental implementation progress


- Validate branch name parsing and task ID extraction logic

### Research and Documentation Tools



**memory_create_entities** - Store implementation knowledge


- Document event processing flows and remediation sequences


- Track GitHub webhook payload structures and filtering patterns


- Remember agent cancellation strategies and label selector approaches


- Store testing scenarios and validation procedures



**brave_web_search** - Supplemental research tool


- Research Argo Events sensor configuration best practices


- Find GitHub webhook payload examples and filtering patterns


- Research Kubernetes label selector syntax and usage patterns


- Lookup event-driven workflow coordination approaches

## Implementation Workflow

### Phase 1: Research and Planning






```
Tools: filesystem, memory_create_entities, brave_web_search






```



1. **Analyze Existing Event Infrastructure**


   - Use `filesystem` to examine current Argo Events configurations


   - Study existing GitHub webhook EventSource setups


   - Document current sensor patterns and trigger mechanisms



2. **Research GitHub Push Event Structure**


   - Use `brave_web_search` for GitHub webhook payload documentation


   - Study sender identification and branch reference extraction


   - Plan task ID extraction from branch name patterns



3. **Design Remediation Flow**


   - Use `memory_create_entities` to document remediation sequence


   - Plan agent cancellation strategy using label selectors


   - Design workflow state reset and resumption approach

### Phase 2: EventSource and Sensor Development






```
Tools: filesystem, kubernetes, memory_create_entities






```



1. **Create Rex Push EventSource**
   ```yaml
   # Focus areas for EventSource configuration


   - GitHub webhook endpoint configuration


   - Event filtering for Rex push events


   - Task branch pattern matching


   - Sender validation and authentication





```



2. **Implement Rex Remediation Sensor**
   ```yaml
   # Focus areas for Sensor configuration


   - Event dependency and filtering logic


   - Workflow trigger with proper parameter extraction


   - Task ID correlation from branch references


   - Error handling and validation





```



3. **Test Event Detection**


   - Use `kubernetes` to apply EventSource and Sensor configurations


   - Test webhook event processing with simulated payloads


   - Validate task ID extraction from various branch patterns

### Phase 3: Remediation Workflow Implementation






```
Tools: filesystem, kubernetes, memory_create_entities






```



1. **Create Agent Cancellation Logic**
   ```bash
   # Focus areas for agent cancellation


   - Label selector precision for targeting


   - CodeRun CRD deletion with proper validation


   - Verification of successful cancellation


   - Error handling for missing agents





```



2. **Implement GitHub API Integration**
   ```bash
   # Focus areas for GitHub operations


   - PR discovery using task labels


   - Ready-for-QA label removal


   - GitHub API authentication and error handling


   - Repository and PR state validation





```



3. **Create Workflow State Management**
   ```bash
   # Focus areas for workflow management


   - Main workflow discovery and targeting


   - Atomic stage label updates


   - Workflow resumption at correct stages


   - State consistency validation





```

### Phase 4: Testing and Validation






```
Tools: kubernetes, filesystem, memory_create_entities






```



1. **Create Test Scenarios**


   - Test CodeRun CRD creation and cancellation


   - Test event simulation with various webhook payloads


   - Test concurrent remediation scenarios



2. **Integration Testing**


   - Test complete remediation flow end-to-end


   - Test interaction with main play workflows


   - Test GitHub API integration with real PRs



3. **Safety and Validation Testing**


   - Test event filtering prevents false positives


   - Test agent cancellation precision


   - Test idempotency with duplicate events



## Best Practices

### Event Processing Safety
- **Validation First**: Always validate event payload before processing
- **Precise Filtering**: Use specific filters to prevent false positive triggers
- **Sender Verification**: Confirm events actually come from 5DLabs-Rex[bot]
- **Task Correlation**: Extract and validate task IDs before any operations

### Agent Cancellation Safety
- **Label Selector Precision**: Use exact label matching to target only relevant agents
- **Existence Checks**: Verify agents exist before attempting deletion
- **Selective Targeting**: Never delete Rex agents during remediation
- **Verification**: Confirm all targeted agents cancelled successfully

### Workflow Coordination
- **Atomic Operations**: Use atomic operations for workflow state updates
- **State Consistency**: Verify workflow state changes before proceeding
- **Error Recovery**: Provide mechanisms to recover from partial failures
- **Timing Coordination**: Ensure proper sequencing of remediation steps



## Tool Usage Examples

### Reading Existing Configurations



```bash
# Use filesystem to examine current Argo Events setup
filesystem.read_file("infra/argo-events/sensors/github-sensors.yaml")
filesystem.list_directory("infra/argo-events/")






```

### Testing Kubernetes Operations



```bash
# Use kubernetes for testing agent operations
kubernetes.kubectl_get("coderun", "-l", "task-id=3,github-app=5DLabs-Cleo")
kubernetes.kubectl_delete("coderun", "-l", "task-id=3,github-app in (5DLabs-Cleo,5DLabs-Tess)")






```

### Creating Sensor Configurations



```bash
# Use filesystem to create sensor and workflow files
filesystem.write_file("infra/argo-events/sensors/rex-remediation-sensor.yaml", sensor_config)
filesystem.write_file("infra/argo-workflows/templates/rex-remediation-workflow.yaml", workflow_template)






```

### Validating Changes



```bash
# Use git to track configuration changes
git.status()  # Check current changes
git.diff()    # Review modifications
git.log()     # Review related changes






```

## Common Pitfalls to Avoid

1. **False Positive Events**: Ensure event filtering is precise enough to avoid non-Rex triggers
2. **Agent Over-Cancellation**: Don't cancel Rex agents or agents from other tasks
3. **Race Conditions**: Handle concurrent remediation requests safely
4. **GitHub API Failures**: Include proper error handling for GitHub operations
5. **Workflow State Corruption**: Verify all state changes before proceeding
6. **Incomplete Remediation**: Ensure all steps complete before marking success

## Event Processing Patterns

### EventSource Configuration Pattern



```yaml
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: rex-remediation-events
spec:
  github:
    rex-push:
      events: ["push"]
      filter:
        expression: |
          body.sender.login == "5DLabs-Rex[bot]" &&
          body.ref matches "refs/heads/task-.*"






```

### Sensor Trigger Pattern



```yaml
triggers:
- template:
    name: rex-remediation-trigger
    conditions: "rex-push-event"
    argoWorkflow:
      operation: submit
      source:
        resource:
          apiVersion: argoproj.io/v1alpha1
          kind: Workflow
          metadata:
            generateName: rex-remediation-






```

### Agent Cancellation Pattern



```bash
# Precise label selector for agent targeting
kubectl delete coderun \


  -l "task-id=${TASK_ID},github-app in (5DLabs-Cleo,5DLabs-Tess)" \


  --ignore-not-found






```

## Success Validation

### Event Detection Quality Checks


- [ ] Rex push events detected accurately with no false positives


- [ ] Task ID extraction works for all branch naming patterns


- [ ] Event filtering prevents processing of non-Rex events


- [ ] Webhook payload validation handles malformed events correctly

### Agent Cancellation Quality Checks


- [ ] Only targeted agents cancelled (no over-cancellation)


- [ ] Agent cancellation completes before proceeding to next steps


- [ ] Missing agents handled gracefully without errors


- [ ] Verification confirms all targeted agents successfully cancelled

### Workflow Integration Quality Checks


- [ ] GitHub API operations complete successfully


- [ ] Workflow stage resets performed atomically


- [ ] Main workflow resumption works correctly


- [ ] Complete remediation flow completes within acceptable timeframe

### Safety and Reliability Checks


- [ ] Multiple concurrent remediation requests handled safely


- [ ] Duplicate events processed idempotently without side effects


- [ ] Partial failures can be recovered manually or automatically


- [ ] System maintains consistency even during error scenarios

This implementation requires careful attention to event processing safety and precise agent targeting. Focus on creating reliable remediation that automatically handles Rex pushes while preventing any unintended cancellations or system disruptions.