# Task 8: Build Rex Remediation Sensor



## Overview

Create a dedicated Argo Events Sensor to detect Rex push events and trigger QA pipeline restart with downstream cancellation. This ensures that when Rex addresses PR feedback, any running Cleo/Tess work is cancelled and the QA pipeline restarts cleanly.

## Technical Context

When Rex pushes fixes in response to PR feedback, any ongoing Cleo or Tess work becomes obsolete since it's based on outdated code. The system needs to automatically detect Rex pushes and restart the QA pipeline from the quality-work stage while cancelling any running downstream agents.

## Implementation Guide

### Phase 1: Create Rex Remediation EventSource



1. **GitHub Push Event Detection**
   ```yaml
   # EventSource for Rex push events
   apiVersion: argoproj.io/v1alpha1
   kind: EventSource
   metadata:
     name: rex-remediation-events
   spec:
     github:
       rex-push:
         repositories:
         - owner: 5dlabs
           names: ["cto"]
         webhook:
           endpoint: "/rex-push-events"
           port: "12000"
           method: POST
         events: ["push"]
         filter:
           expression: |
             (
               body.sender.login == "5DLabs-Rex[bot]" ||
               body.pusher.name == "5DLabs-Rex[bot]"
             ) &&
             body.ref matches "refs/heads/task-.*"





```



2. **Task ID Extraction Setup**
   ```yaml
   # Data extraction for task correlation
   dataFilters:
   - path: "body.ref"
     type: string
     template: |
       {{ .body.ref | regexReplaceAll "refs/heads/task-([0-9]+)-.*" "${1}" }}
   - path: "body.repository.name"
     type: string
   - path: "body.sender.login"
     type: string





```

### Phase 2: Implement Agent Cancellation Logic



1. **CodeRun Deletion Template**
   ```yaml
   # Kubernetes resource template for agent cancellation
   - name: cancel-downstream-agents
     resource:
       action: delete
       source:
         resource:
           apiVersion: agents.platform/v1
           kind: CodeRun
           metadata:
             labelSelector: |
               task-id={{task-id}},
               github-app in (5DLabs-Cleo,5DLabs-Tess)
       successCondition: "status.phase == Succeeded"
       failureCondition: "status.phase == Failed"





```



2. **Label-Based Agent Selection**
   ```bash
   #!/bin/bash
   # Script for selective agent cancellation

   TASK_ID="{{inputs.parameters.task-id}}"

   # Cancel Cleo agents for this task
   kubectl delete coderun -l "task-id=${TASK_ID},github-app=5DLabs-Cleo" --wait=true

   # Cancel Tess agents for this task
   kubectl delete coderun -l "task-id=${TASK_ID},github-app=5DLabs-Tess" --wait=true

   # Verify cancellation completed
   REMAINING=$(kubectl get coderun -l "task-id=${TASK_ID},github-app in (5DLabs-Cleo,5DLabs-Tess)" --no-headers | wc -l)

   if [ "$REMAINING" -eq 0 ]; then
     echo "✅ All downstream agents cancelled successfully"
   else
     echo "⚠️  $REMAINING agents still running, may need manual intervention"
   fi





```

### Phase 3: Create Rex Remediation Sensor



1. **Complete Sensor Configuration**
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: rex-remediation-restart
     namespace: argo-events
   spec:
     dependencies:
     - name: rex-push-event
       eventSourceName: rex-remediation-events
       eventName: rex-push
       filters:
         dataFilters:
         - path: "body.sender.login"
           type: string
           value:


           - "5DLabs-Rex[bot]"
         - path: "body.ref"
           type: string
           comparator: "=~"
           value: "refs/heads/task-[0-9]+-.*"

     triggers:
     - template:
         name: restart-qa-pipeline
         conditions: "rex-push-event"
         argoWorkflow:
           operation: submit
           source:
             resource:
               apiVersion: argoproj.io/v1alpha1
               kind: Workflow
               metadata:
                 generateName: rex-remediation-
                 labels:
                   workflow-type: rex-remediation
                   task-id: "{{rex-push-event.task-id}}"
               spec:
                 entrypoint: remediation-flow
                 arguments:
                   parameters:
                   - name: task-id
                     value: "{{rex-push-event.task-id}}"
                   - name: repository
                     value: "{{rex-push-event.repository}}"
                   - name: branch-ref
                     value: "{{rex-push-event.ref}}"





```



2. **Remediation Workflow Template**
   ```yaml
   # Workflow template for Rex remediation process
   templates:
   - name: remediation-flow
     dag:
       tasks:
       - name: cancel-downstream-agents
         template: cancel-agents
         arguments:
           parameters:
           - name: task-id
             value: "{{workflow.parameters.task-id}}"

       - name: remove-qa-label
         dependencies: [cancel-downstream-agents]
         template: remove-ready-for-qa-label
         arguments:
           parameters:
           - name: task-id
             value: "{{workflow.parameters.task-id}}"

       - name: reset-workflow-stage
         dependencies: [remove-qa-label]
         template: update-workflow-stage
         arguments:
           parameters:
           - name: new-stage
             value: "waiting-pr-created"
           - name: task-id
             value: "{{workflow.parameters.task-id}}"

       - name: resume-from-cleo
         dependencies: [reset-workflow-stage]
         template: resume-qa-workflow
         arguments:
           parameters:
           - name: task-id
             value: "{{workflow.parameters.task-id}}"





```

### Phase 4: Implement Ready-for-QA Label Removal



1. **GitHub API Label Management**
   ```bash
   #!/bin/bash
   # Script to remove ready-for-qa label from PR

   TASK_ID="{{inputs.parameters.task-id}}"
   GITHUB_TOKEN=$(cat /etc/github-app/token)

   # Find PR number for this task
   PR_NUMBER=$(gh pr list --repo 5dlabs/cto --label "task-${TASK_ID}" --json number --jq '.[0].number')

   if [ -n "$PR_NUMBER" ]; then
     echo "Removing ready-for-qa label from PR #${PR_NUMBER}"

     # Remove ready-for-qa label
     gh pr edit "$PR_NUMBER" --remove-label "ready-for-qa"

     echo "✅ Ready-for-QA label removed, resetting QA pipeline state"
   else
     echo "⚠️  No PR found for task ${TASK_ID}"
   fi





```



2. **Workflow State Reset**
   ```yaml
   - name: update-workflow-stage
     inputs:
       parameters:
       - name: new-stage
       - name: task-id
     script:
       image: bitnami/kubectl:latest
       command: [bash]
       source: |
         TASK_ID="{{inputs.parameters.task-id}}"
         NEW_STAGE="{{inputs.parameters.new-stage}}"

         # Find and update main workflow for this task
         WORKFLOW_NAME=$(kubectl get workflow \


           -l "workflow-type=play-orchestration,task-id=${TASK_ID}" \


           -o jsonpath='{.items[0].metadata.name}')

         if [ -n "$WORKFLOW_NAME" ]; then
           kubectl patch workflow "$WORKFLOW_NAME" \


             --type='merge' \
             --patch="{\"metadata\":{\"labels\":{\"current-stage\":\"$NEW_STAGE\"}}}"
           echo "✅ Workflow $WORKFLOW_NAME reset to stage: $NEW_STAGE"
         else
           echo "⚠️  No workflow found for task $TASK_ID"
         fi





```

### Phase 5: Implement Idempotency and Safety Measures



1. **Duplicate Event Prevention**
   ```yaml
   # Add idempotency check to sensor
   filters:
     dataFilters:
     - path: "body.commits[0].id"
       type: string
       template: "{{.body.commits[0].id}}"
     context:
       # Store processed commit IDs to prevent duplicate processing
       processedCommits: []





```



2. **Safety Checks and Validation**
   ```bash
   #!/bin/bash
   # Safety validation before agent cancellation

   TASK_ID="{{inputs.parameters.task-id}}"

   # Verify this is actually a Rex push event
   SENDER="{{inputs.parameters.sender}}"
   if [[ "$SENDER" != "5DLabs-Rex[bot]" ]]; then
     echo "❌ Invalid sender: $SENDER (expected 5DLabs-Rex[bot])"
     exit 1
   fi

   # Verify task ID is valid
   if ! [[ "$TASK_ID" =~ ^[0-9]+$ ]]; then
     echo "❌ Invalid task ID: $TASK_ID"
     exit 1
   fi

   # Check if there are actually agents to cancel
   AGENTS_TO_CANCEL=$(kubectl get coderun -l "task-id=${TASK_ID},github-app in (5DLabs-Cleo,5DLabs-Tess)" --no-headers | wc -l)

   if [ "$AGENTS_TO_CANCEL" -eq 0 ]; then
     echo "ℹ️  No downstream agents found for task $TASK_ID, nothing to cancel"
   else
     echo "🎯 Found $AGENTS_TO_CANCEL agents to cancel for task $TASK_ID"
   fi





```



## Code Examples

### Complete Rex Remediation Sensor



```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: rex-remediation-restart
  namespace: argo-events
spec:
  template:
    serviceAccountName: argo-events-sa

  dependencies:
  - name: rex-push-event
    eventSourceName: rex-remediation-events
    eventName: rex-push
    filters:
      dataFilters:
      - path: "body.sender.login"
        type: string
        value: ["5DLabs-Rex[bot]"]
      - path: "body.ref"
        type: string
        comparator: "=~"
        value: "refs/heads/task-[0-9]+-.*"
      - path: "body.repository.name"
        type: string
        value: ["cto"]

  triggers:
  - template:
      name: restart-qa-pipeline
      conditions: "rex-push-event"
      argoWorkflow:
        operation: submit
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: rex-remediation-
              labels:
                workflow-type: rex-remediation
                task-id: "{{rex-push-event.body.ref | regexReplaceAll \"refs/heads/task-([0-9]+)-.*\" \"${1}\"}}"
            spec:
              entrypoint: remediation-flow
              serviceAccountName: workflow-executor-sa
              arguments:
                parameters:
                - name: task-id
                  value: "{{rex-push-event.body.ref | regexReplaceAll \"refs/heads/task-([0-9]+)-.*\" \"${1}\"}}"
                - name: repository
                  value: "{{rex-push-event.body.repository.name}}"
                - name: commit-sha
                  value: "{{rex-push-event.body.after}}"
                - name: sender
                  value: "{{rex-push-event.body.sender.login}}"

              templates:
              - name: remediation-flow
                dag:
                  tasks:
                  - name: validate-event
                    template: validate-rex-push

                  - name: cancel-agents
                    dependencies: [validate-event]
                    template: cancel-downstream-agents

                  - name: remove-qa-label
                    dependencies: [cancel-agents]
                    template: remove-ready-for-qa-label

                  - name: reset-workflow
                    dependencies: [remove-qa-label]
                    template: reset-workflow-stage

                  - name: resume-qa-pipeline
                    dependencies: [reset-workflow]
                    template: resume-main-workflow

              - name: validate-rex-push
                script:
                  image: alpine:latest
                  command: [sh]
                  source: |
                    echo "Validating Rex push event..."
                    echo "Task ID: {{workflow.parameters.task-id}}"
                    echo "Sender: {{workflow.parameters.sender}}"
                    echo "Commit: {{workflow.parameters.commit-sha}}"

                    # Validate sender is Rex
                    if [ "{{workflow.parameters.sender}}" != "5DLabs-Rex[bot]" ]; then
                      echo "❌ Invalid sender"
                      exit 1
                    fi

                    echo "✅ Event validation passed"

              - name: cancel-downstream-agents
                script:
                  image: bitnami/kubectl:latest
                  command: [bash]
                  source: |
                    TASK_ID="{{workflow.parameters.task-id}}"

                    echo "🛑 Cancelling downstream agents for task $TASK_ID"

                    # Cancel Cleo agents
                    kubectl delete coderun -l "task-id=${TASK_ID},github-app=5DLabs-Cleo" --ignore-not-found

                    # Cancel Tess agents
                    kubectl delete coderun -l "task-id=${TASK_ID},github-app=5DLabs-Tess" --ignore-not-found

                    echo "✅ Agent cancellation completed"

              - name: remove-ready-for-qa-label
                script:
                  image: ghcr.io/cli/cli:latest
                  command: [bash]
                  source: |
                    TASK_ID="{{workflow.parameters.task-id}}"

                    # Authenticate with GitHub
                    echo "$GITHUB_TOKEN" | gh auth login --with-token

                    # Find PR for this task
                    PR_NUMBER=$(gh pr list --repo 5dlabs/cto \


                      --label "task-${TASK_ID}" \


                      --json number --jq '.[0].number')

                    if [ -n "$PR_NUMBER" ] && [ "$PR_NUMBER" != "null" ]; then
                      echo "📋 Removing ready-for-qa label from PR #${PR_NUMBER}"
                      gh pr edit "$PR_NUMBER" --remove-label "ready-for-qa" --repo 5dlabs/cto
                      echo "✅ Label removed"
                    else
                      echo "ℹ️  No PR found for task $TASK_ID"
                    fi
                  env:
                  - name: GITHUB_TOKEN
                    valueFrom:
                      secretKeyRef:
                        name: github-app-5dlabs-rex
                        key: token

              - name: reset-workflow-stage
                script:
                  image: bitnami/kubectl:latest
                  command: [bash]
                  source: |
                    TASK_ID="{{workflow.parameters.task-id}}"

                    # Find main workflow for this task
                    WORKFLOW_NAME=$(kubectl get workflow \


                      -l "workflow-type=play-orchestration,task-id=${TASK_ID}" \


                      -o jsonpath='{.items[0].metadata.name}')

                    if [ -n "$WORKFLOW_NAME" ]; then
                      echo "🔄 Resetting workflow $WORKFLOW_NAME to waiting-pr-created"
                      kubectl patch workflow "$WORKFLOW_NAME" \


                        --type='merge' \
                        --patch='{"metadata":{"labels":{"current-stage":"waiting-pr-created"}}}'
                      echo "✅ Workflow stage reset"
                    else
                      echo "⚠️  No main workflow found for task $TASK_ID"
                    fi

              - name: resume-main-workflow
                script:
                  image: bitnami/kubectl:latest
                  command: [bash]
                  source: |
                    TASK_ID="{{workflow.parameters.task-id}}"

                    # Resume main workflow at quality stage
                    WORKFLOW_NAME=$(kubectl get workflow \


                      -l "workflow-type=play-orchestration,task-id=${TASK_ID}" \


                      -o jsonpath='{.items[0].metadata.name}')

                    if [ -n "$WORKFLOW_NAME" ]; then
                      echo "▶️  Resuming workflow $WORKFLOW_NAME"
                      argo resume "$WORKFLOW_NAME"
                      echo "✅ QA pipeline restart initiated"
                    else
                      echo "⚠️  No workflow to resume for task $TASK_ID"
                    fi






```

## Architecture Patterns

### Event-Driven Cancellation Flow






```
Rex Push Event → Sensor Detection → Agent Cancellation → Label Removal → Workflow Reset → Pipeline Restart






```

### Idempotent Operations
All remediation operations are designed to be idempotent:


- Agent cancellation ignores already-deleted agents


- Label removal checks if label exists before removal


- Workflow stage updates verify current state


- Resume operations check workflow status first



### Safety-First Approach


- Validate sender is actually Rex before any cancellation


- Extract task ID from branch name with validation


- Verify agents exist before attempting cancellation


- Include comprehensive logging for debugging

## Testing Strategy

### Event Detection Testing


1. **Rex Push Event Simulation**
   ```bash
   # Test Rex push event detection
   curl -X POST http://rex-remediation-events:12000/rex-push-events \
     -H "Content-Type: application/json" \


     -d '{
       "sender": {"login": "5DLabs-Rex[bot]"},
       "ref": "refs/heads/task-3-implement-auth",
       "repository": {"name": "cto"},
       "after": "abc123def456"
     }'





```



2. **Agent Cancellation Testing**
   ```bash
   # Create test CodeRun CRDs
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: test-cleo-task-3
     labels:
       task-id: "3"
       github-app: "5DLabs-Cleo"
   spec:
     github_app: "5DLabs-Cleo"
   EOF

   # Trigger remediation
   # Verify CodeRuns are deleted
   kubectl get coderun -l "task-id=3,github-app=5DLabs-Cleo"





```

### Integration Testing


- Test rapid sequential Rex pushes don't cause conflicts


- Verify QA pipeline restarts cleanly after remediation


- Test multiple concurrent tasks don't interfere


- Validate error handling for edge cases

## Key Design Decisions

1. **Dedicated Sensor**: Separate sensor for Rex remediation keeps logic focused
2. **Label-Based Selection**: Use Kubernetes label selectors for precise agent targeting
3. **Complete Pipeline Reset**: Remove labels and reset workflow stage for clean restart
4. **Idempotent Design**: All operations safe to retry and duplicate
5. **Comprehensive Validation**: Validate all inputs before taking destructive actions

## References

- [Argo Events Sensor Configuration](https://argoproj.github.io/argo-events/sensors/)
- [GitHub Webhook Events](https://docs.github.com/en/developers/webhooks-and-events/webhook-events-and-payloads#push)
- [Kubernetes Label Selectors](https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#label-selectors)


- [Multi-Agent Architecture](/.taskmaster/docs/architecture.md)
