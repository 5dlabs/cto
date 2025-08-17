# Task 3: Design Multi-Agent Workflow DAG Structure

## Overview

Create the core Argo Workflow template with parameterized agent selection, DAG task dependencies, and suspend points for event-driven transitions. This template forms the backbone of the multi-agent orchestration system, providing flexible agent assignment and reliable workflow state management.

## Technical Context

The workflow template must support configurable agent selection while maintaining strict event-driven coordination between stages. The DAG structure implements suspend/resume patterns that enable workflows to pause for external GitHub events, supporting development cycles that span days or weeks.

## Implementation Guide

### Phase 1: Base Workflow Template Structure

1. **Create Workflow Template Foundation**
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: WorkflowTemplate
   metadata:
     name: play-workflow-template
     namespace: argo
   spec:
     activeDeadlineSeconds: 1209600  # 14 days
     entrypoint: main
     arguments:
       parameters:
       - name: implementation-agent
         value: "5DLabs-Rex"
       - name: quality-agent
         value: "5DLabs-Cleo"
       - name: testing-agent
         value: "5DLabs-Tess"
       - name: task-id
         value: ""
       - name: repository
         value: "5dlabs/cto"
   ```

2. **Configure Workflow Labels**
   ```yaml
   metadata:
     generateName: play-workflow-
     labels:
       workflow-type: play-orchestration
       task-id: "{{workflow.parameters.task-id}}"
       repository: "{{workflow.parameters.repository}}"
   ```

### Phase 2: Implementation Work Task

1. **Define Implementation Task**
   ```yaml
   - name: implementation-work
     template: agent-coderun
     arguments:
       parameters:
       - name: github-app
         value: "{{workflow.parameters.implementation-agent}}"
       - name: task-id
         value: "{{workflow.parameters.task-id}}"
       - name: stage
         value: "implementation"
   ```

2. **Agent CodeRun Template**
   ```yaml
   - name: agent-coderun
     inputs:
       parameters:
       - name: github-app
       - name: task-id
       - name: stage
     resource:
       action: create
       manifest: |
         apiVersion: agents.platform/v1
         kind: CodeRun
         metadata:
           generateName: coderun-{{inputs.parameters.stage}}-
           labels:
             task-id: "{{inputs.parameters.task-id}}"
             github-app: "{{inputs.parameters.github-app}}"
             workflow-name: "{{workflow.name}}"
         spec:
           github_app: "{{inputs.parameters.github-app}}"
           service: "cto"
           model: "claude-3-5-sonnet-20241022"
           continue_session: true
   ```

### Phase 3: Suspend Point Implementation

1. **Wait-PR-Created Suspend Template**
   ```yaml
   - name: wait-pr-created
     suspend: {}
     metadata:
       labels:
         current-stage: waiting-pr-created
         task-id: "{{workflow.parameters.task-id}}"
   ```

2. **Stage Label Management**
   ```yaml
   # Update workflow labels on stage transitions
   - name: update-stage-label
     resource:
       action: patch
       manifest: |
         apiVersion: argoproj.io/v1alpha1
         kind: Workflow
         metadata:
           name: "{{workflow.name}}"
           labels:
             current-stage: "{{inputs.parameters.new-stage}}"
   ```

### Phase 4: Quality Work Task

1. **Quality Assurance Task Definition**
   ```yaml
   - name: quality-work
     dependencies: [wait-pr-created]
     template: agent-coderun
     arguments:
       parameters:
       - name: github-app
         value: "{{workflow.parameters.quality-agent}}"
       - name: task-id
         value: "{{workflow.parameters.task-id}}"
       - name: stage
         value: "quality"
   ```

2. **PR Information Propagation**
   ```yaml
   # Pass PR context from suspend resume
   - name: pr-url
     value: "{{steps.wait-pr-created.outputs.parameters.pr-url}}"
   - name: pr-number
     value: "{{steps.wait-pr-created.outputs.parameters.pr-number}}"
   ```

### Phase 5: Testing Work Implementation

1. **Testing Task Configuration**
   ```yaml
   - name: testing-work
     dependencies: [wait-ready-for-qa]
     template: agent-coderun
     arguments:
       parameters:
       - name: github-app
         value: "{{workflow.parameters.testing-agent}}"
       - name: task-id
         value: "{{workflow.parameters.task-id}}"
       - name: stage
         value: "testing"
       - name: pr-context
         value: "{{steps.wait-ready-for-qa.outputs.parameters.qa-status}}"
   ```

2. **Test Environment Setup**
   ```yaml
   # Additional testing-specific configuration
   env:
   - name: PR_URL
     value: "{{inputs.parameters.pr-url}}"
   - name: QA_STATUS
     value: "{{inputs.parameters.qa-status}}"
   - name: ENABLE_LIVE_TESTING
     value: "true"
   ```

### Phase 6: Workflow Completion

1. **Complete Task Implementation**
   ```yaml
   - name: complete-task
     dependencies: [wait-pr-approved]
     template: task-completion
     arguments:
       parameters:
       - name: task-id
         value: "{{workflow.parameters.task-id}}"
       - name: workflow-summary
         value: "{{steps.generate-summary.outputs.result}}"
   ```

2. **Cleanup and Archive**
   ```yaml
   - name: task-completion
     script:
       image: alpine:latest
       command: [sh]
       source: |
         #!/bin/sh
         echo "Completing task {{inputs.parameters.task-id}}"
         
         # Move task directory to completed
         mv docs/.taskmaster/docs/task-{{inputs.parameters.task-id}} \
            docs/.taskmaster/docs/.completed/task-{{inputs.parameters.task-id}}
         
         # Update task status
         echo "Task {{inputs.parameters.task-id}} completed at $(date)" >> \
              docs/.taskmaster/completion.log
         
         # Trigger next task workflow if available
         NEXT_TASK=$(({{inputs.parameters.task-id}} + 1))
         if [ -d "docs/.taskmaster/docs/task-$NEXT_TASK" ]; then
           echo "Starting next task: $NEXT_TASK"
           # Submit new workflow for next task
         fi
   ```

## Code Examples

### Complete DAG Structure
```yaml
templates:
- name: main
  dag:
    tasks:
    - name: implementation-work
      template: agent-coderun
      arguments:
        parameters:
        - name: github-app
          value: "{{workflow.parameters.implementation-agent}}"
    
    - name: wait-pr-created
      dependencies: [implementation-work]
      template: suspend-for-event
      arguments:
        parameters:
        - name: event-type
          value: "pr-created"
    
    - name: quality-work
      dependencies: [wait-pr-created]
      template: agent-coderun
      arguments:
        parameters:
        - name: github-app
          value: "{{workflow.parameters.quality-agent}}"
    
    - name: wait-ready-for-qa
      dependencies: [quality-work]
      template: suspend-for-event
      arguments:
        parameters:
        - name: event-type
          value: "ready-for-qa"
    
    - name: testing-work
      dependencies: [wait-ready-for-qa]
      template: agent-coderun
      arguments:
        parameters:
        - name: github-app
          value: "{{workflow.parameters.testing-agent}}"
    
    - name: wait-pr-approved
      dependencies: [testing-work]
      template: suspend-for-event
      arguments:
        parameters:
        - name: event-type
          value: "pr-approved"
    
    - name: complete-task
      dependencies: [wait-pr-approved]
      template: task-completion
```

### Parameterized Agent Selection
```yaml
# Workflow can be started with different agent combinations
argo submit play-workflow-template \
  --parameter implementation-agent="5DLabs-Blaze" \
  --parameter quality-agent="5DLabs-Cleo" \
  --parameter testing-agent="5DLabs-Tess" \
  --parameter task-id="5"
```

### Event Correlation Labels
```yaml
# Dynamic label management for event correlation
metadata:
  labels:
    workflow-type: play-orchestration
    task-id: "{{workflow.parameters.task-id}}"
    current-stage: "{{workflow.status.phase}}"
    implementation-agent: "{{workflow.parameters.implementation-agent}}"
    quality-agent: "{{workflow.parameters.quality-agent}}"
    testing-agent: "{{workflow.parameters.testing-agent}}"
```

## Architecture Patterns

### Event-Driven Workflow State Machine
The DAG implements a sophisticated state machine:
1. **Sequential Stages**: Implementation → Quality → Testing → Completion
2. **Suspend Points**: Workflow pauses between stages for external events
3. **Event Correlation**: GitHub webhooks resume specific workflow instances
4. **State Propagation**: Information flows between stages through parameters

### Parameterized Agent Architecture
- **No Hardcoded Agents**: All agent references use workflow parameters
- **Flexible Selection**: Support for Rex, Blaze, Morgan, Cleo, Tess, and future agents
- **Consistent Interface**: Same CodeRun template works for all agents
- **Agent-Specific Configuration**: Templates handle agent differences transparently
- **Multi-Agent Remediation**: Event-driven system supports any implementation agent (Rex, Blaze, Morgan) triggering QA pipeline restarts

### Workflow Lifecycle Management
```yaml
# Workflow progresses through defined stages:
Stage 1: implementation-work (Rex/Blaze/Morgan execution)
  ↓
Stage 2: wait-pr-created (suspend for PR creation event)
  ↓
Stage 3: quality-work (Cleo execution)
  ↓
Stage 4: wait-ready-for-qa (suspend for QA ready event)
  ↓
Stage 5: testing-work (Tess execution)
  ↓
Stage 6: wait-pr-approved (suspend for approval event)
  ↓
Stage 7: complete-task (cleanup and next task trigger)
```

## Key Implementation Details

### Workflow Template Structure
- **WorkflowTemplate**: Reusable template for multiple task executions
- **Parameters**: Configurable agent selection and task metadata
- **Labels**: Comprehensive labeling for event correlation and monitoring
- **Timeout**: 14-day deadline accommodates realistic development cycles

### Suspend/Resume Architecture
- **Indefinite Suspend**: No timeout on suspend points by default
- **Event-Driven Resume**: Argo Events sensors trigger workflow resumption
- **State Persistence**: Workflow state and parameters preserved during suspension
- **Resume Data**: External events can provide data to resumed workflows

### Agent Integration
- **CodeRun CRD**: Standard interface for all agent types
- **Session Continuity**: Agents continue previous sessions using `continue_session: true`
- **Workspace Isolation**: Each agent gets dedicated PVC workspace
- **Resource Management**: Proper limits and cleanup for long-running workflows

## Testing and Validation

### Template Validation
1. **Dry Run Testing**: `argo template create --dry-run play-workflow-template.yaml`
2. **Parameter Validation**: Test with various agent combinations
3. **DAG Visualization**: Verify structure in Argo UI
4. **Timeout Testing**: Confirm 14-day timeout works correctly

### Suspend/Resume Testing
1. **Create Test Workflow**: Start workflow and verify first suspend
2. **Manual Resume**: Use `argo resume` to test resumption
3. **Parameter Propagation**: Verify data flows between stages
4. **Label Correlation**: Test event correlation with workflow labels

### Integration Testing
1. **End-to-End Flow**: Complete workflow with all agents
2. **Agent Substitution**: Test different agent combinations
3. **Error Handling**: Test failures and recovery scenarios
4. **Resource Cleanup**: Verify proper cleanup on completion

## References

- [Argo Workflows DAG Documentation](https://argoproj.github.io/argo-workflows/walk-through/dag/)
- [Suspend and Resume](https://argoproj.github.io/argo-workflows/walk-through/suspend-resume/)
- [Workflow Templates](https://argoproj.github.io/argo-workflows/workflow-templates/)
- [Multi-Agent Architecture](.taskmaster/docs/architecture.md)
- [CodeRun CRD Specification](controller/src/crds/coderun.rs)