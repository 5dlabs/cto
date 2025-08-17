# Autonomous Agent Prompt: Setup Argo Events Infrastructure

## Mission

You are tasked with creating and configuring specialized Argo Events Sensors for multi-agent workflow orchestration. Your mission is to enable seamless event-driven coordination between Rex, Cleo, and Tess agents through robust GitHub webhook processing and workflow resumption mechanisms.

## Context

The multi-agent orchestration system requires sophisticated event handling to coordinate sequential agent execution. The existing Argo Events infrastructure (EventBus, EventSource) is functional, but specialized Sensors are needed to handle the complex state transitions and correlation logic required for the play workflow.

## Objectives

1. **Create Multi-Agent Workflow Resume Sensor**
   - Handle PR creation events from GitHub
   - Extract task IDs from PR labels and branch names
   - Resume suspended workflows after Rex completes implementation
   - Implement multi-method validation to prevent false positives

2. **Build Ready-for-QA Label Sensor**
   - Detect when Cleo adds "ready-for-qa" label to PRs
   - Verify the label was added by 5DLabs-Cleo[bot]
   - Resume workflows to trigger Tess testing phase
   - Ensure proper workflow stage targeting

3. **Implement PR Approval Sensor**
   - Process PR review approval events
   - Confirm approval comes from 5DLabs-Tess[bot]
   - Extract task correlation data from PR metadata
   - Resume workflow completion and task progression

4. **Develop Rex Remediation Sensor**
   - Detect push events from 5DLabs-Rex[bot] to task branches
   - Implement immediate cancellation of running Cleo/Tess work
   - Reset workflow state and remove stale labels
   - Restart QA pipeline with fresh code changes

## Technical Requirements

### Infrastructure Integration
- **EventSource**: Use existing `github` EventSource for webhook processing
- **EventBus**: Connect to existing `argo` EventBus for event distribution
- **Namespace**: Deploy all sensors in `argo` namespace
- **RBAC**: Ensure sensors have permissions for workflow resume operations

### Event Correlation Logic
```yaml
# Standard task ID extraction pattern
- src:
    dependencyName: github-event
    dataTemplate: |
      {{jq '.pull_request.labels[?(@.name | startswith("task-"))].name | split("-")[1]'}}
  dest: spec.arguments.parameters.task-id

# Workflow targeting with label selectors
labelSelector: |
  workflow-type=play-orchestration,
  task-id={{task-id}},
  current-stage={{target-stage}}
```

### Webhook Field Processing
Each sensor must extract and validate:
- **Task ID**: From PR labels (`task-3` → extracts "3")
- **Branch Validation**: Ensure branch name matches (`task-3-*`)
- **Actor Verification**: Confirm events come from expected GitHub Apps
- **Action Filtering**: Process only relevant webhook actions

### Quality Assurance Patterns
- **Multi-Method Validation**: PR labels AND branch names must match
- **Actor Verification**: Ensure events come from correct GitHub Apps
- **Idempotent Operations**: Handle duplicate webhook deliveries gracefully
- **Error Recovery**: Implement proper logging and alerting for failures

## Implementation Strategy

### Phase 1: Basic Sensor Creation
1. Start with multi-agent workflow resume sensor
2. Test with simple suspended workflow
3. Validate task ID extraction and correlation
4. Confirm workflow resumption works correctly

### Phase 2: Label-Based Progression
1. Implement ready-for-QA label sensor
2. Test Cleo → Tess handoff mechanism
3. Validate label actor verification
4. Ensure proper stage targeting

### Phase 3: Approval Handling
1. Create PR approval sensor
2. Test Tess approval processing
3. Validate workflow completion flow
4. Implement task progression logic

### Phase 4: Remediation System
1. Build Rex remediation sensor
2. Test push event detection
3. Implement CodeRun cancellation
4. Validate QA pipeline restart

## Sensor Configuration Templates

### Multi-Agent Resume Sensor (Supported Pattern)
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: multi-agent-workflow-resume
  namespace: argo
spec:
  dependencies:
  - name: github-pr-created
    eventSourceName: github
    eventName: pull_request
    filters:
      data:
      - path: action
        type: string
        value: "opened"
  
  triggers:
  - template:
      name: resume-after-implementation
      argoWorkflow:
        operation: resume
        args: []
        parameters:
        - src:
            dependencyName: github-pr-created
            dataTemplate: |
              play-task-{{ range $i, $l := .Input.body.pull_request.labels }}{{ if hasPrefix "task-" $l.name }}{{ $p := splitList "-" $l.name }}{{ if gt (len $p) 1 }}{{ index $p 1 }}{{ end }}{{ end }}{{ end }}-workflow
          dest: args.0
```

### Ready-for-QA Label Sensor
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: ready-for-qa-sensor
  namespace: argo
spec:
  dependencies:
  - name: github-pr-labeled
    eventSourceName: github
    eventName: pull_request
    filters:
      data:
      - path: action
        type: string
        value: "labeled"
      - path: label.name
        type: string
        value: "ready-for-qa"
```

### Implementation Agent Remediation Sensor (Cleanup Workflow)
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: implementation-agent-remediation
  namespace: argo
spec:
  dependencies:
  - name: implementation-push-event
    eventSourceName: github
    eventName: push
    filters:
      data:
      - path: headers.X-Github-Event
        type: string
        value: ["push"]
      - path: body.sender.login
        type: string
        comparator: "~"
        value: "5DLabs-(Rex|Blaze|Morgan)\\[bot\\]|5DLabs-(Rex|Blaze|Morgan)"
      - path: ref
        type: string
        comparator: "~"
        value: "refs/heads/task-.*"
        
  triggers:
  - template:
      name: create-cleanup-workflow
      argoWorkflow:
        operation: submit
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: cleanup-quality-agents-
              namespace: argo
            spec:
              entrypoint: cleanup-quality-agents
              arguments:
                parameters:
                  - name: task-id
                    value: ""
                  - name: pushing-agent
                    value: ""
              templates:
                - name: cleanup-quality-agents
                  inputs:
                    parameters:
                      - name: task-id
                      - name: pushing-agent
                  script:
                    image: bitnami/kubectl:latest
                    command: [sh]
                    source: |
                      #!/bin/sh
                      echo "Implementation agent {{inputs.parameters.pushing-agent}} pushed to task-{{inputs.parameters.task-id}}"
                      echo "Cancelling quality/testing agent CodeRuns for task-{{inputs.parameters.task-id}}"

                      kubectl delete coderun -n argo \
                        -l task-id={{inputs.parameters.task-id}} \
                        -l github-app!=5DLabs-Rex \
                        -l github-app!=5DLabs-Blaze \
                        -l github-app!=5DLabs-Morgan \
                        || true
                      echo "QA pipeline cleanup complete for task-{{inputs.parameters.task-id}}"
```

## Testing and Validation

### Functional Testing
1. **Deploy and Verify**: Use `kubectl get sensors -n argo` to confirm deployment
2. **Log Monitoring**: Watch sensor logs with `kubectl logs -f sensor-pod -n argo`
3. **Webhook Testing**: Trigger actual GitHub events to test processing
4. **Correlation Validation**: Verify task ID extraction and workflow targeting

### Integration Testing
1. **End-to-End Flow**: Test complete Rex → Cleo → Tess pipeline
2. **Remediation Testing**: Trigger Rex pushes and validate cancellation
3. **Error Scenarios**: Test correlation failures and recovery
4. **Rate Limiting**: Monitor for GitHub API rate limit issues

### Operational Validation
1. **Performance Monitoring**: Track webhook processing latency
2. **Resource Usage**: Monitor sensor pod resource consumption
3. **Error Rates**: Implement alerting for correlation failures
4. **Scalability Testing**: Verify handling of concurrent workflows

## Success Criteria

- All four sensors deployed and operational in Argo namespace
- Successful correlation of GitHub events with suspended workflows
- Reliable workflow resumption at appropriate stages
- Robust Rex remediation with proper cancellation logic
- Comprehensive testing validates all event handling scenarios
- No disruption to existing Argo Events infrastructure

## Key Implementation Notes

### GitHub Webhook Considerations
- **Rate Limiting**: GitHub may rate limit webhook deliveries
- **Payload Validation**: Verify webhook signatures for security
- **Duplicate Handling**: Process duplicate deliveries idempotently
- **Field Extraction**: Use robust jq expressions for JSON processing

### Workflow Correlation Challenges
- **Label Consistency**: Ensure agents properly label PRs with task IDs
- **Branch Naming**: Validate branch names match task correlation
- **Stage Management**: Workflows must update stage labels correctly
- **Timing Issues**: Handle race conditions in event processing

### Operational Requirements
- **Monitoring**: Implement comprehensive logging and metrics
- **Alerting**: Set up alerts for sensor failures and correlation errors
- **Recovery**: Design sensors to recover gracefully from failures
- **Scalability**: Ensure sensors can handle multiple concurrent tasks

Begin implementation systematically, starting with the multi-agent workflow resume sensor and progressing through each specialized sensor. Validate each component thoroughly before proceeding to the next phase.