# Acceptance Criteria: Implement Task Progression Logic



## Overview

This document defines the specific, testable criteria that must be met to consider Task 13 (Implement Task Progression Logic) complete. All criteria must pass before the task can be approved and merged.

## Functional Requirements

### FR-1: Task Completion Processing
**Requirement**: Completed tasks are automatically moved to .completed directory

**Test Cases**:


- [ ] Task directory successfully moved from `docs/.taskmaster/docs/task-X/` to `docs/.taskmaster/docs/.completed/task-X/`


- [ ] All files and subdirectories preserved during move operation


- [ ] `.completed` directory created if it doesn't exist


- [ ] Git commit created documenting task completion


- [ ] Original task directory no longer exists in main directory

**Verification**:



```bash


# Setup test task
mkdir -p docs/.taskmaster/docs/task-99
echo "# Task ID: 99" > docs/.taskmaster/docs/task-99/task.txt
echo "test content" > docs/.taskmaster/docs/task-99/test-file.md

# Create test workflow for task completion
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

# Wait for completion and verify
argo wait $(argo list -o name | head -1)



# Verify task moved to .completed
ls docs/.taskmaster/docs/.completed/task-99/
cat docs/.taskmaster/docs/.completed/task-99/task.txt
cat docs/.taskmaster/docs/.completed/task-99/test-file.md

# Verify original directory removed
ls docs/.taskmaster/docs/task-99/ 2>/dev/null && echo "ERROR: Original task directory still exists" || echo "PASS: Original directory removed"



# Verify git commit
git log --oneline -1 | grep "Task 99 completed"






```



### FR-2: Next Task Discovery
**Requirement**: System correctly identifies next pending task in sequence

**Test Cases**:


- [ ] Finds lowest numbered pending task using natural version sort


- [ ] Ignores tasks in `.completed` directory


- [ ] Validates task structure before selection


- [ ] Handles gaps in task numbering correctly


- [ ] Returns empty result when no more tasks exist

**Verification**:



```bash
# Setup test task sequence with gaps
mkdir -p docs/.taskmaster/docs/{task-2,task-5,task-10,task-15}
mkdir -p docs/.taskmaster/docs/.completed/task-1
echo "# Task ID: 2" > docs/.taskmaster/docs/task-2/task.txt
echo "# Task ID: 5" > docs/.taskmaster/docs/task-5/task.txt
echo "# Task ID: 10" > docs/.taskmaster/docs/task-10/task.txt
echo "# Task ID: 15" > docs/.taskmaster/docs/task-15/task.txt



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

# Wait and check results
argo wait $(argo list -o name | head -1)
argo logs $(argo list -o name | head -1) | grep "Next task found: 2"



# Test with task-2 completed
mv docs/.taskmaster/docs/task-2 docs/.taskmaster/docs/.completed/task-2

# Run discovery again
argo submit -f test-discovery-workflow.yaml
argo wait $(argo list -o name | head -1)
argo logs $(argo list -o name | head -1) | grep "Next task found: 5"

# Test empty queue scenario
mv docs/.taskmaster/docs/task-{5,10,15} docs/.taskmaster/docs/.completed/

argo submit -f test-discovery-workflow.yaml
argo wait $(argo list -o name | head -1)
argo logs $(argo list -o name | head -1) | grep "No more pending tasks found"






```

### FR-3: Workflow Loop Implementation
**Requirement**: Automatic workflow creation for next task in sequence

**Test Cases**:


- [ ] New workflow created when next task exists


- [ ] New workflow has correct task-id parameter


- [ ] New workflow uses correct workflow template reference


- [ ] Workflow labels set correctly for event correlation


- [ ] No workflow created when queue is empty

**Verification**:



```bash
# Setup test sequence
mkdir -p docs/.taskmaster/docs/{task-20,task-21}
echo "# Task ID: 20" > docs/.taskmaster/docs/task-20/task.txt
echo "# Task ID: 21" > docs/.taskmaster/docs/task-21/task.txt

# Submit workflow for task-20 with progression enabled
argo submit play-workflow-template.yaml -p task-id=20

# Wait for task-20 workflow to reach completion step
# (This requires full multi-agent workflow, so we'll simulate)

# Manually trigger progression step to test
argo submit -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: test-progression-
spec:
  entrypoint: test-loop
  templates:
  - name: test-loop
    template: task-completion-and-progression
    arguments:
      parameters:
      - name: current-task-id
        value: "20"
EOF

# Wait and verify new workflow created
argo wait $(argo list -o name | head -1)

# Check for new workflow with task-21
argo list -l task-id=21
argo list -l task-id=21 | grep "play-workflow"



# Verify task-20 moved to completed
ls docs/.taskmaster/docs/.completed/task-20/
ls docs/.taskmaster/docs/task-21/  # Should still exist as active task






```

### FR-4: Queue Completion Handling
**Requirement**: System gracefully handles empty task queue

**Test Cases**:


- [ ] Queue completion status file created


- [ ] Completion timestamp recorded


- [ ] Total completed tasks count accurate


- [ ] Git commit created for queue completion


- [ ] No attempt to create new workflow when queue empty

**Verification**:



```bash
# Ensure only one task remains
rm -rf docs/.taskmaster/docs/task-*
mkdir -p docs/.taskmaster/docs/task-99
echo "# Task ID: 99" > docs/.taskmaster/docs/task-99/task.txt

# Add some completed tasks for counting
mkdir -p docs/.taskmaster/docs/.completed/{task-1,task-2,task-3}

# Run progression with last task
argo submit -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: test-queue-completion-
spec:
  entrypoint: test-completion
  templates:
  - name: test-completion
    template: task-completion-and-progression
    arguments:
      parameters:
      - name: current-task-id
        value: "99"
EOF

# Wait for completion
argo wait $(argo list -o name | head -1)

# Verify completion status file
cat docs/.taskmaster/queue-complete.json
jq '.status' docs/.taskmaster/queue-complete.json | grep "complete"
jq '.totalTasksCompleted' docs/.taskmaster/queue-complete.json | grep "4"  # 3 existing + 1 just completed



# Verify git commit
git log --oneline -1 | grep "queue processing complete"

# Verify no new workflow created
argo list | wc -l  # Should not increase beyond expected count






```

## Error Handling Requirements

### EH-1: Corrupted Task Handling
**Requirement**: Invalid task structures are quarantined without breaking progression

**Test Cases**:


- [ ] Tasks missing task.txt moved to .corrupted directory


- [ ] Tasks with invalid task.txt format quarantined


- [ ] Corruption logged to task-errors.log


- [ ] Progression continues with next valid task


- [ ] Git commit records quarantine action

**Verification**:



```bash


# Create corrupted task structure
mkdir -p docs/.taskmaster/docs/{task-30,task-31,task-32}
echo "# Task ID: 30" > docs/.taskmaster/docs/task-30/task.txt  # Valid
# task-31 missing task.txt (corrupted)
echo "Invalid format" > docs/.taskmaster/docs/task-32/task.txt  # Invalid format



# Test discovery with corrupted tasks
argo submit -f test-discovery-workflow.yaml
argo wait $(argo list -o name | head -1)

# Verify valid task found despite corruption
argo logs $(argo list -o name | head -1) | grep "Next task found: 30"

# Test corruption handling during progression
# (This would require full workflow simulation)

# Verify corrupted tasks quarantined
ls docs/.taskmaster/docs/.corrupted/

# Verify error logging
cat docs/.taskmaster/task-errors.log | grep "quarantined due to invalid structure"



# Verify git commit
git log --oneline | grep "quarantine"






```

### EH-2: Workflow Creation Failures
**Requirement**: Failed workflow creation doesn't break progression chain

**Test Cases**:


- [ ] Retry logic attempts workflow creation multiple times


- [ ] Failure logged with details for debugging


- [ ] System continues with manual intervention possible


- [ ] Resource exhaustion doesn't cause infinite retries

**Verification**:



```bash
# This requires testing against resource constraints or RBAC restrictions
# Simulate by creating workflow template with invalid reference

# Test retry logic (implementation dependent)
# Monitor logs for retry attempts and eventual failure handling

# Verify failure logging
argo logs <failed-workflow> | grep -E '(retry|attempt|failed)'

# Verify system doesn't create infinite retry loops
# Monitor resource usage and workflow count over time






```

### EH-3: Git Operation Failures
**Requirement**: Git operation failures don't prevent task progression

**Test Cases**:


- [ ] Git commit failures logged but don't stop progression


- [ ] Directory operations complete even if git fails


- [ ] Manual git recovery possible after automation failure


- [ ] Progress continues with next task despite git issues

**Verification**:



```bash
# Simulate git failure by setting invalid git config
git config user.email ""
git config user.name ""

# Run task completion workflow
argo submit -f test-task-completion.yaml



# Verify task still moved despite git failure
ls docs/.taskmaster/docs/.completed/

# Check for git error logging
argo logs $(argo list -o name | head -1) | grep -E '(git.*failed|commit.*failed)'

# Restore git config
git config user.email "test@example.com"
git config user.name "Test User"






```

## Performance Requirements

### PR-1: Task Discovery Performance
**Requirement**: Next task discovery completes within reasonable time limits

**Test Cases**:


- [ ] Discovery completes within 30 seconds for up to 100 tasks


- [ ] Memory usage remains constant regardless of task count


- [ ] Sorting algorithm handles large task numbers efficiently


- [ ] No performance degradation with deep .completed directory

**Verification**:



```bash
# Create large task set for performance testing
for i in $(seq 1 100); do
  mkdir -p docs/.taskmaster/docs/task-$i
  echo "# Task ID: $i" > docs/.taskmaster/docs/task-$i/task.txt
done

# Move some to completed to test filtering
for i in $(seq 1 50); do
  mkdir -p docs/.taskmaster/docs/.completed/task-$i
done

# Time discovery operation
time argo submit -f test-discovery-workflow.yaml
argo wait $(argo list -o name | head -1)

# Verify discovery time in workflow logs
argo logs $(argo list -o name | head -1) | grep -E '(duration|elapsed)'

# Check memory usage doesn't spike
# Monitor during execution: kubectl top pods -n argo






```

### PR-2: Workflow Creation Performance
**Requirement**: New workflow creation doesn't significantly delay progression

**Test Cases**:


- [ ] Workflow creation completes within 10 seconds


- [ ] No resource exhaustion from rapid workflow creation


- [ ] Workflow templates render efficiently


- [ ] Event correlation setup doesn't cause delays

**Verification**:



```bash
# Test rapid workflow creation
for i in $(seq 1 5); do
  argo submit play-workflow-template.yaml -p task-id=$i &
done
wait



# Verify all workflows created successfully
argo list | grep play-workflow | wc -l  # Should be 5

# Check creation times
argo list -o wide | grep Created

# Monitor resource usage during creation
kubectl top nodes
kubectl top pods -n argo






```

## Integration Requirements

### IR-1: Multi-Agent Workflow Compatibility
**Requirement**: Task progression integrates seamlessly with existing multi-agent workflow

**Test Cases**:


- [ ] Rex → Cleo → Tess → Human approval flow preserved


- [ ] Event correlation continues working with progressive workflows


- [ ] PR labeling and branch naming conventions maintained


- [ ] GitHub webhook processing unaffected by progression

**Verification**:



```bash
# Create test task sequence
mkdir -p docs/.taskmaster/docs/{task-40,task-41}
echo "# Task ID: 40" > docs/.taskmaster/docs/task-40/task.txt
echo "# Task ID: 41" > docs/.taskmaster/docs/task-41/task.txt

# Submit full multi-agent workflow
argo submit play-workflow-template.yaml -p task-id=40

# Verify workflow progresses through all agents
# Monitor workflow status through each stage
argo get $(argo list -l task-id=40 -o name)

# Check for proper event correlation
# Verify PR labels include task-40
# Verify branch naming includes task-40

# Monitor automatic progression to task-41


# After task-40 completes, verify task-41 workflow starts
argo list -l task-id=41



# Verify task-40 moved to completed
ls docs/.taskmaster/docs/.completed/task-40/






```

### IR-2: GitHub Event Correlation
**Requirement**: GitHub webhooks correctly correlate with progressive workflows

**Test Cases**:


- [ ] PR creation events resume correct progressive workflow


- [ ] PR labeling events target appropriate workflow instance


- [ ] PR approval events trigger progression correctly


- [ ] Multiple progressive workflows don't interfere

**Verification**:



```bash
# This requires actual GitHub integration testing
# Create PR with task-X label and verify correct workflow resumes

# Monitor Argo Events sensor processing
kubectl logs -n argo-events -l sensor-name=github-webhook-sensor

# Verify event correlation in workflow logs
argo logs <workflow-name> | grep -E '(event|webhook|correlation)'

# Check workflow label matching
argo list -l task-id=X,current-stage=waiting-pr-created






```

## Resource Management Requirements



### RM-1: Workflow Resource Limits
**Requirement**: Progressive workflows don't exhaust cluster resources

**Test Cases**:


- [ ] Maximum workflow count limits enforced


- [ ] Workflow cleanup after completion


- [ ] Resource quotas respected


- [ ] No memory leaks from long-running progression chains

**Verification**:



```bash
# Check workflow resource limits in templates
grep -A 10 "activeDeadlineSeconds" play-workflow-template.yaml
grep -A 10 "ttlStrategy" play-workflow-template.yaml

# Monitor resource usage during progression
kubectl top nodes
kubectl top pods -n argo

# Verify workflow cleanup
argo list --completed | wc -l
# Should not continuously grow without cleanup



# Check for resource quotas
kubectl describe resourcequota -n argo






```

### RM-2: Infinite Loop Prevention
**Requirement**: System prevents infinite workflow creation loops

**Test Cases**:


- [ ] Maximum workflow chain length enforced


- [ ] Circuit breaker pattern for repeated failures


- [ ] Deadlock detection and recovery


- [ ] Resource exhaustion alerts

**Verification**:



```bash
# Create intentional loop condition (invalid next task)
# Monitor system behavior and recovery

# Check for loop prevention logic in templates
grep -A 5 "activeDeadlineSeconds" play-workflow-template.yaml

# Verify circuit breaker implementation
# Look for retry limits and backoff strategies
grep -A 10 "retryPolicy" play-workflow-template.yaml

# Monitor for stuck workflows
argo list | grep Running | wc -l
# Should not continuously increase






```

## Monitoring and Observability Requirements

### MO-1: Task Progression Logging
**Requirement**: All task progression events are logged for monitoring

**Test Cases**:


- [ ] Task completion events logged with timestamps


- [ ] Next task discovery results logged


- [ ] Workflow creation events logged


- [ ] Error conditions logged with details


- [ ] Structured logging format for parsing

**Verification**:



```bash
# Run progression and check logs
argo submit -f test-progression-workflow.yaml
argo wait $(argo list -o name | head -1)

# Verify progression events in logs
argo logs $(argo list -o name | head -1) | grep -E '(TASK_PROGRESSION|Moving task|Next task found|Workflow created)'



# Check log format structure
argo logs $(argo list -o name | head -1) | grep '\[TASK_PROGRESSION\]'

# Verify error logging
argo logs $(argo list -o name | head -1) | grep -E '(ERROR|WARN)'






```

### MO-2: Progression Metrics
**Requirement**: Task progression metrics available for monitoring

**Test Cases**:


- [ ] Task completion rate tracked


- [ ] Queue length metrics available


- [ ] Progression failure rate tracked


- [ ] Average task processing time recorded

**Verification**:



```bash
# Check for metrics endpoints in workflow pods
kubectl get pods -l app=argo-server -o yaml | grep -i metrics

# Look for progression-specific metrics
# This depends on monitoring setup (Prometheus, etc.)
curl http://argo-server/metrics | grep task_progression

# Verify custom metrics in workflow logs
argo logs <workflow> | grep -E '(metric|duration|count)'






```

## Final Validation Checklist

Before considering Task 13 complete:



- [ ] All functional requirements (FR-1 through FR-4) pass


- [ ] All error handling requirements (EH-1 through EH-3) pass


- [ ] All performance requirements (PR-1 through PR-2) pass


- [ ] All integration requirements (IR-1 through IR-2) pass


- [ ] All resource management requirements (RM-1 through RM-2) pass


- [ ] All monitoring requirements (MO-1 through MO-2) pass


- [ ] End-to-end task sequence processes automatically


- [ ] Edge cases handled gracefully


- [ ] No resource leaks or infinite loops


- [ ] Code review completed and approved


- [ ] Documentation updated and reviewed


- [ ] Changes tested in isolated environment


- [ ] Ready for production deployment



## Success Metrics



1. **100% test case pass rate** - All test cases must pass


2. **Zero infinite loops** - No workflow resource exhaustion


3. **<30 second task discovery** - Fast next task identification


4. **<10 second workflow creation** - Efficient progression


5. **Zero data loss** - All task files preserved during moves


6. **100% error handling** - All edge cases handled gracefully

## Post-Deployment Monitoring

After Task 13 completion, monitor these key indicators:



- **Task progression success rate** - Percentage of successful progressions


- **Queue processing time** - Time to complete entire task queue


- **Workflow chain length** - Number of consecutive automated progressions


- **Error frequency** - Rate of corruption and failure handling


- **Resource utilization** - CPU/Memory usage during progression


- **Manual intervention rate** - Frequency of required human intervention

When all acceptance criteria are met, Task 13 successfully implements automatic task progression that enables continuous multi-agent workflow processing without manual intervention.