# Parallel Task Execution - User Guide

This guide explains how to use the parallel task execution feature in the CTO platform to significantly reduce project completion times by running independent tasks concurrently.

## Overview

The parallel task execution system analyzes TaskMaster dependencies and automatically creates execution levels where tasks with no interdependencies run simultaneously. This can provide **theoretical speedups of 1.5x to 3x** depending on your project's dependency structure.

## Quick Start

### Option 1: Using MCP Tool (Cursor/IDE Integration)

The easiest way to trigger parallel execution is through the MCP `play()` tool:

```javascript
play({
  task_id: 1,
  repository: "5dlabs/my-repo",
  service: "my-service",
  docs_repository: "5dlabs/my-repo",
  docs_project_directory: "docs",
  parallel_execution: true  // Enable parallel execution
})
```

**Benefits:**
- ✅ No need to write YAML manifests
- ✅ Direct integration with Cursor/IDE
- ✅ Uses your configured agents and defaults
- ✅ Simple boolean flag to enable parallel mode

### Option 2: Manual Workflow Submission

Set the `parallel-execution` parameter to `"true"` when launching a play project workflow:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: play-project-parallel-
spec:
  workflowTemplateRef:
    name: play-project-workflow-template
  arguments:
    parameters:
      - name: parallel-execution
        value: "true"
      - name: repository
        value: "your-org/your-repo"
      # ... other parameters
```

Submit with:
```bash
argo submit -n cto parallel-play.yaml
```

### 2. TaskMaster Requirements

Your project must have TaskMaster configured with proper task dependencies:

```json
{
  "master": {
    "tasks": [
      {"id": 1, "title": "Setup Database", "dependencies": []},
      {"id": 2, "title": "Create API Endpoints", "dependencies": [1]},
      {"id": 3, "title": "Build Frontend", "dependencies": []},
      {"id": 4, "title": "Integration Tests", "dependencies": [2, 3]}
    ]
  }
}
```

This creates the execution plan:
- **Level 0**: Tasks 1, 3 (run in parallel)
- **Level 1**: Task 2 (waits for task 1)
- **Level 2**: Task 4 (waits for tasks 2, 3)

### 3. Monitor Execution

Watch the workflow logs to see parallel execution in action:

```bash
kubectl logs -f workflow/your-workflow-name -c main
```

You'll see output like:
```
📊 LEVEL 0
════════════════════════════════════════
🚀 Launching parallel tasks for level 0:
  → Task 1
  → Task 3
    ✅ Created workflow: play-task-1-abc123
    ✅ Created workflow: play-task-3-def456
⏳ Waiting for all tasks in level 0 to complete...
✅ All tasks in level 0 completed successfully
```

## Configuration Options

### Core Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `parallel-execution` | `"false"` | Enable/disable parallel execution |
| `integration-pr-enabled` | `"true"` | Create integration PRs per level |
| `integration-base-branch` | `"main"` | Base branch for integration merges |
| `conflict-detection` | `"true"` | Detect overlapping file changes |

### Integration PR Coordination

When enabled, the system creates integration PRs that merge all task branches from each execution level:

- **Per-Level Integration**: Each execution level gets its own integration branch
- **Conflict Detection**: Warns about potential file overlaps between parallel tasks
- **Automatic Merging**: Combines all task branches into a single integration PR
- **Sequential Integration**: Integration PRs are created level by level

Example integration branch name: `integration/level-0-play-project-workflow-abc123`

### Conflict Detection

The system analyzes changed files across PRs in the same level to identify potential conflicts:

```
⚠️ Potential overlap detected across PRs in level 0:
src/config/database.js
src/utils/helpers.js
```

This helps you identify when parallel tasks might be touching the same code areas.

## Best Practices

### 1. Design for Parallelism

Structure your TaskMaster tasks to maximize parallelism:

**Good**: Independent feature development
```json
[
  {"id": 1, "title": "User Authentication", "dependencies": []},
  {"id": 2, "title": "Product Catalog", "dependencies": []},
  {"id": 3, "title": "Shopping Cart", "dependencies": []},
  {"id": 4, "title": "Integration Tests", "dependencies": [1, 2, 3]}
]
```

**Avoid**: Over-coupling tasks
```json
[
  {"id": 1, "title": "Setup Database", "dependencies": []},
  {"id": 2, "title": "Create User Model", "dependencies": [1]},
  {"id": 3, "title": "Create User Controller", "dependencies": [2]},
  {"id": 4, "title": "Create User Views", "dependencies": [3]}
]
```

### 2. File Organization

Organize code to minimize conflicts between parallel tasks:

- **Separate directories**: Each task works in its own module/directory
- **Minimal shared files**: Avoid multiple tasks modifying the same files
- **Configuration isolation**: Use separate config files or sections

### 3. Testing Strategy

- **Unit tests**: Each task should have comprehensive unit tests
- **Integration level**: Use final level for integration/E2E tests
- **Dependency validation**: Ensure task dependencies are correctly specified

## Performance Benefits

### Theoretical Speedup Calculation

The system calculates theoretical speedup assuming equal task durations:

```
Speedup = Sequential Time / Parallel Time
        = Total Tasks / Number of Levels
```

**Example**: 6 tasks in 3 levels = 6/3 = 2x speedup

### Real-World Factors

Actual speedup depends on:
- **Task duration variance**: Some tasks take longer than others
- **Resource constraints**: Kubernetes cluster capacity
- **Agent availability**: Number of concurrent agent pods
- **Dependency accuracy**: Incorrectly specified dependencies reduce parallelism

## Troubleshooting

### Common Issues

**1. No parallelism detected**
```
📊 Execution levels: 6
📊 Max parallel tasks: 1
```
- Check TaskMaster dependencies - tasks may be over-coupled
- Verify `tasks.json` format is correct

**2. Merge conflicts in integration PR**
```
❌ Merge conflict when merging feature/task-2-implementation
```
- Review file overlap warnings during execution
- Consider restructuring tasks to avoid shared files
- Manual conflict resolution may be needed

**3. Workflow timeouts**
```
⏱️ Timeout waiting for level 0 tasks
```
- Check individual task workflows for failures
- Increase timeout if tasks are legitimately long-running
- Review agent resource limits

**4. Missing TaskMaster data**
```
❌ TaskMaster tasks.json not found: .taskmaster/tasks/tasks.json
```
- Ensure TaskMaster is properly initialized in your project
- Verify the correct path and file structure

### Debug Commands

Monitor parallel execution:
```bash
# Watch workflow progress
kubectl get workflows -l parallel-execution=true -w

# Check individual task workflows
kubectl get workflows -l execution-level=0

# View workflow logs
kubectl logs workflow/your-workflow-name -c main -f

# Check for failed tasks
kubectl get workflows -l project-play=true --field-selector status.phase=Failed
```

## Advanced Usage

### Custom Dependency Analysis

For complex projects, you may want to analyze the dependency graph before execution:

```bash
# Run dependency analysis locally
python3 scripts/build-dependency-graph.py .taskmaster/tasks/tasks.json

# Expected output:
# ✅ Dependency graph built successfully
# 📊 Total tasks: 6
# 📊 Execution levels: 3
# 📊 Max parallel tasks: 3
# 📊 Theoretical speedup: 2.0x
```

### Integration with CI/CD

Parallel execution works seamlessly with existing CI/CD pipelines:

1. **TaskMaster generates** task dependencies
2. **Workflow orchestrator** creates execution levels
3. **Agents execute** tasks in parallel per level
4. **Integration PRs** coordinate results
5. **Final merge** combines all changes

### Monitoring and Metrics

The system provides telemetry for analyzing performance:

- **Execution time per level**
- **Total workflow duration**
- **Actual vs theoretical speedup**
- **Resource utilization**

## Migration Guide

### From Sequential to Parallel

1. **Audit dependencies**: Ensure TaskMaster dependencies are accurate
2. **Test in isolation**: Run with a small number of tasks first
3. **Monitor conflicts**: Watch for file overlap warnings
4. **Gradual rollout**: Enable for specific projects before global adoption

### Fallback to Sequential

If issues arise, disable parallel execution:

```yaml
parameters:
  - name: parallel-execution
    value: "false"  # Falls back to sequential processing
```

## Limitations

### Current Limitations

- **Resource overhead**: Each parallel task requires its own workflow pod
- **Cluster capacity**: Limited by available Kubernetes resources
- **Agent concurrency**: Multiple tasks may compete for agent instances
- **File conflicts**: Tasks modifying the same files require manual resolution

### Future Enhancements

- **Smart scheduling**: Better resource utilization and agent pooling
- **Automatic conflict resolution**: AI-powered merge conflict resolution
- **Dynamic parallelism**: Adjust parallelism based on cluster capacity
- **Performance analytics**: Detailed metrics and optimization recommendations

## Support

For issues with parallel execution:

1. **Check logs**: Review workflow and individual task logs
2. **Validate TaskMaster**: Ensure proper dependency specification
3. **Test locally**: Use the dependency graph builder script
4. **Contact support**: Provide workflow name and error details

The parallel execution system is designed to be robust and provide significant performance benefits when properly configured. With careful task design and dependency management, you can achieve substantial reductions in project completion time.
