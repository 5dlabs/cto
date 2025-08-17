# Autonomous Agent Prompt: Implement Task Progression Logic

## Mission

You are tasked with implementing automatic task progression logic for the multi-agent orchestration system. This system will automatically move completed tasks to a `.completed` directory, discover the next pending task, and trigger continuous task processing without manual intervention.

## Context

**System Architecture**: Multi-agent Play Workflow with Rex/Blaze (implementation) → Cleo (quality) → Tess (testing) → Human approval

**Your Role**: Workflow automation engineer implementing continuous task processing

**Current Problem**: Each task requires manual workflow initiation. After Task 5 completes, someone must manually start Task 6. This breaks the automation flow.

**Solution Goal**: Implement automatic task queue processing where completing Task N automatically starts Task N+1, continuing until all tasks are processed.

## Primary Objectives

### 1. Task Completion Logic
Implement workflow step that moves completed tasks from `docs/.taskmaster/docs/task-X/` to `docs/.taskmaster/docs/.completed/task-X/`

### 2. Next Task Discovery
Create algorithm to find the next pending task using natural version sorting and validation

### 3. Workflow Loop Implementation
Implement Argo Workflows recursive pattern to automatically start next task processing

### 4. Edge Case Handling
Handle scenarios: no more tasks, corrupted task structure, workflow failures

## Technical Implementation

### Phase 1: Task Completion Workflow Step

**Argo Workflow Template Addition**:
```yaml
templates:
- name: mark-task-complete
  inputs:
    parameters:
    - name: task-id
  script:
    image: alpine/git:latest
    command: [sh]
    source: |
      set -e
      TASK_ID="{{inputs.parameters.task-id}}"
      SOURCE_DIR="docs/.taskmaster/docs/task-$TASK_ID"
      TARGET_DIR="docs/.taskmaster/docs/.completed/task-$TASK_ID"
      
      echo "Moving task $TASK_ID to completed directory"
      
      # Validate source directory exists
      if [ ! -d "$SOURCE_DIR" ]; then
        echo "Error: Source directory $SOURCE_DIR not found"
        exit 1
      fi
      
      # Create .completed directory if needed
      mkdir -p "docs/.taskmaster/docs/.completed"
      
      # Move task directory
      mv "$SOURCE_DIR" "$TARGET_DIR"
      
      # Commit completion
      cd /workspace/src
      git add -A
      git commit -m "Task $TASK_ID completed - moved to .completed directory"
      
      echo "Task $TASK_ID successfully marked complete"
```

### Phase 2: Next Task Discovery

**Task Discovery Algorithm**:
```yaml
templates:
- name: find-next-task
  script:
    image: alpine:latest
    command: [sh]
    source: |
      set -e
      cd /workspace/src
      
      echo "Discovering next pending task..."
      
      # Find next task using natural version sort
      NEXT_TASK_PATH=$(find docs/.taskmaster/docs/ \
        -maxdepth 1 \
        -name "task-*" \
        -type d \
        | grep -v ".completed" \
        | sort -V \
        | head -1)
      
      if [ -n "$NEXT_TASK_PATH" ]; then
        NEXT_TASK_ID=$(echo "$NEXT_TASK_PATH" | grep -o 'task-[0-9]*' | cut -d'-' -f2)
        
        # Validate task structure
        if [ -f "$NEXT_TASK_PATH/task.txt" ] && grep -q "^# Task ID: $NEXT_TASK_ID$" "$NEXT_TASK_PATH/task.txt"; then
          echo "Next task found: $NEXT_TASK_ID"
          echo "$NEXT_TASK_ID" > /tmp/next-task-id
        else
          echo "Task $NEXT_TASK_ID has invalid structure - skipping"
          echo "" > /tmp/next-task-id
        fi
      else
        echo "No more pending tasks found"
        echo "" > /tmp/next-task-id
      fi
  outputs:
    parameters:
    - name: next-task-id
      valueFrom:
        path: /tmp/next-task-id
```

### Phase 3: Workflow Loop Logic

**Recursive Workflow Pattern**:
```yaml
templates:
- name: task-completion-and-progression
  inputs:
    parameters:
    - name: current-task-id
  steps:
  - - name: mark-current-complete
      template: mark-task-complete
      arguments:
        parameters:
        - name: task-id
          value: "{{inputs.parameters.current-task-id}}"
          
  - - name: discover-next-task
      template: find-next-task
      
  - - name: start-next-workflow
      when: "{{steps.discover-next-task.outputs.parameters.next-task-id}} != ''"
      template: trigger-next-task-workflow
      arguments:
        parameters:
        - name: task-id
          value: "{{steps.discover-next-task.outputs.parameters.next-task-id}}"
          
  - - name: finalize-queue-processing
      when: "{{steps.discover-next-task.outputs.parameters.next-task-id}} == ''"
      template: handle-queue-complete
      
- name: trigger-next-task-workflow
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
        labels:
          workflow-type: play-orchestration
          task-id: "{{inputs.parameters.task-id}}"
          triggered-by: task-progression
      spec:
        arguments:
          parameters:
          - name: task-id
            value: "{{inputs.parameters.task-id}}"
        workflowTemplateRef:
          name: play-workflow-template
```

### Phase 4: Edge Case Handling

**Comprehensive Error Handling**:
```yaml
templates:
- name: handle-queue-complete
  script:
    image: alpine/git:latest
    command: [sh]
    source: |
      cd /workspace/src
      
      echo "All tasks in queue have been processed"
      
      # Count completed tasks
      COMPLETED_COUNT=$(ls docs/.taskmaster/docs/.completed/ | grep -c '^task-' || echo "0")
      
      # Create completion marker
      cat > docs/.taskmaster/queue-complete.json << EOF
      {
        "status": "complete",
        "completedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "totalTasksCompleted": $COMPLETED_COUNT,
        "queueProcessingComplete": true
      }
      EOF
      
      # Commit completion state
      git add docs/.taskmaster/queue-complete.json
      git commit -m "Task queue processing complete - $COMPLETED_COUNT tasks processed"
      
      echo "Task queue processing successfully completed"

- name: handle-corrupted-task
  inputs:
    parameters:
    - name: task-id
  script:
    image: alpine/git:latest
    command: [sh]
    source: |
      cd /workspace/src
      TASK_ID="{{inputs.parameters.task-id}}"
      
      echo "Handling corrupted task: $TASK_ID"
      
      # Create quarantine directory
      mkdir -p "docs/.taskmaster/docs/.corrupted"
      
      # Move corrupted task
      mv "docs/.taskmaster/docs/task-$TASK_ID" "docs/.taskmaster/docs/.corrupted/task-$TASK_ID"
      
      # Log corruption
      echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): Task $TASK_ID quarantined due to invalid structure" >> docs/.taskmaster/task-errors.log
      
      # Commit quarantine
      git add -A
      git commit -m "Quarantine corrupted task $TASK_ID"
      
      echo "Corrupted task $TASK_ID quarantined"
```

## Critical Success Criteria

### 1. Functional Requirements
- [ ] Completed tasks automatically moved to `.completed/` directory
- [ ] Next pending task discovered using proper numerical sorting
- [ ] New workflow triggered automatically for next task
- [ ] Process continues until no more tasks remain
- [ ] Queue completion marked with status file

### 2. Edge Case Handling
- [ ] Corrupted tasks moved to `.corrupted/` directory
- [ ] No more tasks scenario handled gracefully
- [ ] Workflow failures don't break progression chain
- [ ] Proper logging for all progression events

### 3. Integration Requirements
- [ ] Works with existing multi-agent workflow (Rex → Cleo → Tess)
- [ ] Maintains event correlation for GitHub webhooks
- [ ] Preserves task labeling and branch naming conventions
- [ ] No impact on individual workflow execution

### 4. Operational Requirements
- [ ] Resource limits prevent infinite loops
- [ ] Workflow cleanup prevents resource exhaustion
- [ ] Comprehensive logging for troubleshooting
- [ ] Monitoring integration for progression tracking

## Implementation Strategy

### Step 1: Workflow Template Integration
```yaml
# Add task progression steps to existing play-workflow-template
spec:
  templates:
  - name: main
    dag:
      tasks:
      # ... existing workflow steps ...
      
      - name: wait-pr-approved
        dependencies: [tess-testing]
        template: suspend-for-webhook
        
      - name: complete-and-progress  # NEW STEP
        dependencies: [wait-pr-approved]
        template: task-completion-and-progression
        arguments:
          parameters:
          - name: current-task-id
            value: "{{workflow.parameters.task-id}}"
```

### Step 2: Task Discovery Testing
```bash
# Test task discovery algorithm
cd docs/.taskmaster/docs/

# Create test task structure
mkdir -p task-2 task-3 task-10 .completed/task-1
touch task-2/task.txt task-3/task.txt task-10/task.txt

# Test discovery script
find . -maxdepth 1 -name "task-*" -type d | grep -v ".completed" | sort -V | head -1
# Expected result: ./task-2
```

### Step 3: Recursive Workflow Testing
```bash
# Submit initial workflow
argo submit play-workflow-template.yaml -p task-id=5

# Monitor workflow progression
argo list -l workflow-type=play-orchestration

# Check for automatic next workflow creation
argo get $(argo list -l task-id=6 -o name)
```

### Step 4: End-to-End Validation
```bash
# Prepare test task sequence
ls docs/.taskmaster/docs/
# Should show: task-5/ task-6/ task-7/

# Start progression with task-5
argo submit play-workflow-template.yaml -p task-id=5

# Wait for complete progression
watch argo list -l workflow-type=play-orchestration

# Verify final state
ls docs/.taskmaster/docs/.completed/
# Should show: task-5/ task-6/ task-7/

cat docs/.taskmaster/queue-complete.json
# Should show completion status
```

## Key Files to Create/Modify

### New Workflow Templates
- `argo-workflows/templates/task-progression.yaml` - Task progression logic
- `argo-workflows/templates/task-discovery.yaml` - Next task discovery
- `argo-workflows/templates/queue-management.yaml` - Queue completion handling

### Modified Templates
- `argo-workflows/play-workflow-template.yaml` - Add progression step
- `argo-workflows/sensors/github-events.yaml` - Ensure event correlation works with progression

### Test Resources
- `test/task-progression-test.yaml` - End-to-end progression test
- `test/edge-case-scenarios.yaml` - Corrupted task and error handling tests

## Error Handling Scenarios

### Scenario 1: Corrupted Task Structure
```bash
# Task missing task.txt or invalid format
if [ ! -f "$TASK_PATH/task.txt" ]; then
  echo "Task missing required task.txt file"
  # Move to .corrupted/ directory
  # Continue with next task
fi
```

### Scenario 2: Workflow Creation Failure
```yaml
# Retry policy for workflow creation
retryPolicy:
  limit: 3
  retryPolicy: Always
  backoff:
    duration: "30s"
    factor: 2
```

### Scenario 3: Git Operation Failures
```bash
# Handle git commit failures gracefully
if ! git commit -m "Task completion"; then
  echo "Git commit failed - manual intervention required"
  # Log failure but don't stop progression
fi
```

## Testing Commands

### Task Completion Testing
```bash
# Test task completion logic
argo submit -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: test-task-completion-
spec:
  entrypoint: test-completion
  arguments:
    parameters:
    - name: task-id
      value: "99"
  templates:
  - name: test-completion
    template: mark-task-complete
    arguments:
      parameters:
      - name: task-id
        value: "{{workflow.parameters.task-id}}"
EOF

# Verify task moved to .completed
ls docs/.taskmaster/docs/.completed/task-99/
```

### Next Task Discovery Testing
```bash
# Test discovery algorithm
argo submit -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: test-task-discovery-
spec:
  entrypoint: test-discovery
  templates:
  - name: test-discovery
    template: find-next-task
EOF

# Check discovery results
argo logs $(argo list -o name | head -1)
```

## Expected Deliverables

1. **Task Progression Workflow Templates**: Complete Argo workflow templates for automatic task progression
2. **Task Discovery Algorithm**: Robust next task discovery with validation and error handling
3. **Workflow Loop Implementation**: Recursive workflow pattern for continuous processing
4. **Edge Case Handling**: Comprehensive error handling for all failure scenarios
5. **Integration with Existing System**: Seamless integration with multi-agent orchestration
6. **Testing Suite**: Complete test cases for all progression scenarios

## Dependencies & Prerequisites

- **Task 3**: Multi-agent orchestration system operational
- **Task 7**: Event-driven workflow coordination functional
- **Argo Workflows**: Recursive template support and DAG execution
- **Git Repository**: Write access for task directory operations
- **Kubernetes Resources**: Sufficient resources for workflow chaining

## Constraints

- **Resource Limits**: Prevent infinite workflow loops consuming cluster resources
- **Sequential Processing**: One task progression chain at a time to avoid conflicts
- **Atomic Operations**: Task movement and workflow creation must be atomic
- **Backward Compatibility**: Don't break existing single-task workflow execution

## Quality Gates

Before marking complete:
- [ ] Task completion moves directory correctly
- [ ] Next task discovery finds correct task in sequence
- [ ] Workflow loop creates new workflows automatically
- [ ] Queue completion handled gracefully
- [ ] All edge cases tested and handled
- [ ] No resource leaks or infinite loops
- [ ] Integration with multi-agent system verified
- [ ] End-to-end task sequence processes successfully

This implementation establishes automatic task queue processing that enables continuous multi-agent workflow execution without manual intervention, dramatically improving the system's automation capabilities.