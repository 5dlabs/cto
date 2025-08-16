# Acceptance Criteria: Git Worktrees for Parallel Isolation

## Worktree Management
- [ ] Base repository created at /work/base with no checkout
- [ ] Per-task worktrees created at /work/trees/${TASK_ID}
- [ ] Each worktree checks out specified ref/branch correctly
- [ ] Git safety directories configured properly

## Parallel Execution
- [ ] Multiple concurrent workflows use separate worktree directories
- [ ] No file handle collisions between parallel executions
- [ ] Semaphore limits enforce per-repo/branch concurrency controls
- [ ] Performance significantly better than full clone approach

## PVC Integration
- [ ] Optional PVC mode works with volumeClaimTemplates
- [ ] EmptyDir fallback works when PVC disabled
- [ ] PVC cleanup occurs on workflow completion
- [ ] Resource quotas respected in both modes

## Cleanup and Resource Management
- [ ] Worktrees properly removed on workflow completion
- [ ] Base repository pruned and garbage collected
- [ ] No storage leaks from abandoned worktrees
- [ ] onExit cleanup handles both success and failure cases

## Integration Testing
- [ ] Works with coderun-template workspace.path setting
- [ ] Compatible with GitHub App token mounting
- [ ] Rate limiting and debounce controls function correctly
- [ ] End-to-end workflow execution successful