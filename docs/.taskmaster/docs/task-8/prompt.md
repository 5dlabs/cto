# Autonomous Implementation Prompt: Git Worktrees for Parallel Isolation

## Mission Statement
Implement git worktree-based workspace isolation for safe parallel task execution with better performance than full repository clones.

## Technical Requirements
1. **Init Template** creating base repo and per-task worktrees
2. **Parameter Management** with taskId generation and workspace paths
3. **Optional PVC Isolation** using volumeClaimTemplates
4. **Concurrency Controls** via semaphores and rate limiting
5. **Cleanup Management** for worktrees and PVCs on completion

## Key Implementation Points
- Base repo at /work/base, worktrees at /work/trees/${TASK_ID}
- Integration with coderun-template for workspace.path setting
- ConfigMap-based semaphores for per-repo/branch concurrency limits
- Automatic cleanup via onExit handlers

## Success Criteria
- Multiple parallel workflows use distinct worktree directories
- Performance >2x faster than full clones for typical repositories
- No file handle collisions between concurrent executions
- Proper cleanup prevents storage leaks