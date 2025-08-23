# Toolman Guide: Setup Argo Events Infrastructure

## Overview

This task requires Kubernetes infrastructure management capabilities combined with filesystem operations for YAML configuration and documentation. The selected tools focus on Kubernetes resource management and file operations for creating Argo Events Sensors.

## Core Tools

### Kubernetes Operations
The kubernetes server provides essential tools for managing Argo Events infrastructure:

#### `listResources`
- **Purpose**: Discover existing Argo Events resources and validate infrastructure
- **When to Use**: Understanding current EventSource, EventBus, and Sensor configurations
- **Example Usage**: List sensors with `listResources kind=Sensor namespace=argo`
- **Best Practice**: Start by inventorying existing resources before creating new ones

#### `describeResource`
- **Purpose**: Examine existing Sensor configurations for patterns and validation
- **When to Use**: Analyzing `github-demo-sensor.yaml` structure and EventSource integration
- **Example Usage**: Describe existing sensors to understand configuration patterns
- **Best Practice**: Study working examples before creating new configurations

#### `createResource`
- **Purpose**: Deploy new Sensor configurations to Kubernetes cluster
- **When to Use**: Creating the four specialized sensors for multi-agent orchestration
- **Example Usage**: Create sensors from YAML manifests
- **Best Practice**: Use `--dry-run=server` for validation before actual creation

#### `deleteResource`
- **Purpose**: Remove test resources or cleanup during development
- **When to Use**: Cleaning up test sensors or correcting misconfigurations
- **Example Usage**: Delete specific sensors during testing iterations
- **Best Practice**: Be cautious with deletion; always verify resource before deletion

#### `applyResource`
- **Purpose**: Update existing sensor configurations or create new ones idempotently
- **When to Use**: Iterative development and configuration refinement
- **Example Usage**: Apply sensor YAML files with configuration updates
- **Best Practice**: Use apply for iterative development rather than create/delete cycles

### Filesystem Operations
The filesystem server handles configuration file management:

#### `write_file`
- **Purpose**: Create YAML configuration files for Argo Events Sensors
- **When to Use**: Writing sensor manifests and configuration files
- **Example Usage**: Create `multi-agent-workflow-resume-sensor.yaml`
- **Best Practice**: Follow Kubernetes YAML conventions and include proper metadata

#### `read_file`
- **Purpose**: Read existing sensor configurations and reference materials
- **When to Use**: Examining `github-demo-sensor.yaml` patterns and templates
- **Example Usage**: Read reference configurations to understand patterns
- **Best Practice**: Use as templates for new sensor configurations

#### `search_files`
- **Purpose**: Find existing Argo Events configurations and examples
- **When to Use**: Locating reference configurations and documentation
- **Example Usage**: Search for `*.yaml` files containing EventSource patterns
- **Best Practice**: Find working examples to use as configuration templates

#### `directory_tree`
- **Purpose**: Understand project structure for configuration placement
- **When to Use**: Understanding where to place sensor configurations
- **Example Usage**: Map infrastructure directories for proper file organization
- **Best Practice**: Follow established patterns for configuration organization

#### `list_directory`
- **Purpose**: Inventory existing configurations and validate completeness
- **When to Use**: Checking for existing sensors and configuration files
- **Example Usage**: List contents of infrastructure directories
- **Best Practice**: Verify all required configurations are present

## Supporting Tools

### Knowledge Management

#### `memory_create_entities`
- **Purpose**: Create knowledge graph nodes for Argo Events components
- **When to Use**: Recording discovered infrastructure components and relationships
- **Example Usage**: Create entities for "EventBus", "EventSource", "Workflow Resume Sensor"
- **Best Practice**: Document component relationships for system understanding

#### `memory_add_observations`
- **Purpose**: Add detailed findings about sensor configurations and behaviors
- **When to Use**: Recording specific configuration patterns and webhook correlations
- **Example Usage**: Add observations about task ID extraction patterns
- **Best Practice**: Include YAML snippets and configuration examples in observations

### Research Tools

#### `brave_web_search`
- **Purpose**: Find Argo Events documentation and configuration examples
- **When to Use**: Researching webhook field extraction, jq expressions, label selectors
- **Example Usage**: Search for "Argo Events sensor GitHub webhook correlation"
- **Best Practice**: Validate findings against actual deployed infrastructure

## Implementation Flow

### Phase 1: Infrastructure Discovery
1. Use `listResources` to inventory existing Argo Events infrastructure:
   - EventBus resources in `argo` namespace
   - EventSource configurations for GitHub webhooks
   - Existing sensors for pattern reference
2. Use `describeResource` to examine `github-demo-sensor.yaml` structure
3. Create knowledge entities for discovered infrastructure components

### Phase 2: Configuration Development
1. Use `read_file` to study reference sensor configurations
2. Use `write_file` to create the four specialized sensor YAML files:
   - `multi-agent-workflow-resume-sensor.yaml`
   - `ready-for-qa-label-sensor.yaml`
   - `pr-approval-sensor.yaml`
   - `rex-remediation-sensor.yaml`
3. Implement proper webhook field extraction and correlation logic

### Phase 3: Deployment and Testing
1. Use `createResource` with `--dry-run=server` to validate configurations
2. Use `applyResource` to deploy sensors to the cluster
3. Use `listResources` to verify successful sensor creation
4. Use `describeResource` to inspect deployed sensor status and configuration

### Phase 4: Validation and Monitoring
1. Monitor sensor logs using Kubernetes tools
2. Test webhook processing with actual GitHub events
3. Validate workflow correlation and resumption logic
4. Document findings in knowledge graph

## Sensor Configuration Patterns

### Standard Sensor Structure
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: sensor-name
  namespace: argo
spec:
  dependencies:
  - name: dependency-name
    eventSourceName: github
    eventName: webhook-type
    filters:
      data:
      - path: field.path
        type: string
        value: "expected-value"

  triggers:
  - template:
      name: trigger-name
      argoWorkflow:
        operation: resume
        source:
          resource:
            labelSelector: "key=value"
```

### Webhook Field Extraction
```yaml
# Task ID extraction from PR labels
- src:
    dependencyName: github-event
    dataTemplate: |
      {{jq '.pull_request.labels[?(@.name | startswith("task-"))].name | split("-")[1]'}}
  dest: spec.arguments.parameters.task-id

# Branch validation
- src:
    dependencyName: github-event
    dataTemplate: |
      {{jq '.pull_request.head.ref | capture("^task-(?<id>[0-9]+)-.*").id'}}
  dest: spec.arguments.parameters.branch-task-id
```

### Workflow Targeting
```yaml
# Precise workflow label selector
labelSelector: |
  workflow-type=play-orchestration,
  task-id={{task-id}},
  current-stage={{target-stage}}

# Examples of stage targeting:
# current-stage=waiting-pr-created     (after Rex)
# current-stage=waiting-ready-for-qa   (after Cleo)
# current-stage=waiting-pr-approved    (after Tess)
```

## Best Practices

### Configuration Management
- Use consistent naming conventions for all sensors
- Include comprehensive metadata and labels
- Document webhook field paths and extraction logic
- Implement robust error handling and logging

### Testing Strategy
- Start with dry-run validation before deployment
- Test each sensor individually before integration
- Use actual GitHub events for validation
- Monitor logs for correlation and processing errors

### Operational Considerations
- Implement proper resource limits and requests
- Configure appropriate RBAC permissions
- Set up monitoring and alerting for sensor health
- Document troubleshooting procedures

### Error Handling
- Handle missing webhook fields gracefully
- Implement proper logging for correlation failures
- Set up alerts for sensor processing errors
- Design idempotent operations for duplicate events

## Common Patterns

### Multi-Agent Sensor Pattern
1. Start with EventSource integration
2. Define webhook event filtering
3. Implement task ID extraction
4. Configure workflow label selector
5. Set up trigger action (resume/delete)

### Remediation Sensor Pattern
1. Detect Rex push events
2. Extract task ID from branch name
3. Cancel running CodeRun CRDs
4. Reset workflow state
5. Remove stale labels

## Troubleshooting

### Sensor Deployment Issues
- Verify namespace and RBAC permissions
- Check EventSource and EventBus connectivity
- Validate YAML syntax and structure
- Examine sensor pod logs for startup errors

### Webhook Processing Problems
- Test webhook field extraction with actual payloads
- Validate jq expressions in isolation
- Check EventSource webhook delivery
- Monitor rate limiting and GitHub API issues

### Correlation Failures
- Verify PR labels and branch naming consistency
- Check workflow label selectors match
- Validate task ID extraction logic
- Test multi-method validation requirements

## Notes

This task focuses on Kubernetes infrastructure management with emphasis on:
- Argo Events Sensor creation and deployment
- GitHub webhook processing and correlation
- Workflow state management and resumption
- Multi-agent coordination through event-driven architecture

The tool selection enables comprehensive infrastructure operations while maintaining operational visibility through knowledge management and research capabilities.
