# Toolman Guide: Git Worktrees for Parallel Isolation

## Overview
Guide for using git worktree-based workspace isolation for safe parallel task execution.

## Key Concepts

### Worktree Structure
```
/work/
├── base/           # Shared git repository (no checkout)
└── trees/          # Individual worktrees per task
    ├── task-123/   # Worktree for task 123
    └── task-456/   # Worktree for task 456
```

### Usage in Templates
```yaml
# Enhanced coderun-template with worktree support
- name: init-worktree
  template: init-worktree
  arguments:
    parameters:
      - {name: taskId, value: "pr-{{workflow.parameters.pr}}"}
- name: create-coderun
  dependencies: [init-worktree]
  template: create-coderun
  arguments:
    parameters:
      - {name: workspacePath, value: "{{tasks.init-worktree.outputs.parameters.workspacePath}}"}
```

## Testing Tools

### Worktree Tester
```bash
# Test parallel execution
./scripts/test-worktrees.sh --parallel 10 --repo test-org/test-repo

# Performance comparison
./scripts/test-worktrees.sh --benchmark --iterations 5
```

### Cleanup Validator
```bash
# Validate cleanup after workflows
./scripts/validate-cleanup.sh --check-storage --check-processes
```

## Troubleshooting

### Common Issues
- **Git safe directory errors**: Directories not in git safe list
- **Worktree creation failures**: Stale worktree references
- **Cleanup issues**: Orphaned worktrees or PVCs

### Debug Commands
```bash
# Check worktree status
git -C /work/base worktree list

# Validate semaphore configuration
kubectl get configmap workflow-semaphores -o yaml

# Monitor resource usage
kubectl top pods -l app=coderun
```

## Performance Benefits
- **Shared Objects**: Git database shared between worktrees
- **Faster Checkout**: Worktree creation vs full clone ~3-5x faster
- **Storage Efficiency**: Significant storage savings for large repositories
- **Network Optimization**: Reduced git fetch operations