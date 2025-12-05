# Parallel Task Execution - Implementation Plan

## Executive Summary

**Goal**: Reduce project completion time by executing independent tasks in parallel  
**Current State**: All tasks execute sequentially, one at a time  
**Target**: 2-5x speedup for projects with independent tasks  
**Risk Level**: Medium (requires careful PR coordination)

## Current Architecture Analysis

### Sequential Execution Flow
```
TaskMaster generates:          Play-Project Workflow:           Per-Task Execution:
├─ task-1 (deps: [])          ┌───────────────────┐           ┌─────────────┐
├─ task-2 (deps: [1])    ────→│ Discover Tasks    │      ┌───→│ Rex: Code   │
├─ task-3 (deps: [])          │  task-1, 2, 3, 4  │      │    │ Cleo: Review│
└─ task-4 (deps: [1,2])       └─────────┬─────────┘      │    │ Cipher: Sec │
                                        │                 │    │ Tess: Test  │
                               ┌────────▼────────┐        │    └──────┬──────┘
                               │  FOR task IN    │───┐    │           │
                               │  1,2,3,4        │   │    │           │
                               └─────────────────┘   │    │      Wait for PR
                                                     │    │      merge to main
                                                     └────┘           │
                                    SEQUENTIAL                 ┌──────▼──────┐
                                    2-8 hrs/task               │  Continue   │
                                    Total: 8-32hrs             └─────────────┘
```

### Key Files

- `infra/charts/controller/templates/workflowtemplates/play-project-workflow-template.yaml`
  - Line 90-98: **Sequential task processor** - main loop
  - Line 291-403: FOR loop that waits for each task completion
  
- TaskMaster Integration:
  - Line 221: Looks for `.taskmaster/docs/` directory
  - Line 232: Discovers `task-*` directories
  - **Note**: TaskMaster `tasks.json` has `dependencies` field - currently unused!

## Proposed Parallel Architecture

### Phase 1: Conservative Parallel Groups

**Strategy**: Group tasks by dependency level, parallelize within levels

```
Level 0 (no deps):       Level 1 (deps on L0):     Level 2 (deps on L1):
┌────────────┐           ┌────────────┐             ┌────────────┐
│  Task 1    │           │  Task 2    │             │  Task 4    │
│ (Frontend) │──┐    ┌───│ (API)      │          ┌──│ (Deploy)   │
└────────────┘  │    │   └────────────┘          │  └────────────┘
                │    │                            │
┌────────────┐  │    │                            │
│  Task 3    │──┤    └────────────────────────────┘
│ (Docs)     │  │
└────────────┘  │
                │
    Parallel    │         Sequential wait
   (2-4 hrs)    │            (2-4 hrs)
                │
```

### Implementation Approach

#### 1. Dependency Graph Construction

**Input**: TaskMaster `tasks.json`
```json
[
  {"id": 1, "title": "Create Frontend", "dependencies": []},
  {"id": 2, "title": "Build API", "dependencies": [1]},
  {"id": 3, "title": "Write Docs", "dependencies": []},
  {"id": 4, "title": "Deploy", "dependencies": [1, 2]}
]
```

**Output**: Execution Levels
```
Level 0: [1, 3]      # Can run in parallel
Level 1: [2]         # Waits for task 1
Level 2: [4]         # Waits for tasks 1, 2
```

#### 2. Parallel Workflow Template

Create new template: `parallel-task-processor`

```yaml
- name: parallel-task-processor
  inputs:
    parameters:
      - name: task-levels  # JSON: [[1,3], [2], [4]]
      - name: tasks-metadata  # Full task info with deps
  steps:
    {{- range $level_idx, $level := .taskLevels }}
    # Execute level {{ $level_idx }} in parallel
    - - name: level-{{ $level_idx }}
        parallelism: {{ len $level }}
        withParam: "{{ $level }}"
        template: execute-task
        arguments:
          parameters:
            - name: task-id
              value: "{{`{{item}}`}}"
    
    # Wait for level completion before next level
    {{- end }}
```

#### 3. PR Coordination Strategy

**Challenge**: Multiple tasks creating PRs simultaneously  
**Solutions**:

**Option A - Single PR Approach** (Recommended for Phase 1)
- Create ONE feature branch for all parallel tasks
- Each task commits to same branch
- Git handles merges/conflicts automatically
- Single PR review at end of level
- **Pros**: Simpler, fewer PRs, natural conflict detection
- **Cons**: All tasks must succeed or all fail

**Option B - Multiple PR Approach**
- Each task creates its own PR
- Merge PRs sequentially after all complete
- **Pros**: Task failures are isolated
- **Cons**: Complex coordination, potential merge conflicts

**Recommendation**: Start with Option A

#### 4. Conflict Detection

**File Overlap Analysis**:
```bash
# For each parallel task pair (A, B):
task_a_files=$(git diff --name-only origin/main...task-a-branch)
task_b_files=$(git diff --name-only origin/main...task-b-branch)

# Check intersection
conflicts=$(comm -12 <(echo "$task_a_files" | sort) <(echo "$task_b_files" | sort))

if [ -n "$conflicts" ]; then
  echo "⚠️ Potential file conflicts detected between task-A and task-B"
  # Serialize these tasks
fi
```

## Detailed Implementation Steps

### Step 1: Parse TaskMaster Dependencies
```bash
# In task-discovery template
cd /workspace/$DOCS_DIR

if [ -f ".taskmaster/tasks/tasks.json" ]; then
  # Extract dependencies
  jq -r '.[] | "\(.id):\(.dependencies | join(","))"' \
    .taskmaster/tasks/tasks.json > /tmp/task-deps.txt
  
  # Build level groups
  python3 /scripts/build-dependency-graph.py \
    /tmp/task-deps.txt \
    /tmp/execution-levels.json
fi
```

### Step 2: Create Dependency Graph Builder
```python
# scripts/build-dependency-graph.py
import json
import sys

def build_levels(tasks_with_deps):
    """
    Input: {"1": [], "2": ["1"], "3": [], "4": ["1","2"]}
    Output: [[1, 3], [2], [4]]
    """
    levels = []
    completed = set()
    
    while len(completed) < len(tasks_with_deps):
        current_level = []
        for task_id, deps in tasks_with_deps.items():
            if task_id in completed:
                continue
            # Can execute if all deps are completed
            if all(dep in completed for dep in deps):
                current_level.append(int(task_id))
        
        if not current_level:
            raise Exception("Circular dependency detected!")
        
        levels.append(sorted(current_level))
        completed.update(str(t) for t in current_level)
    
    return levels

# ... main execution
```

### Step 3: Implement Parallel Processor Template

**Location**: `play-project-workflow-template.yaml`

```yaml
- name: parallel-task-processor
  inputs:
    parameters:
      - name: execution-levels  # "[[1,3],[2],[4]]"
  script:
    image: alpine/k8s:1.31.0
    command: [sh]
    source: |
      #!/bin/sh
      set -e
      
      levels='{{`{{inputs.parameters.execution-levels}}`}}'
      level_count=$(echo "$levels" | jq 'length')
      
      for level_idx in $(seq 0 $((level_count - 1))); do
        level_tasks=$(echo "$levels" | jq -r ".[$level_idx] | join(\",\")")
        echo "🔄 Executing Level $level_idx: [$level_tasks]"
        
        # Launch all tasks in this level simultaneously
        pids=()
        for task_id in $(echo $level_tasks | tr ',' ' '); do
          # Create workflow for this task
          kubectl create -f - <<EOF &
      apiVersion: argoproj.io/v1alpha1
      kind: Workflow
      metadata:
        generateName: play-task-${task_id}-
        namespace: cto
      spec:
        workflowTemplateRef:
          name: play-workflow-template
        arguments:
          parameters:
            - name: task-id
              value: "$task_id"
            # ... other params
      EOF
          pids+=($!)
        done
        
        # Wait for all tasks in level to complete
        for pid in "${pids[@]}"; do
          wait $pid || echo "Task failed: $pid"
        done
        
        echo "✅ Level $level_idx complete"
      done
```

### Step 4: Update Play Workflow for Shared Branch

**Modification**: `play-workflow-template.yaml` agent templates

```bash
# In container-base.sh.hbs for all CLIs

# Instead of: feature/task-{{task_id}}-implementation
# Use: feature/level-{{level_id}}-implementation

LEVEL_ID="${EXECUTION_LEVEL:-0}"
BRANCH_NAME="feature/level-${LEVEL_ID}-task-{{task_id}}"

# Multiple tasks can commit to same branch
git checkout -b "$BRANCH_NAME" 2>/dev/null || git checkout "$BRANCH_NAME"
```

## Testing Strategy

### Phase 1 Test Cases

1. **Simple Parallel (2 independent tasks)**
   - Task 1: Add `/health` endpoint
   - Task 2: Add README documentation
   - **Expected**: Both complete in ~2-4 hours (vs 4-8 sequential)

2. **Three-Level Dependency**
   - Level 0: Task 1 (database schema)
   - Level 1: Task 2 (API using schema), Task 3 (docs)
   - Level 2: Task 4 (integration tests)
   - **Expected**: ~6-8 hours (vs 8-16 sequential)

3. **File Conflict Detection**
   - Task 1: Modify `src/main.rs`
   - Task 2: Also modify `src/main.rs`
   - **Expected**: System detects conflict, serializes tasks

## Metrics & Success Criteria

### Key Metrics
- **Speedup Factor**: `sequential_time / parallel_time`
- **Parallel Efficiency**: `speedup / num_parallel_tasks`
- **Conflict Rate**: `conflicts_detected / parallel_attempts`

### Success Criteria (Phase 1)
- ✅ Successfully execute 2+ independent tasks in parallel
- ✅ Achieve 1.5x+ speedup on test project
- ✅ Zero merge conflicts on test runs
- ✅ All quality gates pass (Rex/Cleo/Cipher/Tess)

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Git merge conflicts | High | Start with non-overlapping file tasks |
| Architectural inconsistency | Medium | Review PRs before merge |
| Resource exhaustion | Medium | Limit parallelism to 3-4 tasks max |
| Complex debugging | High | Enhanced logging, task isolation |
| PR coordination failures | High | Implement robust retry logic |

## Rollout Plan

### Week 1: Foundation
- ✅ Implement dependency graph parser
- ✅ Create parallel processor template
- ✅ Test with 2 simple independent tasks

### Week 2: Enhancement
- Add file conflict detection
- Implement shared branch strategy
- Test with 3-task multi-level dependency

### Week 3: Production
- Enable for selected projects
- Monitor metrics and conflicts
- Iterate based on feedback

## Future Enhancements (Phase 2+)

- **Smart Conflict Prediction**: ML model to predict conflicts before execution
- **Speculative Execution**: Run tasks speculatively, rollback if conflicts
- **Dynamic Parallelism**: Adjust parallel count based on resource availability
- **Cross-Repo Parallelization**: Parallel tasks across microservices

## References

- Design Doc: `docs/engineering/parallel-task-execution-design.md`
- Current Workflow: `infra/charts/controller/templates/workflowtemplates/play-project-workflow-template.yaml`
- TaskMaster Structure: `.taskmaster/docs/task-*/` directories

