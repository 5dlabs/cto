# Task 26: Implement Comprehensive Task Association Validation Using Multi-Method Approach

## Overview

This task implements a robust three-tier validation system to ensure accurate correlation between GitHub workflows and Task Master tasks. The system prevents workflow execution errors by requiring agreement between PR labels, branch naming patterns, and marker files before allowing workflow execution.

## Technical Requirements

### Validation Methods

1. **Primary Method - PR Labels**
   - Pattern: `task-{id}` labels on pull requests
   - JQ extraction: `.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]`

2. **Secondary Method - Branch Naming**
   - Patterns: `task-{id}-{description}` or `feature/task-{id}`
   - Regex: `^(?:feature/)?task-(\d+)(?:-.*)?$`
   - Source: `pull_request.head.ref` field

3. **Fallback Method - Marker File**
   - Location: `docs/.taskmaster/current-task.json`
   - Schema: `{"task_id": "26", "started_at": "2024-01-15T10:00:00Z", "agent": "rex"}`

## Implementation Guide

### Step 1: Argo Workflows Sensor Configuration

Create or update the Argo Events Sensor with validation templates:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: task-validation-sensor
spec:
  triggers:
  - template:
      name: validate-task-association
      argoWorkflow:
        operation: submit
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: validate-task-
            spec:
              entrypoint: validate-and-execute
              arguments:
                parameters:
                - name: pr-number
                  value: "{{.Input.pull_request.number}}"
                - name: pr-labels
                  value: "{{.Input.pull_request.labels | tojson}}"
                - name: branch-ref
                  value: "{{.Input.pull_request.head.ref}}"
                - name: commit-sha
                  value: "{{.Input.pull_request.head.sha}}"
```

### Step 2: Workflow Template Implementation

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: task-validation-template
spec:
  entrypoint: validate-and-execute
  templates:
  - name: validate-and-execute
    steps:
    - - name: extract-task-ids
        template: extract-all-methods
        arguments:
          parameters:
          - name: pr-labels
            value: "{{workflow.parameters.pr-labels}}"
          - name: branch-ref
            value: "{{workflow.parameters.branch-ref}}"
    - - name: validate-agreement
        template: validate-task-ids
        arguments:
          parameters:
          - name: label-task-id
            value: "{{steps.extract-task-ids.outputs.parameters.label-id}}"
          - name: branch-task-id
            value: "{{steps.extract-task-ids.outputs.parameters.branch-id}}"
          - name: marker-task-id
            value: "{{steps.extract-task-ids.outputs.parameters.marker-id}}"
    - - name: create-marker-file
        template: marker-file-creator
        when: "{{steps.validate-agreement.outputs.parameters.validation-passed}} == true"
        arguments:
          parameters:
          - name: task-id
            value: "{{steps.validate-agreement.outputs.parameters.agreed-task-id}}"

  - name: extract-all-methods
    inputs:
      parameters:
      - name: pr-labels
      - name: branch-ref
    container:
      image: stedolan/jq:latest
      command: [sh, -c]
      args:
      - |
        # Extract from PR labels
        LABEL_ID=$(echo '{{inputs.parameters.pr-labels}}' | jq -r '.[] | select(.name | startswith("task-")) | .name | split("-")[1]' | head -1)
        
        # Extract from branch name
        BRANCH_ID=$(echo '{{inputs.parameters.branch-ref}}' | sed -n 's/^.*task-\([0-9]\+\).*$/\1/p')
        
        # Read marker file
        if [ -f /workspace/docs/.taskmaster/current-task.json ]; then
          MARKER_ID=$(cat /workspace/docs/.taskmaster/current-task.json | jq -r '.task_id')
        else
          MARKER_ID=""
        fi
        
        echo -n "$LABEL_ID" > /tmp/label-id
        echo -n "$BRANCH_ID" > /tmp/branch-id  
        echo -n "$MARKER_ID" > /tmp/marker-id
      volumeMounts:
      - name: workspace
        mountPath: /workspace
    outputs:
      parameters:
      - name: label-id
        valueFrom:
          path: /tmp/label-id
      - name: branch-id
        valueFrom:
          path: /tmp/branch-id
      - name: marker-id
        valueFrom:
          path: /tmp/marker-id

  - name: validate-task-ids
    inputs:
      parameters:
      - name: label-task-id
      - name: branch-task-id
      - name: marker-task-id
    container:
      image: alpine:latest
      command: [sh, -c]
      args:
      - |
        LABEL_ID="{{inputs.parameters.label-task-id}}"
        BRANCH_ID="{{inputs.parameters.branch-task-id}}"
        MARKER_ID="{{inputs.parameters.marker-task-id}}"
        
        echo "Validating task IDs:"
        echo "  Label ID: $LABEL_ID"
        echo "  Branch ID: $BRANCH_ID"
        echo "  Marker ID: $MARKER_ID"
        
        # Check if all non-empty IDs match
        VALID_IDS=""
        if [ -n "$LABEL_ID" ]; then VALID_IDS="$LABEL_ID"; fi
        if [ -n "$BRANCH_ID" ]; then
          if [ -z "$VALID_IDS" ]; then
            VALID_IDS="$BRANCH_ID"
          elif [ "$VALID_IDS" != "$BRANCH_ID" ]; then
            echo "ERROR: Branch ID ($BRANCH_ID) doesn't match Label ID ($LABEL_ID)"
            echo "false" > /tmp/validation-passed
            exit 1
          fi
        fi
        if [ -n "$MARKER_ID" ]; then
          if [ -z "$VALID_IDS" ]; then
            VALID_IDS="$MARKER_ID"
          elif [ "$VALID_IDS" != "$MARKER_ID" ]; then
            echo "ERROR: Marker ID ($MARKER_ID) doesn't match other IDs ($VALID_IDS)"
            echo "false" > /tmp/validation-passed
            exit 1
          fi
        fi
        
        if [ -z "$VALID_IDS" ]; then
          echo "ERROR: No valid task ID found in any method"
          echo "false" > /tmp/validation-passed
          exit 1
        fi
        
        echo "SUCCESS: All methods agree on task ID: $VALID_IDS"
        echo "true" > /tmp/validation-passed
        echo "$VALID_IDS" > /tmp/agreed-task-id
    outputs:
      parameters:
      - name: validation-passed
        valueFrom:
          path: /tmp/validation-passed
      - name: agreed-task-id
        valueFrom:
          path: /tmp/agreed-task-id

  - name: marker-file-creator
    inputs:
      parameters:
      - name: task-id
    container:
      image: alpine/git:latest
      command: [sh, -c]
      args:
      - |
        cd /workspace
        
        # Create marker file
        cat > docs/.taskmaster/current-task.json <<EOF
        {
          "task_id": "{{inputs.parameters.task-id}}",
          "started_at": "$(date -Iseconds)",
          "agent": "{{workflow.parameters.implementation-agent}}",
          "workflow_id": "{{workflow.name}}",
          "commit_sha": "$(git rev-parse HEAD)",
          "pr_number": "{{workflow.parameters.pr-number}}"
        }
        EOF
        
        # Commit the marker file
        git add docs/.taskmaster/current-task.json
        git commit -m "chore: Set current task marker for task-{{inputs.parameters.task-id}}"
        git push origin HEAD
      volumeMounts:
      - name: workspace
        mountPath: /workspace
  
  volumes:
  - name: workspace
    persistentVolumeClaim:
      claimName: task-workspace
```

### Step 3: Error Handling and GitHub Integration

Implement GitHub comment creation for validation failures:

```yaml
  - name: report-validation-error
    inputs:
      parameters:
      - name: error-message
      - name: pr-number
    container:
      image: curlimages/curl:latest
      command: [sh, -c]
      args:
      - |
        cat <<EOF > /tmp/comment.json
        {
          "body": "## Task Association Validation Failed\n\n{{inputs.parameters.error-message}}\n\n### Required Actions:\n1. Ensure PR has correct 'task-{id}' label\n2. Verify branch name follows pattern 'task-{id}-description'\n3. Check marker file contains matching task ID\n\nAll three methods must agree for workflow to proceed."
        }
        EOF
        
        curl -X POST \
          -H "Authorization: token $GITHUB_TOKEN" \
          -H "Accept: application/vnd.github.v3+json" \
          -d @/tmp/comment.json \
          "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/{{inputs.parameters.pr-number}}/comments"
      env:
      - name: GITHUB_TOKEN
        valueFrom:
          secretKeyRef:
            name: github-token
            key: token
      - name: GITHUB_REPOSITORY
        value: "5dlabs/cto"
```

### Step 4: Retry and Recovery Logic

```yaml
  - name: validation-with-retry
    retryStrategy:
      limit: 3
      retryPolicy: "Always"
      backoff:
        duration: "30s"
        factor: 2
        maxDuration: "5m"
    steps:
    - - name: attempt-validation
        template: validate-task-ids
```

## Error Scenarios and Handling

1. **Label Missing**: Creates GitHub comment requesting proper task label
2. **Branch Mismatch**: Reports discrepancy and blocks execution
3. **Marker File Absent**: Attempts to create marker file based on agreed ID
4. **All Different**: Detailed error report with all found IDs
5. **Transient Failures**: Automatic retry up to 3 attempts

## Monitoring and Observability

### Metrics Collection

```yaml
  - name: emit-validation-metrics
    container:
      image: prom/pushgateway:latest
      command: [sh, -c]
      args:
      - |
        echo "task_validation_attempts_total{status=\"{{inputs.parameters.status}}\"} 1" | \
          curl -X POST --data-binary @- http://pushgateway:9091/metrics/job/task-validation
```

### Logging

```yaml
  - name: log-validation-result
    container:
      image: alpine:latest
      command: [sh, -c]
      args:
      - |
        echo "$(date -Iseconds) [INFO] Task validation: {{inputs.parameters.result}} for workflow {{workflow.name}}"
```

## Security Considerations

1. **Token Management**: GitHub tokens stored in Kubernetes secrets
2. **File Permissions**: Marker files created with restricted permissions
3. **Input Validation**: All external inputs sanitized before processing
4. **Audit Trail**: All validation attempts logged with timestamps

## Integration Points

- **Argo Events**: Sensor triggers for PR events
- **Argo Workflows**: Template execution and status tracking
- **GitHub API**: Comment creation and label validation
- **Persistent Storage**: Marker file persistence across workflow stages
- **Monitoring**: Prometheus metrics and Grafana dashboards