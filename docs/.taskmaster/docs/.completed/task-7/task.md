# Task 7: Implement Workflow Stage Transitions



## Overview

Create workflow template logic to update current-stage labels and handle transitions between agent phases. This enables event-driven coordination by maintaining workflow state through label updates that trigger subsequent agent stages.

Scope update: The project already sets stage labels at suspend points and has working correlation sensors (from task 5). Enhance the existing workflow by adding explicit post-agent stage transition steps and atomic label update mechanisms (JSON merge patch + verification) after Rex, Cleo, and Tess complete. Do not rework the existing correlation sensors; integrate with them.

## Technical Context

Multi-agent orchestration requires atomic state management to coordinate agent handoffs. The workflow must track its current stage and update labels atomically to prevent race conditions while enabling event-driven progression through agent phases.

## Implementation Guide

### Phase 1: Design Label Transition System



1. **Define Stage Progression Flow**
   ```yaml
   # Workflow stage progression
   waiting-pr-created → waiting-ready-for-qa → waiting-pr-approved → completed





```



2. **Label Update Mechanism**


   - Use Kubernetes resource template for atomic label updates


   - Implement via kubectl patch commands with JSON merge strategy


   - Ensure idempotent operations to prevent duplicate updates


   - Add workflow correlation metadata for event targeting

### Phase 2: Implement Argo Workflows Label Updates



1. **Create Label Update Template**
   ```yaml
   # In workflow template - add after each agent stage
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
             task-id: "{{workflow.parameters.task-id}}"
             repository: "{{workflow.parameters.repository}}"
         spec: {}





```



2. **Implement Stage Transition Logic**
   ```yaml
   # Stage transition after Rex completes
   - name: transition-to-cleo-stage
     dependencies: [rex-implementation]
     template: update-workflow-stage
     arguments:
       parameters:
       - name: new-stage
         value: "waiting-pr-created"





```

### Phase 3: Create Atomic Label Update Operations



1. **Kubernetes API Patch Operations**
   ```bash
   #!/bin/bash
   # Atomic label update script for workflow stages

   WORKFLOW_NAME="{{workflow.name}}"
   NEW_STAGE="{{inputs.parameters.new-stage}}"
   TASK_ID="{{workflow.parameters.task-id}}"

   # Perform atomic label update
   kubectl patch workflow "$WORKFLOW_NAME" \


     --type='merge' \
     --patch="{\"metadata\":{\"labels\":{\"current-stage\":\"$NEW_STAGE\"}}}"

   # Verify update succeeded
   CURRENT_LABEL=$(kubectl get workflow "$WORKFLOW_NAME" -o jsonpath='{.metadata.labels.current-stage}')

   if [ "$CURRENT_LABEL" = "$NEW_STAGE" ]; then
     echo "✅ Stage transition successful: $NEW_STAGE"
   else
     echo "❌ Stage transition failed"
     exit 1
   fi





```



2. **Race Condition Prevention**
   ```yaml
   # Use resource versioning for concurrent safety
   - name: safe-label-update
     resource:
       action: patch
       source:
         resource:
           apiVersion: argoproj.io/v1alpha1
           kind: Workflow
           metadata:
             name: "{{workflow.name}}"
             resourceVersion: "{{workflow.metadata.resourceVersion}}"
       patch: |
         [
           {
             "op": "replace",
             "path": "/metadata/labels/current-stage",
             "value": "{{inputs.parameters.new-stage}}"
           }
         ]
       patchType: "application/json-patch+json"





```

### Phase 4: Integrate with Argo Events Sensors



1. **Stage-Aware Event Correlation**
   ```yaml
   # Argo Events sensor with stage targeting
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: github-workflow-progression
   spec:
     dependencies:
     - name: github-pr-created
       eventSourceName: github-webhook
       eventName: pull-request-opened

     triggers:
     - template:
         name: resume-after-pr-created
         argoWorkflow:
           operation: resume
           source:
             resource:
               # Target specific workflow at specific stage
               labelSelector: |
                 workflow-type=play-orchestration,
                 task-id={{extracted-task-id}},
                 current-stage=waiting-pr-created





```



2. **Multi-Stage Event Handling**
   ```yaml
   # Different sensors for different stages
   triggers:
   - template:
       name: resume-after-cleo-complete
       conditions: "github-pr-labeled"
       argoWorkflow:
         operation: resume
         source:
           resource:
             labelSelector: |
               current-stage=waiting-ready-for-qa,
               task-id={{extracted-task-id}}





```

### Phase 5: Implement Workflow Stage Suspend/Resume Pattern



1. **Suspend Points with Stage Updates**
   ```yaml
   # Complete workflow template with stage transitions
   templates:
   - name: main
     dag:
       tasks:
       - name: rex-implementation
         template: agent-coderun
         arguments:
           parameters:
           - name: github-app
             value: "5DLabs-Rex"

       - name: update-to-waiting-pr
         dependencies: [rex-implementation]
         template: update-workflow-stage
         arguments:
           parameters:
           - name: new-stage
             value: "waiting-pr-created"

       - name: wait-pr-created
         dependencies: [update-to-waiting-pr]
         template: suspend-for-webhook

       - name: cleo-quality
         dependencies: [wait-pr-created]
         template: agent-coderun
         arguments:
           parameters:
           - name: github-app
             value: "5DLabs-Cleo"

       - name: update-to-waiting-qa
         dependencies: [cleo-quality]
         template: update-workflow-stage
         arguments:
           parameters:
           - name: new-stage
             value: "waiting-ready-for-qa"





```



2. **Dynamic Label Management Template**
   ```yaml
   - name: update-workflow-stage
     inputs:
       parameters:
       - name: new-stage
       - name: previous-stage
         value: "{{workflow.labels.current-stage}}"
     script:
       image: bitnami/kubectl:latest
       command: [bash]
       source: |
         set -e

         WORKFLOW_NAME="{{workflow.name}}"
         NEW_STAGE="{{inputs.parameters.new-stage}}"
         PREVIOUS_STAGE="{{inputs.parameters.previous-stage}}"

         echo "Transitioning from $PREVIOUS_STAGE to $NEW_STAGE"

         # Atomic update with verification
         kubectl patch workflow "$WORKFLOW_NAME" \


           --type='merge' \


           --patch="{
             \"metadata\": {
               \"labels\": {
                 \"current-stage\": \"$NEW_STAGE\",
                 \"previous-stage\": \"$PREVIOUS_STAGE\",
                 \"updated-at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
               }
             }
           }"

         # Verify transition succeeded
         ACTUAL_STAGE=$(kubectl get workflow "$WORKFLOW_NAME" \


           -o jsonpath='{.metadata.labels.current-stage}')

         if [ "$ACTUAL_STAGE" != "$NEW_STAGE" ]; then
           echo "ERROR: Stage transition failed!"
           exit 1
         fi

         echo "✅ Successfully transitioned to: $NEW_STAGE"





```



## Code Examples

### Complete Stage Transition Workflow



```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: play-workflow-with-stages
spec:
  entrypoint: main
  activeDeadlineSeconds: 1209600  # 14 days
  arguments:
    parameters:
    - name: task-id
    - name: repository
      value: "5dlabs-cto"

  templates:
  - name: main
    dag:
      tasks:
      # Stage 1: Rex Implementation
      - name: rex-implementation
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Rex"

      - name: stage-1-complete
        dependencies: [rex-implementation]
        template: update-workflow-stage
        arguments:
          parameters:
          - name: new-stage
            value: "waiting-pr-created"

      # Suspend for PR creation event
      - name: wait-pr-created
        dependencies: [stage-1-complete]
        template: suspend-for-webhook

      # Stage 2: Cleo Quality
      - name: cleo-quality
        dependencies: [wait-pr-created]
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Cleo"

      - name: stage-2-complete
        dependencies: [cleo-quality]
        template: update-workflow-stage
        arguments:
          parameters:
          - name: new-stage
            value: "waiting-ready-for-qa"

      # Suspend for ready-for-qa label event
      - name: wait-ready-for-qa
        dependencies: [stage-2-complete]
        template: suspend-for-webhook

      # Stage 3: Tess Testing
      - name: tess-testing
        dependencies: [wait-ready-for-qa]
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Tess"

      - name: stage-3-complete
        dependencies: [tess-testing]
        template: update-workflow-stage
        arguments:
          parameters:
          - name: new-stage
            value: "waiting-pr-approved"

      # Suspend for PR approval event
      - name: wait-pr-approved
        dependencies: [stage-3-complete]
        template: suspend-for-webhook

      # Task Completion
      - name: complete-task
        dependencies: [wait-pr-approved]
        template: mark-task-complete

  # Reusable templates
  - name: suspend-for-webhook
    suspend: {}

  - name: update-workflow-stage
    inputs:
      parameters:
      - name: new-stage
    script:
      image: bitnami/kubectl:latest
      command: [bash]
      source: |
        # Atomic label update implementation
        kubectl patch workflow "{{workflow.name}}" \


          --type='merge' \
          --patch='{"metadata":{"labels":{"current-stage":"{{inputs.parameters.new-stage}}"}}}'






```

### Event Sensor Integration



```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: workflow-stage-progression
spec:
  dependencies:
  - name: github-pr-events
    eventSourceName: github-webhook
    eventName: pull-request

  triggers:
  # Resume workflow at waiting-pr-created stage
  - template:
      name: resume-waiting-pr-created
      conditions: "github-pr-events && github-pr-events.action == 'opened'"
      argoWorkflow:
        operation: resume
        source:
          resource:
            labelSelector: |
              workflow-type=play-orchestration,
              current-stage=waiting-pr-created,
              task-id={{task-id-from-pr-labels}}

  # Resume workflow at waiting-ready-for-qa stage
  - template:
      name: resume-waiting-ready-for-qa
      conditions: |
        github-pr-events &&
        github-pr-events.action == 'labeled' &&
        github-pr-events.label.name == 'ready-for-qa'
      argoWorkflow:
        operation: resume
        source:
          resource:
            labelSelector: |
              workflow-type=play-orchestration,
              current-stage=waiting-ready-for-qa,
              task-id={{task-id-from-pr-labels}}






```

## Architecture Patterns

### State Machine Implementation
The workflow implements a finite state machine with these characteristics:
- **States**: Defined by `current-stage` label values
- **Transitions**: Atomic label updates between stages
- **Events**: GitHub webhooks trigger state transitions
- **Guards**: Label selectors ensure correct workflow targeting

### Event-Driven Coordination






```
Rex Complete → update-stage → suspend → GitHub Event → resume → Cleo Start
Cleo Complete → update-stage → suspend → GitHub Event → resume → Tess Start
Tess Complete → update-stage → suspend → GitHub Event → resume → Task Complete






```

### Idempotent Operations
All label updates are idempotent to handle:


- Duplicate webhook events


- Workflow restarts


- Kubernetes API retries


- Concurrent update attempts

## Testing Strategy

### Label Update Testing


1. **Atomic Update Verification**
   ```bash
   # Test label updates don't create race conditions
   for i in {1..10}; do
     kubectl patch workflow test-workflow \


       --type='merge' \
       --patch='{"metadata":{"labels":{"current-stage":"test-'$i'"}}}' &
   done
   wait

   # Verify only one final state
   kubectl get workflow test-workflow -o jsonpath='{.metadata.labels.current-stage}'





```



2. **Stage Progression Testing**
   ```bash
   # Verify workflow progresses through all stages correctly
   stages=("waiting-pr-created" "waiting-ready-for-qa" "waiting-pr-approved")

   for stage in "${stages[@]}"; do
     # Trigger stage update
     argo submit stage-update-test.yaml --parameter new-stage=$stage

     # Verify update
     actual_stage=$(argo get workflow test-workflow -o jsonpath='{.metadata.labels.current-stage}')
     [ "$actual_stage" = "$stage" ] || echo "ERROR: Expected $stage, got $actual_stage"
   done





```

### Event Integration Testing


1. **Webhook Correlation Testing**


   - Send test GitHub webhooks with task labels


   - Verify correct workflows are resumed at correct stages


   - Test multiple concurrent workflows don't interfere



2. **Stage-Specific Resumption**


   - Workflows only resume when in correct stage


   - Wrong-stage workflows remain suspended


   - Multiple workflows with same task ID handled correctly

## Key Design Decisions

1. **Kubernetes Native Labels**: Use workflow metadata labels for state tracking
2. **Atomic Updates**: JSON merge patches prevent race conditions
3. **Stage Isolation**: Each stage explicitly updates workflow state
4. **Event Correlation**: Combine task ID and stage for precise targeting
5. **Idempotent Design**: All operations safe to retry and duplicate

## Troubleshooting Guide

### Common Issues
1. **Label Update Failures**: Check RBAC permissions for workflow patches
2. **Race Conditions**: Verify resource versioning in patch operations
3. **Wrong Stage Resumption**: Check label selector syntax in sensors
4. **Orphaned Workflows**: Implement TTL and cleanup for stuck workflows

### Monitoring Points


- Label update success/failure rates


- Stage transition latency


- Workflow suspension duration


- Event correlation accuracy

## References

- [Argo Workflows Resource Templates](https://argoproj.github.io/argo-workflows/workflow-templates/#resource-template)
- [Kubernetes JSON Patch Operations](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/update-api-object-kubectl-patch/)
- [Argo Events Sensor Configuration](https://argoproj.github.io/argo-events/sensors/triggers/argo-workflow/)


- [Multi-Agent Architecture](/.taskmaster/docs/architecture.md)
