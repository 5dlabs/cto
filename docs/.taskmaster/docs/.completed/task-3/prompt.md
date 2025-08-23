# Autonomous Agent Prompt: Design Multi-Agent Workflow DAG Structure

## 🚨 CRITICAL: Argo Events Reference Documentation

**BEFORE implementing ANY Argo Events sensors/triggers, MUST review official examples:**
- **Location:** [docs/references/argo-events/](../../../references/argo-events/)
- **Key Files:**
  - `github.yaml` - GitHub webhook sensor patterns
  - `complete-trigger-parameterization.yaml` - Dynamic parameter extraction
  - `special-workflow-trigger.yaml` - ArgoWorkflow operations (submit/resume)
  - `trigger-standard-k8s-resource.yaml` - K8s resource creation patterns

**❌ UNSUPPORTED Operations (will cause deployment failures):**
- `operation: delete` ❌
- `operation: patch` ❌
- `operation: update` ❌
- Template variables in `labelSelector` ❌

**✅ SUPPORTED Operations:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows via args: `[workflow-name]`)
- `dest: metadata.name` (for submit paths; for resume prefer args)

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!

## Mission

You are tasked with designing and implementing the core Argo Workflow template that orchestrates multi-agent collaboration through event-driven coordination. Your mission is to create a robust, parameterized DAG structure that enables flexible agent selection while maintaining reliable workflow state management across development cycles spanning days or weeks.

## Context

The multi-agent orchestration system requires a sophisticated workflow template that can coordinate Rex, Cleo, and Tess agents (or their alternatives) through a series of implementation, quality assurance, and testing phases. The workflow must support suspend/resume patterns, event-driven state transitions, and comprehensive parameter propagation between stages.

## Objectives

1. **Create Base Workflow Template Structure**
   - Design WorkflowTemplate with parameterized agent selection
   - Configure 14-day timeout for realistic development cycles
   - Implement comprehensive labeling for event correlation
   - Set up proper metadata and workflow generation patterns

2. **Implement Sequential DAG Tasks**
   - Create implementation-work task for Rex/Blaze execution
   - Design quality-work task for Cleo processing
   - Build testing-work task for Tess validation
   - Implement complete-task for workflow finalization

3. **Design Suspend Points for Event Coordination**
   - Create wait-pr-created suspend after implementation
   - Implement wait-ready-for-qa suspend after quality work
   - Design wait-pr-approved suspend after testing
   - Configure indefinite suspend with proper correlation labels

4. **Build Parameter Propagation System**
   - Enable data flow between workflow stages
   - Implement PR context sharing across tasks
   - Design QA feedback propagation to testing phase
   - Create workflow summary and completion reporting

5. **Implement Workflow State Management**
   - Dynamic label updates for stage tracking
   - Event correlation through task-id and stage labels
   - Proper workflow lifecycle management
   - Resource cleanup and task progression logic

## Technical Requirements

### Workflow Template Structure
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

### DAG Task Dependencies
```yaml
templates:
- name: main
  dag:
    tasks:
    - name: implementation-work
      template: agent-coderun

    - name: wait-pr-created
      dependencies: [implementation-work]
      template: suspend-for-event

    - name: quality-work
      dependencies: [wait-pr-created]
      template: agent-coderun

    - name: wait-ready-for-qa
      dependencies: [quality-work]
      template: suspend-for-event

    - name: testing-work
      dependencies: [wait-ready-for-qa]
      template: agent-coderun

    - name: wait-pr-approved
      dependencies: [testing-work]
      template: suspend-for-event

    - name: complete-task
      dependencies: [wait-pr-approved]
      template: task-completion
```

### Agent CodeRun Template
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

## Implementation Strategy

### Phase 1: Foundation Setup
1. Create base WorkflowTemplate with proper metadata
2. Define all required parameters with sensible defaults
3. Set up workflow generation naming and labeling
4. Configure 14-day timeout and resource management

### Phase 2: Core DAG Implementation
1. Design main DAG template with task dependencies
2. Create agent-coderun template for all agent types
3. Implement suspend-for-event template for pauses
4. Build task-completion template for finalization

### Phase 3: Event Correlation System
1. Implement dynamic label management for stage tracking
2. Create correlation labels for event targeting
3. Design parameter passing between stages
4. Build workflow state transition logic

### Phase 4: Advanced Features
1. Implement resource cleanup and archival
2. Create next-task progression logic
3. Build comprehensive workflow summary
4. Add error handling and recovery patterns

## Workflow Template Components

### Parameter System
```yaml
arguments:
  parameters:
  - name: implementation-agent
    description: "Agent to use for implementation work"
    value: "5DLabs-Rex"
  - name: quality-agent
    description: "Agent to use for quality assurance"
    value: "5DLabs-Cleo"
  - name: testing-agent
    description: "Agent to use for testing and validation"
    value: "5DLabs-Tess"
  - name: task-id
    description: "Unique task identifier for correlation"
  - name: repository
    description: "GitHub repository for the work"
    value: "5dlabs/cto"
```

### Suspend Template Design
```yaml
- name: suspend-for-event
  inputs:
    parameters:
    - name: event-type
    - name: stage-name
  suspend: {}
  metadata:
    labels:
      current-stage: "waiting-{{inputs.parameters.event-type}}"
      task-id: "{{workflow.parameters.task-id}}"
      workflow-type: play-orchestration
```

### Completion Template
```yaml
- name: task-completion
  inputs:
    parameters:
    - name: task-id
    - name: workflow-summary
  script:
    image: alpine:latest
    command: [sh]
    source: |
      #!/bin/sh
      echo "Completing task {{inputs.parameters.task-id}}"

      # Archive task to completed directory
      mv docs/.taskmaster/docs/task-{{inputs.parameters.task-id}} \
         docs/.taskmaster/docs/.completed/task-{{inputs.parameters.task-id}}

      # Generate completion report
      echo "{{inputs.parameters.workflow-summary}}" > \
           docs/.taskmaster/reports/task-{{inputs.parameters.task-id}}-summary.md

      # Trigger next task if available
      NEXT_TASK=$(({{inputs.parameters.task-id}} + 1))
      if [ -d "docs/.taskmaster/docs/task-$NEXT_TASK" ]; then
        # Submit workflow for next task
        argo submit play-workflow-template \
          --parameter task-id="$NEXT_TASK" \
          --parameter implementation-agent="{{workflow.parameters.implementation-agent}}" \
          --parameter quality-agent="{{workflow.parameters.quality-agent}}" \
          --parameter testing-agent="{{workflow.parameters.testing-agent}}"
      fi
```

## Quality Assurance Patterns

### Parameterized Agent Selection
- **No Hardcoded Names**: All agent references through parameters
- **Default Values**: Sensible defaults for standard agent configuration
- **Flexible Substitution**: Support for Rex, Blaze, and future agents
- **Consistent Interface**: Same template works for all agent types

### Event Correlation Design
```yaml
# Workflow labels for event targeting
metadata:
  labels:
    workflow-type: play-orchestration
    task-id: "{{workflow.parameters.task-id}}"
    current-stage: "{{workflow.status.phase}}"
    repository: "{{workflow.parameters.repository}}"

# Stage-specific correlation
- name: update-workflow-stage
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

### Resource Management
- **Extended Timeout**: 14-day activeDeadlineSeconds for realistic cycles
- **Proper Cleanup**: Resource deletion on workflow completion
- **PVC Management**: Shared workspace for agent coordination
- **Memory Efficiency**: Suspended workflows consume minimal resources

## Testing and Validation Requirements

### Template Validation
1. **Syntax Validation**: Use `argo template create --dry-run`
2. **Parameter Testing**: Validate with different agent combinations
3. **DAG Structure**: Verify dependencies and task ordering
4. **Label Correlation**: Test event targeting with proper labels

### Functional Testing
1. **Suspend/Resume**: Test indefinite suspend and manual resume
2. **Parameter Propagation**: Verify data flows between stages
3. **Agent Integration**: Test CodeRun creation with all agent types
4. **Workflow Completion**: Validate cleanup and next-task progression

### Integration Testing
1. **End-to-End Flow**: Complete workflow with real agents
2. **Event Coordination**: Test with actual GitHub webhook events
3. **Error Scenarios**: Handle agent failures and timeout conditions
4. **Concurrent Workflows**: Test multiple tasks running simultaneously

## Success Criteria

- WorkflowTemplate deploys successfully to Argo namespace
- DAG visualization displays correctly in Argo UI
- Parameter system enables flexible agent selection
- Suspend points pause workflow execution indefinitely
- Event correlation labels enable precise workflow targeting
- Resource cleanup executes properly on workflow completion
- Next-task progression triggers automatically when appropriate
- No hardcoded agent names in any template component

## Key Implementation Notes

### Workflow Design Principles
- **Event-Driven Architecture**: All stage transitions triggered by external events
- **Parameterized Configuration**: No hardcoded values, full customization
- **Resource Efficiency**: Minimal resource usage during suspended states
- **Operational Visibility**: Comprehensive labeling for monitoring and debugging

### Agent Integration Patterns
- **Consistent Interface**: Same CodeRun template for all agent types
- **Session Continuity**: Enable `continue_session` for agent memory
- **Workspace Isolation**: Agent-specific PVC naming and management
- **Error Handling**: Proper retry and failure management

### Suspend/Resume Considerations
- **Indefinite Suspend**: No timeout on suspend templates by default
- **Event Correlation**: Precise targeting through label selectors
- **State Preservation**: Workflow parameters and data persist through suspension
- **Resume Flexibility**: External events can provide data to resumed workflows

Begin implementation with the base template structure, then systematically build out each DAG component. Focus on parameter propagation and event correlation as these are critical for the multi-agent coordination system.
