# Task 3: Create Remediation Sensor and Trigger - Autonomous Implementation Prompt

## Context
You are implementing Task 3 of the Agent Remediation Loop system. This task creates the core automation component that bridges QA feedback with automated remediation by deploying an Argo Events Sensor to process feedback comments and trigger Rex remediation CodeRuns.

## Task Overview
Deploy an Argo Events Sensor that integrates with the existing play workflow infrastructure to trigger Rex remediation when Tess (the QA agent) posts feedback comments containing "🔴 Required Changes". This sensor represents the critical automation bridge between feedback detection and remediation execution.

## Dependencies
- **Task 1**: GitHub webhook infrastructure must be completed and operational
- **Task 2**: Feedback processing logic must be implemented  
- **Existing Infrastructure**: GitHub EventSource, play workflow sensors, CodeRun CRD

## Implementation Requirements

### Primary Objectives
1. **Create Argo Events Sensor** that processes PR feedback comments
2. **Extract task ID** from PR labels using JSONPath expressions
3. **Generate CodeRun resources** with REMEDIATION_MODE=true for Rex execution
4. **Pass comment ID and iteration count** via environment variables
5. **Integrate with existing sensors** for agent cancellation
6. **Implement event filtering** and deduplication logic
7. **Configure state tracking** via ConfigMaps

### Critical Technical Specifications
- **API Version**: `argoproj.io/v1alpha1` for Sensor CRD
- **Namespace**: `github-webhooks` (matches existing infrastructure)
- **EventSource Integration**: Use existing GitHub EventSource
- **CodeRun Target**: `cto.5dlabs.com/v1alpha1` API with `github_app: "5DLabs-Rex"`
- **Environment Variables**: `REMEDIATION_MODE=true`, `FEEDBACK_COMMENT_ID`, `ITERATION_COUNT`
- **Resource Limits**: 128Mi memory request, 256Mi limit, 100m-200m CPU

### Implementation Steps

#### Step 1: Create Sensor Configuration File
Create `infra/gitops/resources/github-webhooks/pr-comment-remediation-sensor.yaml` with:

- **Metadata**: Name `pr-comment-remediation`, namespace `github-webhooks`
- **ServiceAccount**: Use existing `argo-events-sa`
- **EventBus**: Connect to `default` event bus
- **Resource Limits**: Set appropriate CPU/memory constraints

#### Step 2: Configure Event Dependencies and Filters
Implement comprehensive filtering logic:

- **Event Type**: Filter for `issue_comment` events with `action: created`
- **PR Validation**: Ensure `body.issue.pull_request.url` exists (not empty)
- **Comment Pattern**: Match regex `.*🔴 Required Changes.*` in comment body
- **User Authorization**: Only allow `["5DLabs-Tess", "5DLabs-Tess[bot]"]`
- **State Validation**: Check `issue.state == 'open'` using expressions

#### Step 3: Implement JSONPath Parameter Extraction
Configure parameter extraction for CodeRun generation:

- **PR Number**: Extract from `body.issue.number`
- **Comment ID**: Extract from `body.comment.id` 
- **Task ID**: Parse from labels array using regex `^task-\\d+$` pattern
- **Task ID Template**:
  ```yaml
  dataTemplate: |
    {{- range .body.issue.labels -}}
      {{- if regexMatch "^task-\\d+$" .name -}}
        {{ regexReplaceAll "^task-" .name "" }}
      {{- end -}}
    {{- end -}}
  ```

#### Step 4: Configure CodeRun Resource Generation
Set up Kubernetes resource trigger with:

- **Resource Template**: CodeRun CRD structure
- **Naming**: Use `generateName: remediation-rex-`
- **Labels**: Include `task-id`, `trigger-type: comment-feedback`, `agent-type: rex`
- **Annotations**: Add PR number, comment ID, iteration tracking
- **Spec Fields**:
  - `github_app: "5DLabs-Rex"`
  - `remediation_mode: true`
  - `continue_session: true`
  - `pr_number` and `pr_comment_id` from parameters

#### Step 5: Configure Environment Variables
Set up Rex environment context:

```yaml
env:
  REMEDIATION_MODE: "true"
  FEEDBACK_COMMENT_ID: ""  # From comment.id parameter
  ITERATION_COUNT: ""      # From state ConfigMap lookup
  MAX_ITERATIONS: "10"
```

#### Step 6: Implement State Management Integration
Configure iteration tracking:

- **ConfigMap Naming**: `task-{id}-remediation-state`
- **State Fields**: `current_iteration`, `max_iterations`, `last_comment_id`
- **Update Logic**: Increment iteration counter on each trigger
- **Limit Checking**: Respect maximum iteration limits

#### Step 7: Add Agent Cancellation Logic
Integrate with implementation-agent-remediation sensor:

- **Cancel Existing Agents**: Delete running Cleo/Tess CodeRuns for the same task
- **Label Management**: Update PR labels to reflect remediation state
- **State Synchronization**: Coordinate with existing sensor operations

#### Step 8: Configure Error Handling
Implement robust error recovery:

- **Retry Strategy**: 3 attempts with exponential backoff
- **Fallback Actions**: Create warning events for failures
- **State Consistency**: Handle partial failures gracefully
- **Monitoring Integration**: Add prometheus scraping annotations

### Key Implementation Patterns

#### Event Filter Structure
```yaml
filters:
  data:
    - path: body.action
      type: string
      value: ["created"]
    - path: body.issue.pull_request.url
      type: string
      comparator: "!="
      value: [""]
    - path: body.comment.body
      type: string
      comparator: "~"
      value: ".*🔴 Required Changes.*"
    - path: body.comment.user.login
      type: string
      value: ["5DLabs-Tess", "5DLabs-Tess[bot]"]
  exprs:
    - expr: "issue.state == 'open'"
      fields:
        - name: issue.state
          path: body.issue.state
```

#### Resource Generation Template
```yaml
source:
  resource:
    apiVersion: cto.5dlabs.com/v1alpha1
    kind: CodeRun
    metadata:
      generateName: remediation-rex-
      namespace: agent-platform
      labels:
        task-id: ""  # From parameter
        trigger-type: "comment-feedback"
        agent-type: "rex"
    spec:
      github_app: "5DLabs-Rex"
      remediation_mode: true
      continue_session: true
      env:
        REMEDIATION_MODE: "true"
        FEEDBACK_COMMENT_ID: ""
        ITERATION_COUNT: ""
        MAX_ITERATIONS: "10"
```

### Testing and Validation

#### Required Tests
1. **Event Processing**: Create test PR with feedback comment
2. **Parameter Extraction**: Verify task ID extraction from labels
3. **CodeRun Creation**: Confirm resource generation with correct parameters
4. **Integration**: Test cancellation of existing quality agents
5. **State Management**: Verify ConfigMap updates and iteration tracking
6. **Error Handling**: Test with malformed events and missing data

#### Validation Commands
```bash
# Deploy sensor
kubectl apply -f infra/gitops/resources/github-webhooks/pr-comment-remediation-sensor.yaml

# Verify deployment
kubectl get sensor pr-comment-remediation -n github-webhooks
kubectl describe sensor pr-comment-remediation -n github-webhooks

# Check logs
kubectl logs -l app=remediation-sensor -n github-webhooks -f

# Test with sample event
kubectl create -f test/sample-feedback-event.yaml

# Verify CodeRun creation
kubectl get coderuns -l trigger-type=comment-feedback -n agent-platform
```

## Success Criteria
- Sensor successfully deployed and running in github-webhooks namespace
- PR comments with '🔴 Required Changes' trigger sensor activation  
- CodeRun resources created with REMEDIATION_MODE=true
- Task ID correctly extracted from PR labels
- Only authorized users trigger remediation
- Integration with existing cancellation sensor works
- State tracking via ConfigMaps functions properly
- Event deduplication prevents duplicate processing
- Resource usage stays within defined limits
- End-to-end remediation flow completes successfully

## Common Issues and Solutions

### Event Not Triggering
- Verify EventSource is receiving webhooks
- Check filter expressions match payload structure  
- Confirm user authorization in filters
- Validate regex patterns for comment matching

### Parameter Extraction Failures
- Test JSONPath expressions with real webhook data
- Check for missing fields in GitHub payload
- Validate dataTemplate syntax and escaping
- Ensure label format matches task-{id} pattern

### CodeRun Creation Issues  
- Verify RBAC permissions for resource creation
- Check target namespace exists and is accessible
- Confirm CodeRun CRD is installed and accessible
- Validate resource limits don't exceed quotas

### State Management Problems
- Check ConfigMap creation permissions
- Verify namespace access for state storage
- Ensure iteration logic handles edge cases
- Test maximum iteration limit enforcement

## Architecture Integration
This sensor integrates with:
- **GitHub EventSource**: Receives webhook events
- **Play Workflow Sensors**: Coordinates with existing automation
- **Implementation-Agent-Remediation**: Handles agent cancellation
- **ConfigMap State Store**: Tracks remediation iterations
- **CodeRun Controller**: Executes Rex remediation

Remember to follow existing patterns from play-workflow-sensors.yaml and maintain consistency with the established infrastructure while adding remediation-specific functionality.

## Final Notes
This implementation creates the core automation bridge that enables self-healing development workflows. The sensor must be robust, handle edge cases gracefully, and integrate seamlessly with existing infrastructure to ensure reliable operation of the entire remediation loop system.