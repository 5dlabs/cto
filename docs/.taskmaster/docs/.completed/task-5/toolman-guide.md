# Toolman Guide: Create GitHub Webhook Correlation Logic



## Overview

This task requires Kubernetes resource management for Argo Events Sensors and file operations for configuration. The selected tools focus on creating, configuring, and testing webhook correlation mechanisms.



## Core Tools

### Kubernetes Operations

#### `kubernetes_createResource`
- **Purpose**: Create Argo Events Sensor resources
- **When to Use**: Deploying new Sensors for webhook correlation
- **Example Usage**: Creating multi-agent workflow resume sensor
- **Best Practice**: Validate YAML before creation, use dry-run first

#### `kubernetes_listResources`
- **Purpose**: List existing Sensors and workflows
- **When to Use**: Checking deployed Sensors, finding suspended workflows
- **Example Usage**: `kubectl get sensors -n argo`
- **Best Practice**: Filter by labels for specific components

#### `kubernetes_getResource`
- **Purpose**: Retrieve detailed Sensor configurations
- **When to Use**: Debugging correlation issues, verifying configuration
- **Example Usage**: Getting Sensor logs and status
- **Best Practice**: Use `-o yaml` for full configuration details

### File Operations



#### `read_file`
- **Purpose**: Read existing Sensor configurations and templates
- **When to Use**: Reviewing reference implementations like github-demo-sensor.yaml
- **Example Usage**: Reading webhook payload samples for testing
- **Best Practice**: Check multiple examples for patterns



#### `write_file`
- **Purpose**: Create Sensor YAML configurations
- **When to Use**: Generating new Sensor definitions
- **Example Usage**: Writing correlation logic configurations
- **Best Practice**: Use proper YAML formatting and validation



#### `edit_file`
- **Purpose**: Modify existing Sensor configurations
- **When to Use**: Updating JQ expressions, fixing correlation logic
- **Example Usage**: Adjusting label selectors for workflow targeting
- **Best Practice**: Test changes incrementally



#### `search_files`
- **Purpose**: Find related configuration files
- **When to Use**: Locating sensor templates, webhook examples
- **Example Usage**: Finding all files with "sensor" in name
- **Best Practice**: Search in argo namespace directories



### Research Tools



#### `brave_web_search`
- **Purpose**: Research JQ expressions and Argo Events features
- **When to Use**: Finding documentation for v1.9+ parameterization
- **Example Usage**: Searching "Argo Events webhook correlation examples"
- **Best Practice**: Verify against official documentation

## Implementation Flow

### Phase 1: Analysis


1. Use `kubernetes_listResources` to find existing Sensors


2. Use `kubernetes_getResource` to examine github-demo-sensor


3. Use `read_file` to review EventSource configuration


4. Document webhook payload structure

### Phase 2: JQ Expression Development


1. Create test webhook payloads
2. Develop JQ extraction expressions:
   ```jq
   .pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]





```


3. Test with sample data


4. Implement fallback logic for branch names

### Phase 3: Sensor Creation
1. Use `write_file` to create Sensor YAML:


   - Multi-agent workflow resume sensor


   - Ready-for-QA label sensor


   - PR approval sensor


   - Rex remediation sensor


2. Configure dependencies and triggers


3. Add correlation logic with label selectors

### Phase 4: Deployment


1. Use `kubernetes_createResource` to deploy Sensors


2. Verify with `kubernetes_listResources`


3. Check logs with `kubernetes_getResource`


4. Test correlation with suspended workflows

### Phase 5: Testing


1. Create test workflows with proper labels


2. Trigger webhook events


3. Monitor Sensor processing


4. Verify workflow resumption



## Best Practices

### JQ Expression Design


- Start simple, add complexity gradually


- Always include error handling


- Test with malformed input


- Document expression purpose



### Label Selector Strategy



```yaml
labelSelector: |
  workflow-type=play-orchestration,
  task-id={{extracted-task-id}},
  current-stage={{target-stage}}






```


- Use multiple labels for precision


- Avoid overly broad selectors


- Include workflow type discrimination

### Event Filtering


- Filter by action (opened, labeled, etc.)


- Validate sender for remediation events


- Check event state for reviews


- Handle duplicate events

## Common Patterns

### Webhook Payload Extraction



```yaml
- name: extract-task-id
  inline:
    script: |
      echo '{{inputs.body}}' | jq -r '
        .pull_request.labels[] |
        select(.name | startswith("task-")) |
        .name | split("-")[1]
      ' | head -1






```

### Workflow Targeting



```yaml
triggers:
- template:
    name: resume-workflow
    argoWorkflow:
      operation: resume
      source:
        resource:
          selector:
            matchLabels:
              workflow-type: play-orchestration
              task-id: "{{steps.extract-task-id.outputs.result}}"






```

## Troubleshooting

### Missing Correlations


- Check JQ expression output


- Verify label selector syntax


- Ensure workflows have correct labels


- Review Sensor logs for errors



### False Positives


- Tighten label selectors


- Add more discriminating fields


- Validate extracted IDs


- Check for duplicate events

### Performance Issues


- Optimize JQ expressions


- Add resource limits to Sensors


- Check EventBus capacity


- Monitor webhook backlog



## Notes

This task establishes the critical correlation mechanism for the entire multi-agent orchestration system. Key considerations:

- **Precision**: Must accurately target specific workflows
- **Reliability**: Handle edge cases and malformed data
- **Performance**: Process webhooks quickly
- **Debugging**: Include comprehensive logging

The correlation logic is the backbone of event-driven orchestration. Test thoroughly with various webhook payloads and workflow states.
