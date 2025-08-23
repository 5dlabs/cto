# Toolman Guide: Design Multi-Agent Workflow DAG Structure

## Overview

This task requires comprehensive Kubernetes workflow design capabilities combined with file operations for YAML configuration management. The selected tools focus on Argo Workflows template creation, DAG structure design, and workflow validation.

## Core Tools

### Kubernetes Operations
The kubernetes server provides essential tools for Argo Workflows management:

#### `createResource`
- **Purpose**: Deploy WorkflowTemplate and test workflow instances
- **When to Use**: Creating the play-workflow-template and validating deployment
- **Example Usage**: Create WorkflowTemplate from YAML manifest
- **Best Practice**: Use `--dry-run=server` for validation before actual creation

#### `listResources`
- **Purpose**: Discover existing Argo Workflows resources and validate deployments
- **When to Use**: Checking WorkflowTemplate registration and workflow instances
- **Example Usage**: `listResources kind=WorkflowTemplate namespace=argo`
- **Best Practice**: Verify resource creation and monitor workflow status

#### `describeResource`
- **Purpose**: Examine detailed workflow configurations and status
- **When to Use**: Debugging template issues and monitoring workflow execution
- **Example Usage**: Describe specific workflow instances for troubleshooting
- **Best Practice**: Use to understand workflow state and identify issues

#### `applyResource`
- **Purpose**: Update WorkflowTemplate configurations iteratively
- **When to Use**: Refining template structure and making configuration adjustments
- **Example Usage**: Apply updated template YAML with configuration changes
- **Best Practice**: Use for iterative development and template refinement

#### `deleteResource`
- **Purpose**: Clean up test resources and failed workflow instances
- **When to Use**: Removing test workflows and cleaning development environment
- **Example Usage**: Delete specific workflow instances during testing
- **Best Practice**: Clean up resources to prevent cluster clutter

### Filesystem Operations
The filesystem server handles YAML configuration and documentation:

#### `write_file`
- **Purpose**: Create WorkflowTemplate YAML and configuration files
- **When to Use**: Writing play-workflow-template.yaml and related configurations
- **Example Usage**: Create comprehensive workflow template with DAG structure
- **Best Practice**: Follow Argo Workflows YAML conventions and include proper metadata

#### `read_file`
- **Purpose**: Examine existing workflow templates and reference materials
- **When to Use**: Studying existing templates and Argo Workflows documentation
- **Example Usage**: Read example templates to understand DAG patterns
- **Best Practice**: Use existing templates as foundation for new designs

#### `search_files`
- **Purpose**: Find existing Argo workflow configurations and examples
- **When to Use**: Locating reference templates and workflow patterns
- **Example Usage**: Search for `*.yaml` files with WorkflowTemplate resources
- **Best Practice**: Study working examples for template design patterns

#### `directory_tree`
- **Purpose**: Understand project structure for configuration organization
- **When to Use**: Mapping workflow template locations and organization
- **Example Usage**: Explore Argo configuration directories
- **Best Practice**: Follow established patterns for template organization

#### `list_directory`
- **Purpose**: Inventory existing templates and configuration files
- **When to Use**: Checking for existing workflow templates and configurations
- **Example Usage**: List contents of workflow template directories
- **Best Practice**: Ensure no naming conflicts with existing templates

## Supporting Tools

### Knowledge Management

#### `memory_create_entities`
- **Purpose**: Create knowledge graph nodes for workflow components and patterns
- **When to Use**: Recording DAG structure, template patterns, and design decisions
- **Example Usage**: Create entities for "WorkflowTemplate", "DAG Structure", "Suspend Points"
- **Best Practice**: Document complex workflow relationships and dependencies

#### `memory_add_observations`
- **Purpose**: Add detailed findings about workflow design and implementation
- **When to Use**: Recording specific template patterns and configuration details
- **Example Usage**: Add observations about parameter propagation and event correlation
- **Best Practice**: Include YAML snippets and configuration examples

### Research Tools

#### `brave_web_search`
- **Purpose**: Research Argo Workflows best practices and advanced patterns
- **When to Use**: Finding documentation on DAG design, suspend/resume, and template features
- **Example Usage**: Search for "Argo Workflows DAG suspend resume patterns"
- **Best Practice**: Validate findings against official Argo documentation

## Implementation Flow

### Phase 1: Template Foundation
1. Use `search_files` to find existing WorkflowTemplate examples
2. Use `read_file` to study current template patterns and structures
3. Create knowledge entities for workflow architecture components
4. Use `directory_tree` to understand template organization

### Phase 2: DAG Design and Structure
1. Use `write_file` to create base WorkflowTemplate with metadata
2. Design DAG structure with proper task dependencies
3. Implement parameterized agent selection system
4. Create suspend point templates for event coordination

### Phase 3: Template Development
1. Build agent-coderun template for CodeRun CRD creation
2. Implement suspend-for-event templates
3. Create task-completion template with cleanup logic
4. Add comprehensive labeling and correlation system

### Phase 4: Deployment and Validation
1. Use `createResource` with dry-run to validate template syntax
2. Deploy template to Argo namespace
3. Use `listResources` to verify template registration
4. Use `describeResource` to examine template details

### Phase 5: Testing and Refinement
1. Create test workflow instances to validate functionality
2. Monitor workflow execution and suspend behavior
3. Test parameter propagation and agent substitution
4. Refine template based on testing results

## Workflow Template Design Patterns

### Base Template Structure
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
```

### DAG Task Dependencies
```yaml
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

    - name: quality-work
      dependencies: [wait-pr-created]
      template: agent-coderun
      arguments:
        parameters:
        - name: github-app
          value: "{{workflow.parameters.quality-agent}}"
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

### Suspend Point Implementation
```yaml
- name: suspend-for-event
  inputs:
    parameters:
    - name: event-type
  suspend: {}
  metadata:
    labels:
      current-stage: "waiting-{{inputs.parameters.event-type}}"
      task-id: "{{workflow.parameters.task-id}}"
      workflow-type: play-orchestration
```

## Best Practices

### Template Design Principles
- Use parameterized configuration for all agent references
- Implement proper task dependencies and execution order
- Design indefinite suspend points for event-driven coordination
- Include comprehensive labeling for workflow correlation

### YAML Configuration Standards
- Follow Kubernetes YAML formatting conventions
- Use consistent indentation (2 spaces) throughout
- Include descriptive names and comments
- Validate syntax with dry-run before deployment

### Parameter Management
- Provide sensible default values for all parameters
- Use descriptive parameter names and documentation
- Ensure parameters propagate correctly to all templates
- Validate parameter types and constraints

### Event Correlation Design
- Implement dynamic label management for stage tracking
- Use consistent label naming conventions
- Design precise label selectors for workflow targeting
- Test correlation accuracy with various scenarios

## Testing Strategy

### Template Validation Testing
1. **Syntax Validation**: Use `argo template create --dry-run`
2. **Parameter Testing**: Validate with different agent combinations
3. **DAG Structure**: Verify task dependencies and execution order
4. **Resource Creation**: Test CodeRun CRD creation with various parameters

### Functional Testing Patterns
1. **Suspend/Resume**: Test indefinite suspend and manual resume
2. **Parameter Propagation**: Verify data flows between workflow stages
3. **Agent Integration**: Test template with Rex, Blaze, Cleo, and Tess
4. **Label Management**: Validate workflow correlation and targeting

### Integration Testing Approach
1. **End-to-End Flow**: Complete workflow execution from start to finish
2. **Event Coordination**: Test with actual GitHub webhook triggers
3. **Concurrent Workflows**: Multiple task workflows running simultaneously
4. **Error Scenarios**: Agent failures, timeout conditions, and recovery

## Common Patterns

### Multi-Agent Workflow Pattern
1. Start with parameterized template foundation
2. Design sequential DAG with proper dependencies
3. Implement suspend points for event coordination
4. Add comprehensive labeling for correlation
5. Include cleanup and progression logic

### Suspend/Resume Pattern
1. Create suspend template with indefinite duration
2. Add correlation labels for event targeting
3. Design parameter passing for resume data
4. Test manual resume and event-driven resume

### Parameter Propagation Pattern
1. Define workflow-level parameters with defaults
2. Pass parameters through all template calls
3. Use parameter substitution in resource manifests
4. Validate parameter flow with test workflows

## Troubleshooting

### Template Deployment Issues
- Verify YAML syntax and Kubernetes API versions
- Check namespace permissions and RBAC configuration
- Validate template references and dependencies
- Use dry-run testing for early validation

### DAG Structure Problems
- Verify task dependencies are correctly specified
- Check template references resolve properly
- Validate parameter passing between tasks
- Test execution order with simple workflows

### Parameter Issues
- Verify parameter defaults and types are correct
- Check parameter substitution syntax
- Test with various parameter combinations
- Validate parameter propagation to child resources

### Suspend/Resume Problems
- Verify suspend templates have proper correlation labels
- Check workflow targeting with label selectors
- Test manual resume functionality first
- Validate event correlation logic

## Notes

This task focuses on Argo Workflows template design with emphasis on:
- Multi-agent coordination through DAG structure
- Parameterized configuration for flexible agent selection
- Event-driven workflow control through suspend/resume
- Comprehensive labeling for workflow correlation
- Extended runtime support for realistic development cycles

The tool selection enables comprehensive workflow design while maintaining operational visibility and validation capabilities throughout the development process.
