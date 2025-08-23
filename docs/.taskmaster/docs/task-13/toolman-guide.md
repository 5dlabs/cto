# Toolman Guide: Implement Task Progression Logic

## Overview

This guide provides comprehensive instructions for implementing automatic task progression logic in the multi-agent orchestration system. This system automatically moves completed tasks to `.completed` directories, discovers next pending tasks, and triggers continuous workflow processing.

## Tool Recommendations

### Primary Tools for Task Progression Implementation

#### 1. Workflow Management
- **brave-search_brave_web_search**: Research Argo Workflows patterns and best practices
- **memory_create_entities**: Store workflow templates and progression patterns
- **memory_query_entities**: Retrieve stored progression logic and troubleshooting knowledge

#### 2. System Integration (if available)
- **mcp-argo**: Direct Argo Workflows interaction for testing and validation
- **kubernetes_***: Kubernetes resource management for workflow monitoring

### Tool Usage Patterns

#### Phase 1: Research and Design

```bash
# Use brave-search_brave_web_search for research
Search: "argo workflows recursive templates task processing"
Search: "kubernetes workflow automation continuous processing"
Search: "argo workflows DAG conditional steps best practices"
Search: "workflow loop prevention resource limits"
```

#### Phase 2: Implementation and Testing

```bash
# Use memory_create_entities to store successful patterns
Store: Task completion workflow template patterns
Store: Next task discovery algorithm implementations
Store: Error handling strategies for corrupted tasks
Store: Testing procedures and validation scripts

# Use memory_query_entities to retrieve stored knowledge
Query: "task progression workflow templates"
Query: "next task discovery algorithms"
Query: "error handling patterns"
```

#### Phase 3: Validation and Monitoring

```bash
# If mcp-argo available, use for direct workflow interaction
mcp-argo list-workflows --label="workflow-type=play-orchestration"
mcp-argo get-workflow <workflow-name>
mcp-argo submit-workflow task-progression-template.yaml
```

## Best Practices

### 1. Workflow Template Design

**Task Completion Template Pattern**:
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
      set -e  # Exit on any error

      TASK_ID="{{inputs.parameters.task-id}}"
      SOURCE_DIR="docs/.taskmaster/docs/task-$TASK_ID"
      TARGET_DIR="docs/.taskmaster/docs/.completed/task-$TASK_ID"

      # Validation before move
      if [ ! -d "$SOURCE_DIR" ]; then
        echo "ERROR: Source directory $SOURCE_DIR not found"
        exit 1
      fi

      # Atomic directory operation
      echo "Moving task $TASK_ID to completed directory"
      mkdir -p "docs/.taskmaster/docs/.completed"
      mv "$SOURCE_DIR" "$TARGET_DIR"

      # Git commit with error handling
      if git add -A && git commit -m "Task $TASK_ID completed - moved to .completed directory"; then
        echo "Task $TASK_ID successfully committed"
      else
        echo "WARNING: Git commit failed but task move succeeded"
      fi
```

### 2. Task Discovery Algorithm

**Robust Discovery with Validation**:
```yaml
templates:
- name: find-next-task
  script:
    image: alpine:latest
    command: [sh]
    source: |
      set -e

      echo "Discovering next pending task..."

      # Find next task with natural version sort
      NEXT_TASK_PATH=$(find docs/.taskmaster/docs/ \
        -maxdepth 1 \
        -name "task-*" \
        -type d \
        | grep -v ".completed" \
        | sort -V \
        | head -1)

      if [ -n "$NEXT_TASK_PATH" ]; then
        NEXT_TASK_ID=$(echo "$NEXT_TASK_PATH" | grep -o 'task-[0-9]*' | cut -d'-' -f2)

        # Comprehensive task validation
        validate_task() {
          local task_path="$1"
          local task_id="$2"

          # Check task.txt exists
          if [ ! -f "$task_path/task.txt" ]; then
            echo "Task $task_id missing task.txt file"
            return 1
          fi

          # Check task.txt format
          if ! grep -q "^# Task ID: $task_id$" "$task_path/task.txt"; then
            echo "Task $task_id has invalid task.txt format"
            return 1
          fi

          # Check task status
          if grep -q "^# Status: completed$" "$task_path/task.txt"; then
            echo "Task $task_id already marked as completed"
            return 1
          fi

          return 0
        }

        if validate_task "$NEXT_TASK_PATH" "$NEXT_TASK_ID"; then
          echo "Next task found: $NEXT_TASK_ID"
          echo "$NEXT_TASK_ID" > /tmp/next-task-id
        else
          echo "Task $NEXT_TASK_ID failed validation - quarantining"
          mkdir -p "docs/.taskmaster/docs/.corrupted"
          mv "$NEXT_TASK_PATH" "docs/.taskmaster/docs/.corrupted/task-$NEXT_TASK_ID"
          echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): Task $NEXT_TASK_ID quarantined" >> docs/.taskmaster/task-errors.log

          # Recursively find next valid task
          find_next_task  # This would need to be implemented as a loop
        fi
      else
        echo "No more pending tasks found - queue complete"
        echo "" > /tmp/next-task-id
      fi
  outputs:
    parameters:
    - name: next-task-id
      valueFrom:
        path: /tmp/next-task-id
```

### 3. Workflow Loop Implementation

**Safe Recursive Pattern**:
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

# Safe workflow creation with resource limits
- name: trigger-next-task-workflow
  inputs:
    parameters:
    - name: task-id
  resource:
    action: create
    successCondition: status.phase == Succeeded
    failureCondition: status.phase == Failed
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
          parent-workflow: "{{workflow.name}}"
      spec:
        # Resource limits to prevent runaway workflows
        activeDeadlineSeconds: 86400  # 24 hours max
        parallelism: 1

        # Cleanup policy
        ttlStrategy:
          secondsAfterCompletion: 3600  # Clean up after 1 hour

        arguments:
          parameters:
          - name: task-id
            value: "{{inputs.parameters.task-id}}"
        workflowTemplateRef:
          name: play-workflow-template
```

### 4. Error Handling and Recovery

**Comprehensive Error Management**:
```yaml
templates:
- name: handle-corrupted-task
  inputs:
    parameters:
    - name: task-id
    - name: error-reason
  script:
    image: alpine/git:latest
    command: [sh]
    source: |
      TASK_ID="{{inputs.parameters.task-id}}"
      ERROR_REASON="{{inputs.parameters.error-reason}}"

      echo "Handling corrupted task: $TASK_ID (Reason: $ERROR_REASON)"

      # Create quarantine structure
      QUARANTINE_DIR="docs/.taskmaster/docs/.corrupted/task-$TASK_ID"
      mkdir -p "docs/.taskmaster/docs/.corrupted"

      # Move corrupted task
      if [ -d "docs/.taskmaster/docs/task-$TASK_ID" ]; then
        mv "docs/.taskmaster/docs/task-$TASK_ID" "$QUARANTINE_DIR"

        # Add metadata about corruption
        cat > "$QUARANTINE_DIR/.corruption-info" <<EOF
      {
        "taskId": "$TASK_ID",
        "quarantinedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "reason": "$ERROR_REASON",
        "workflow": "{{workflow.name}}"
      }
      EOF
      fi

      # Log corruption event
      echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): Task $TASK_ID quarantined - $ERROR_REASON" >> docs/.taskmaster/task-errors.log

      # Commit quarantine
      git add -A
      git commit -m "Quarantine corrupted task $TASK_ID: $ERROR_REASON"

      echo "Task $TASK_ID successfully quarantined"
```

## Common Workflows

### Workflow 1: Complete Task Progression Implementation

1. **Research and Design Phase**
   ```bash
   # Use brave-search_brave_web_search
   Search: "argo workflows recursive template best practices"
   Search: "kubernetes workflow automation patterns"
   Search: "workflow resource limits prevention infinite loops"

   # Store research findings
   memory_create_entities("Research", {
     "topic": "task-progression-patterns",
     "findings": "Argo workflows support recursive patterns with proper resource limits",
     "best_practices": ["Use activeDeadlineSeconds", "Implement ttlStrategy", "Add parallelism limits"]
   })
   ```

2. **Template Development Phase**
   ```yaml
   # Create comprehensive workflow template
   # infra/argo-workflows/templates/task-progression.yaml

   apiVersion: argoproj.io/v1alpha1
   kind: WorkflowTemplate
   metadata:
     name: task-progression-template
     namespace: argo
   spec:
     entrypoint: process-task-progression
     templates:
     # [Include all templates from best practices above]
   ```

3. **Integration with Existing Workflow**
   ```yaml
   # Modify existing play-workflow-template.yaml
   # Add progression step after multi-agent completion

   templates:
   - name: main
     dag:
       tasks:
       # ... existing tasks ...
       - name: complete-and-progress
         dependencies: [wait-pr-approved]
         template: task-completion-and-progression
         arguments:
           parameters:
           - name: current-task-id
             value: "{{workflow.parameters.task-id}}"
   ```

4. **Testing and Validation Phase**
   ```bash
   # Create test task sequence
   mkdir -p docs/.taskmaster/docs/{task-101,task-102,task-103}
   echo "# Task ID: 101" > docs/.taskmaster/docs/task-101/task.txt
   echo "# Task ID: 102" > docs/.taskmaster/docs/task-102/task.txt
   echo "# Task ID: 103" > docs/.taskmaster/docs/task-103/task.txt

   # Submit test workflow
   argo submit play-workflow-template.yaml -p task-id=101

   # Monitor progression
   watch argo list -l workflow-type=play-orchestration
   ```

### Workflow 2: Error Handling and Recovery Testing

1. **Create Test Scenarios**
   ```bash
   # Setup corrupted task structures
   mkdir -p docs/.taskmaster/docs/{task-201,task-202,task-203}
   echo "# Task ID: 201" > docs/.taskmaster/docs/task-201/task.txt  # Valid
   # task-202 missing task.txt (corrupted)
   echo "Invalid format" > docs/.taskmaster/docs/task-203/task.txt  # Invalid
   ```

2. **Test Discovery Algorithm**
   ```bash
   # Test task discovery with corruption
   argo submit -f - <<EOF
   apiVersion: argoproj.io/v1alpha1
   kind: Workflow
   metadata:
     generateName: test-discovery-
   spec:
     entrypoint: test
     templates:
     - name: test
       template: find-next-task
   EOF

   # Verify results
   argo logs $(argo list -o name | head -1)
   ```

3. **Validate Error Handling**
   ```bash
   # Check quarantine functionality
   ls docs/.taskmaster/docs/.corrupted/
   cat docs/.taskmaster/task-errors.log

   # Verify error recovery continues with valid tasks
   argo logs <workflow> | grep "Next task found: 201"
   ```

### Workflow 3: Performance and Resource Testing

1. **Large Queue Testing**
   ```bash
   # Create large task sequence
   for i in $(seq 301 350); do
     mkdir -p docs/.taskmaster/docs/task-$i
     echo "# Task ID: $i" > docs/.taskmaster/docs/task-$i/task.txt
   done

   # Submit and monitor performance
   argo submit play-workflow-template.yaml -p task-id=301

   # Monitor resource usage
   kubectl top nodes
   kubectl top pods -n argo
   ```

2. **Resource Limit Testing**
   ```bash
   # Test workflow cleanup and limits
   # Monitor workflow count over time
   watch 'argo list | wc -l'

   # Verify TTL cleanup working
   argo list --completed | grep "seconds ago"
   ```

## Troubleshooting Guide

### Issue 1: Task Not Moving to .completed Directory
**Symptoms**: Workflow completes but task remains in main directory

**Diagnosis**:
```bash
# Check workflow logs for task completion step
argo logs <workflow-name> -c mark-task-complete

# Verify directory permissions
ls -la docs/.taskmaster/docs/

# Check git status for uncommitted changes
git status
```

**Resolution**:
1. Verify script has proper permissions for directory operations
2. Check git configuration for commit operations
3. Validate task directory structure and naming
4. Ensure no file locks or permission issues

### Issue 2: Next Task Discovery Fails
**Symptoms**: No next task found despite pending tasks existing

**Diagnosis**:
```bash
# Test discovery algorithm manually
find docs/.taskmaster/docs/ -maxdepth 1 -name "task-*" -type d | grep -v ".completed" | sort -V

# Check task validation
grep "^# Task ID:" docs/.taskmaster/docs/task-*/task.txt

# Verify no hidden characters or encoding issues
hexdump -C docs/.taskmaster/docs/task-*/task.txt | head
```

**Resolution**:
1. Verify task.txt format matches expected pattern exactly
2. Check for hidden characters or encoding issues
3. Validate directory naming conventions
4. Test sort command behavior with current task numbers

### Issue 3: Infinite Workflow Creation
**Symptoms**: Continuous workflow creation without progression

**Diagnosis**:
```bash
# Check workflow creation rate
argo list | grep "play-workflow" | wc -l

# Monitor workflow creation over time
watch 'argo list | grep "Running" | wc -l'

# Check for workflow creation failures
argo logs <workflow> | grep -E '(failed|error|retry)'
```

**Resolution**:
1. Verify activeDeadlineSeconds and resource limits set
2. Check workflow creation success/failure rates
3. Implement circuit breaker pattern for repeated failures
4. Add workflow count monitoring and alerts

### Issue 4: Corrupted Task Handling Failures
**Symptoms**: System stuck on corrupted tasks without quarantine

**Diagnosis**:
```bash
# Check quarantine directory
ls -la docs/.taskmaster/docs/.corrupted/

# Verify error logging
cat docs/.taskmaster/task-errors.log

# Check validation logic
argo logs <workflow> | grep -E '(validation|corrupt|quarantine)'
```

**Resolution**:
1. Verify quarantine directory creation permissions
2. Check validation logic covers all corruption scenarios
3. Ensure error logging is working correctly
4. Test manual quarantine operations

## Tool-Specific Tips

### brave-search_brave_web_search
- Search for "argo workflows" + specific functionality needed
- Look for official Argo documentation and examples
- Research Kubernetes workflow automation best practices
- Find troubleshooting guides for common issues

### memory_create_entities / memory_query_entities
- Store successful workflow template patterns
- Document error handling strategies that work
- Keep track of testing procedures and validation scripts
- Record performance benchmarks and optimization tips

### Direct Workflow Tools (if available)
- Use workflow submission and monitoring for testing
- Implement real-time workflow status checking
- Enable detailed logging and debugging
- Monitor resource usage during progression

## Quality Checks

### Pre-Implementation Checklist
- [ ] Research completed on Argo Workflows recursive patterns
- [ ] Resource limit strategies defined
- [ ] Error handling scenarios identified
- [ ] Testing strategy planned

### Implementation Checklist
- [ ] Task completion logic implemented with validation
- [ ] Next task discovery algorithm robust and tested
- [ ] Workflow loop logic includes proper resource limits
- [ ] Error handling covers all edge cases
- [ ] Integration with existing multi-agent workflow complete

### Post-Implementation Checklist
- [ ] End-to-end task progression tested successfully
- [ ] Error scenarios handled gracefully
- [ ] Resource limits prevent infinite loops
- [ ] Performance acceptable with large task queues
- [ ] Monitoring and logging provide adequate visibility

## Success Indicators

1. **Functional Success**:
   - Tasks automatically progress without manual intervention
   - Completed tasks moved to `.completed` directory correctly
   - Queue completion handled gracefully

2. **Quality Success**:
   - All error scenarios handled without system failure
   - Resource usage remains within acceptable limits
   - No infinite loops or runaway workflows

3. **Integration Success**:
   - Multi-agent workflow compatibility maintained
   - Event correlation continues working correctly
   - GitHub integration unaffected by progression logic

## Performance Optimization

### Task Discovery Optimization
- Use efficient sorting algorithms for large task numbers
- Minimize file system operations during discovery
- Cache validation results when possible
- Implement parallel validation for multiple tasks

### Workflow Resource Optimization
- Set appropriate resource limits and deadlines
- Implement efficient cleanup policies
- Monitor and tune parallelism settings
- Use workflow priorities for important progressions

### Error Handling Optimization
- Implement fast-fail strategies for obvious corruption
- Use batch operations for multiple corrupted tasks
- Optimize logging and monitoring overhead
- Implement intelligent retry strategies

This guide provides the foundation for successfully implementing automatic task progression logic that enables continuous multi-agent workflow processing while maintaining system stability and reliability.
