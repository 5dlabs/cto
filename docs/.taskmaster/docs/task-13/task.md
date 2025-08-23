# Task 13: Implement Task Progression Logic



## Overview

Build workflow logic to automatically move completed tasks to the `.completed` directory and discover the next pending task for processing. This task enables continuous task processing in the multi-agent orchestration system, allowing workflows to automatically progress through the task queue without manual intervention.

## Context

The multi-agent Play Workflow processes individual tasks through sequential quality gates (Rex → Cleo → Tess → Human approval). Currently, each task requires manual workflow initiation. Task progression logic enables:

- **Automatic Task Completion**: Move finished tasks to `.completed` directory
- **Next Task Discovery**: Identify the next pending task to process
- **Continuous Processing**: Chain task workflows together
- **Queue Management**: Maintain organized task directory structure

## Technical Architecture



### Task Directory Structure







```
docs/.taskmaster/docs/
├── task-1/                    # Completed task (to be moved)
│   ├── task.txt
│   └── [documentation files]
├── task-2/                    # Current task (in progress)
│   ├── task.txt
│   └── [documentation files]
├── task-3/                    # Next pending task
│   ├── task.txt
│   └── [documentation files]
├── .completed/                # Completed tasks archive
│   ├── task-1/               # Moved after completion
│   │   ├── task.txt
│   │   └── [documentation files]
│   └── [other completed tasks]
└── architecture.md






```

### Workflow Integration Points




```yaml
# Argo Workflow template with task progression
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: play-workflow-
spec:
  entrypoint: main
  templates:
  - name: main
    dag:
      tasks:
      # ... existing workflow steps ...

      - name: complete-task
        dependencies: [wait-pr-approved]
        template: mark-task-complete

      - name: discover-next-task
        dependencies: [complete-task]
        template: find-next-task

      - name: continue-or-finish
        dependencies: [discover-next-task]
        template: conditional-continue

  - name: mark-task-complete
    script:
      image: alpine/git
      command: [sh]
      source: |
        # Move completed task to .completed directory
        TASK_ID="{{workflow.parameters.task-id}}"

        echo "Moving task-$TASK_ID to completed directory"
        mkdir -p docs/.taskmaster/docs/.completed
        mv "docs/.taskmaster/docs/task-$TASK_ID" "docs/.taskmaster/docs/.completed/task-$TASK_ID"

        # Commit the task completion
        git add -A
        git commit -m "Complete task $TASK_ID - moved to .completed directory"

        echo "Task $TASK_ID marked as complete"

  - name: find-next-task
    script:
      image: alpine
      command: [sh]
      source: |
        # Discover next pending task
        NEXT_TASK=$(find docs/.taskmaster/docs/ -maxdepth 1 -name "task-*" -type d | \
                   grep -v ".completed" | \
                   sort -V | \
                   head -1 | \
                   grep -o 'task-[0-9]*' | \
                   cut -d'-' -f2)

        if [ -n "$NEXT_TASK" ]; then
          echo "Next task found: $NEXT_TASK"
          echo "$NEXT_TASK" > /tmp/next-task-id
        else
          echo "No more tasks to process"
          echo "" > /tmp/next-task-id
        fi
      outputs:
        parameters:
        - name: next-task-id
          valueFrom:
            path: /tmp/next-task-id

  - name: conditional-continue
    inputs:
      parameters:
      - name: next-task-id
    steps:
    - - name: continue-with-next-task
        when: "{{inputs.parameters.next-task-id}} != ''"
        template: start-new-workflow
        arguments:
          parameters:
          - name: task-id
            value: "{{inputs.parameters.next-task-id}}"
    - - name: all-tasks-complete
        when: "{{inputs.parameters.next-task-id}} == ''"
        template: finalize-processing

  - name: start-new-workflow
    inputs:
      parameters:
      - name: task-id
    resource:
      action: create
      manifest: |
        apiVersion: argoproj.io/v1alpha1
        kind: Workflow
        metadata:
          generateName: play-workflow-
          namespace: argo
        spec:
          arguments:
            parameters:
            - name: task-id
              value: "{{inputs.parameters.task-id}}"
          workflowTemplateRef:
            name: play-workflow-template






```

## Implementation Requirements

### 1. Task Completion Logic

**Directory Movement Operation**:



```bash
#!/bin/bash
# Task completion script

TASK_ID="$1"
SOURCE_DIR="docs/.taskmaster/docs/task-$TASK_ID"
TARGET_DIR="docs/.taskmaster/docs/.completed/task-$TASK_ID"

# Validate task exists and is complete
if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Task directory $SOURCE_DIR not found"
    exit 1
fi

# Ensure .completed directory exists
mkdir -p "docs/.taskmaster/docs/.completed"



# Move task directory
echo "Moving $SOURCE_DIR to $TARGET_DIR"
mv "$SOURCE_DIR" "$TARGET_DIR"

if [ $? -eq 0 ]; then
    echo "Task $TASK_ID successfully moved to completed directory"
else
    echo "Error: Failed to move task $TASK_ID"
    exit 1
fi



# Update task marker file
echo '{"lastCompletedTask":"'$TASK_ID'","completedAt":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}' > docs/.taskmaster/last-completed.json






```



### 2. Next Task Discovery

**Discovery Algorithm**:



```bash
#!/bin/bash


# Next task discovery script

# Find next pending task
find_next_task() {
    find docs/.taskmaster/docs/ \


        -maxdepth 1 \


        -name "task-*" \


        -type d \
        | grep -v ".completed" \
        | sort -V \
        | head -1 \
        | grep -o 'task-[0-9]*' \
        | cut -d'-' -f2
}

NEXT_TASK=$(find_next_task)

if [ -n "$NEXT_TASK" ]; then
    echo "Next pending task: $NEXT_TASK"

    # Validate task has required files
    if [ -f "docs/.taskmaster/docs/task-$NEXT_TASK/task.txt" ]; then
        echo "Task $NEXT_TASK validated and ready for processing"
        echo "$NEXT_TASK"
    else
        echo "Error: Task $NEXT_TASK missing required files"
        exit 1
    fi
else
    echo "No more tasks to process - queue complete"
    echo ""
fi






```

### 3. Workflow Loop Implementation

**Argo Workflows Loop Pattern**:



```yaml
# Recursive workflow pattern for continuous task processing
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: play-workflow-template
  namespace: argo
spec:
  entrypoint: process-task
  templates:
  - name: process-task
    inputs:
      parameters:
      - name: task-id
    dag:
      tasks:
      - name: create-task-marker
        template: mark-task-in-progress
        arguments:
          parameters:
          - name: task-id
            value: "{{inputs.parameters.task-id}}"

      - name: implementation-work
        dependencies: [create-task-marker]
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Rex"  # Or workflow parameter for agent selection
          - name: task-id
            value: "{{inputs.parameters.task-id}}"

      # ... rest of multi-agent pipeline ...

      - name: complete-and-continue
        dependencies: [wait-pr-approved]
        template: task-completion-loop
        arguments:
          parameters:
          - name: current-task-id
            value: "{{inputs.parameters.task-id}}"

  - name: task-completion-loop
    inputs:
      parameters:
      - name: current-task-id
    steps:
    - - name: mark-complete
        template: mark-task-complete
        arguments:
          parameters:
          - name: task-id
            value: "{{inputs.parameters.current-task-id}}"
    - - name: find-next
        template: find-next-task
    - - name: continue-processing
        when: "{{steps.find-next.outputs.parameters.next-task-id}} != ''"
        templateRef:
          name: play-workflow-template
          template: process-task
        arguments:
          parameters:
          - name: task-id
            value: "{{steps.find-next.outputs.parameters.next-task-id}}"






```

### 4. Edge Case Handling

**Validation and Error Handling**:



```bash
# Comprehensive edge case handling

validate_task_structure() {
    local task_id="$1"
    local task_dir="docs/.taskmaster/docs/task-$task_id"

    # Check task directory exists
    if [ ! -d "$task_dir" ]; then
        echo "Error: Task directory not found: $task_dir"
        return 1
    fi

    # Check required task.txt file
    if [ ! -f "$task_dir/task.txt" ]; then
        echo "Error: Missing task.txt file in $task_dir"
        return 1
    fi

    # Validate task.txt format
    if ! grep -q "^# Task ID: $task_id$" "$task_dir/task.txt"; then
        echo "Error: Invalid task.txt format in $task_dir"
        return 1
    fi

    echo "Task $task_id structure validated"
    return 0
}

handle_corrupted_task() {
    local task_id="$1"
    local task_dir="docs/.taskmaster/docs/task-$task_id"

    echo "Handling corrupted task: $task_id"

    # Move corrupted task to quarantine
    mkdir -p "docs/.taskmaster/docs/.corrupted"
    mv "$task_dir" "docs/.taskmaster/docs/.corrupted/task-$task_id"

    # Log the issue
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): Task $task_id moved to quarantine due to corruption" >> docs/.taskmaster/task-errors.log

    # Continue with next task
    find_next_task
}

handle_no_more_tasks() {
    echo "All tasks in queue have been processed"

    # Create completion marker
    echo '{"status":"complete","completedAt":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'","totalTasks":"'$(ls docs/.taskmaster/docs/.completed/ | grep task- | wc -l)'"}' > docs/.taskmaster/queue-complete.json

    # Commit final state
    git add docs/.taskmaster/queue-complete.json
    git commit -m "All tasks completed - queue processing finished"

    echo "Task queue processing complete"
}






```

## Implementation Steps

### Phase 1: Task Completion Workflow Step

1. **Create task completion template**:


   - Implement directory movement logic


   - Add git commit for completion tracking


   - Include validation and error handling


   - Test with individual task completion

2. **Integrate with existing workflow**:


   - Add completion step after PR approval


   - Test completion logic in isolation


   - Validate directory structure changes



### Phase 2: Next Task Discovery

1. **Implement discovery algorithm**:


   - Create task finding script with proper sorting


   - Add task structure validation


   - Handle edge cases (no tasks, corrupted tasks)


   - Test discovery with various directory states

2. **Output parameter handling**:


   - Configure Argo workflow parameter passing


   - Test parameter propagation between steps


   - Validate conditional logic for next steps



### Phase 3: Workflow Loop Logic

1. **Implement conditional continuation**:


   - Create recursive workflow pattern


   - Add loop termination conditions


   - Test workflow chaining between tasks


   - Validate resource cleanup and limits

2. **Error handling and recovery**:


   - Handle workflow failures gracefully


   - Implement retry logic for transient failures


   - Add monitoring and alerting for stuck workflows

### Phase 4: Integration and Testing

1. **End-to-end testing**:


   - Test complete task progression sequence


   - Validate multiple task processing


   - Test edge cases and error conditions


   - Monitor resource usage and performance

2. **Production readiness**:


   - Add comprehensive logging and monitoring


   - Implement workflow resource limits


   - Create operational documentation


   - Set up alerts for processing failures



## Task Discovery Algorithm

### Sorting Logic



```bash
# Natural version sorting for proper task ordering
find docs/.taskmaster/docs/ \


    -maxdepth 1 \


    -name "task-*" \


    -type d \
    | grep -v ".completed" \
    | sort -V \
    | head -1

# Example sorting behavior:
# task-2   -> Selected (lowest number)


# task-3


# task-10


# task-15






```

### Task Validation



```bash
validate_next_task() {
    local task_path="$1"
    local task_id=$(echo "$task_path" | grep -o 'task-[0-9]*' | cut -d'-' -f2)

    # Check task.txt exists and is valid
    if [ -f "$task_path/task.txt" ] &&
       grep -q "^# Task ID: $task_id$" "$task_path/task.txt" &&
       grep -q "^# Status: pending$" "$task_path/task.txt"; then
        return 0
    else
        return 1
    fi
}






```

## Integration with Multi-Agent System



### Workflow Parameter Flow



```yaml
# Task ID propagation through workflow
spec:
  arguments:
    parameters:
    - name: task-id
      value: "3"  # Discovered from previous completion
  templates:
  - name: agent-coderun
    inputs:
      parameters:
      - name: github-app
      - name: task-id
    # Agent uses task-id to:
    # 1. Read task file: docs/.taskmaster/docs/task-{{inputs.parameters.task-id}}/task.txt
    # 2. Create PR with label: task-{{inputs.parameters.task-id}}
    # 3. Use branch name: task-{{inputs.parameters.task-id}}-feature-name






```

### Event Correlation Updates



```yaml
# Argo Events sensor updates for dynamic task correlation
apiVersion: argoproj.io/v1alpha1
kind: Sensor
spec:
  triggers:
  - template:
      name: resume-workflow-on-event
      argoWorkflow:
        source:
          resource:
            labelSelector: |
              workflow-type=play-orchestration,
              task-id={{extracted-task-id}},
              current-stage={{target-stage}}
        # Task ID extracted from PR labels dynamically matches
        # the task ID from workflow parameters






```

## Monitoring and Observability

### Task Processing Metrics



```yaml
# Metrics to track task progression
metrics:


- task_completion_rate


- task_processing_duration


- queue_length (pending tasks)


- workflow_chain_length (continuous processing)


- task_failure_rate


- corrupted_task_count






```

### Logging Strategy



```bash
# Structured logging for task progression
log_task_event() {
    local event="$1"
    local task_id="$2"
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local workflow_name="$3"

    echo "[TASK_PROGRESSION] $timestamp event=$event task_id=$task_id workflow=$workflow_name" >> docs/.taskmaster/progression.log
}



# Usage examples:
log_task_event "TASK_COMPLETED" "$TASK_ID" "$WORKFLOW_NAME"
log_task_event "NEXT_TASK_DISCOVERED" "$NEXT_TASK" "$WORKFLOW_NAME"
log_task_event "QUEUE_COMPLETE" "" "$WORKFLOW_NAME"






```

## Error Handling Strategies



### Workflow Failure Recovery



```yaml
# Retry policy for task progression failures
spec:
  retryPolicy:
    limit: 3
    retryPolicy: Always
    backoff:
      duration: "30s"
      factor: 2
      maxDuration: "5m"

  # Failure handling
  onExit:
    template: handle-progression-failure

templates:
- name: handle-progression-failure
  script:
    source: |
      echo "Task progression failed - investigating"

      # Log failure details
      echo "$(date): Workflow {{workflow.name}} failed during task progression" >> docs/.taskmaster/failures.log

      # Check for partial completion
      if [ -f "/tmp/next-task-id" ]; then
        NEXT_TASK=$(cat /tmp/next-task-id)
        echo "Next task was identified: $NEXT_TASK"
        # Could trigger manual recovery workflow
      fi






```

### Deadlock Prevention



```yaml
# Prevent infinite loops and resource exhaustion
spec:
  activeDeadlineSeconds: 86400  # 24 hours max for any single workflow chain
  parallelism: 1  # Ensure sequential task processing

  # Resource limits for task progression workflows
  podGC:
    strategy: OnWorkflowCompletion
  ttlStrategy:
    secondsAfterCompletion: 3600  # Clean up after 1 hour






```

## Testing Strategy

### Unit Testing Scenarios
1. **Task Completion**: Single task move to `.completed`
2. **Next Task Discovery**: Find correct next task in sequence
3. **Empty Queue**: Handle no more tasks gracefully
4. **Corrupted Tasks**: Handle invalid task structures
5. **Concurrent Processing**: Prevent race conditions

### Integration Testing Scenarios
1. **End-to-End Progression**: Complete 3-task sequence
2. **Workflow Chaining**: Validate recursive workflow creation
3. **Multi-Agent Integration**: Ensure agent coordination works
4. **Event Correlation**: Verify PR events match correct workflows

### Performance Testing
1. **Large Queue Processing**: 50+ tasks in sequence
2. **Resource Usage**: Monitor memory and CPU during progression
3. **Workflow Cleanup**: Ensure completed workflows are cleaned up
4. **Concurrent Workflow Limits**: Test system under load

## Dependencies

- **Task 3**: Multi-agent orchestration system foundation
- **Task 7**: Event-driven workflow coordination


- Argo Workflows with DAG and recursive template support


- Git repository with proper directory permissions


- Kubernetes cluster with sufficient resources for workflow chaining



## Expected Outcomes

### Functional Outcomes
1. **Automatic Task Processing**: Tasks process sequentially without manual intervention
2. **Clean Directory Management**: Completed tasks moved to `.completed` directory
3. **Queue Exhaustion Handling**: System gracefully handles empty task queue
4. **Error Recovery**: Failed tasks don't break the processing chain

### Operational Outcomes
1. **Reduced Manual Overhead**: No manual workflow triggering between tasks
2. **Better Task Organization**: Clear separation between active and completed tasks
3. **Improved Visibility**: Task progression status easily tracked
4. **Scalable Processing**: System handles growing task queues efficiently

## Future Enhancements

- **Parallel Task Processing**: Process multiple independent tasks simultaneously
- **Priority Queue Support**: Process high-priority tasks first
- **Task Dependencies**: Support task dependencies beyond sequential ordering
- **Progress Dashboards**: Real-time visualization of task queue progress
- **Notification Integration**: Alert on task completion and queue status
- **Task Retry Logic**: Automatically retry failed tasks with backoff
