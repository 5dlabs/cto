# Play Remediation Loop - Agent Instructions

You are an autonomous agent executing the full E2E play workflow.
Your goal is to start a play, monitor it, remediate failures, and continue
until completion.

**This process is fully automated with no manual intervention required.**

## The One Command

Run this single command from the test repository directory:

```bash
play-monitor full --task-id <TASK_ID>
```

This command:

1. **Reads** `cto-config.json` for play configuration
2. **Submits** the workflow to Argo with configured agents
3. **Monitors** the workflow via Argo CLI (efficient, event-driven)
4. **Emits** JSON events for each status change
5. **Fetches logs** automatically on failure
6. **Queries `OpenMemory`** for known solutions
7. **Continues** until completion or max failures

### Options

```bash
play-monitor full --task-id <TASK_ID> \
  --config cto-config.json \    # Config file path (default: cto-config.json)
  --interval 10 \               # Status check interval in seconds (default: 10)
  --max-failures 5 \            # Stop after N consecutive failures (default: 5)
  --template play-workflow-template  # Argo template name
```

## Environment Variables

<!-- markdownlint-disable MD013 -->
| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | GitHub token for `gh` CLI and git push (automation) |
| `OPENMEMORY_URL` | No | `OpenMemory` API endpoint (defaults to cluster DNS) |
| `KUBECONFIG` | No | Kubernetes config for `argo` and `kubectl` CLIs |
<!-- markdownlint-enable MD013 -->

The `gh` CLI uses `GITHUB_TOKEN` automatically. Git push uses HTTPS with token
when `GITHUB_TOKEN` is set, otherwise falls back to SSH.

## Agent Flow Sequence

```text
Rex/Blaze (Implementation)
    ↓
Cleo (Code Quality Review)
    ↓
Cypher (Security Review)
    ↓
Tess (QA Testing)
    ↓
Atlas (Integration + Merge)
```

## JSON Events

The command emits these JSON events to stdout:

### `started` - Play submitted and monitoring begun

```json
{
  "event_type": "started",
  "play_id": "play-task-42-abc123",
  "interval_seconds": 10,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### `status` - Current workflow state

```json
{
  "event_type": "status",
  "play_id": "play-task-42-abc123",
  "workflow_phase": "Running",
  "stage": "implementation",
  "steps": [...],
  "timestamp": "2024-01-15T10:30:10Z"
}
```

### `stage_complete` - A stage finished successfully

```json
{
  "event_type": "stage_complete",
  "play_id": "play-task-42-abc123",
  "stage": "implementation",
  "next_stage": "code-quality",
  "timestamp": "2024-01-15T10:35:00Z"
}
```

### `failure` - Workflow or step failed

```json
{
  "event_type": "failure",
  "play_id": "play-task-42-abc123",
  "stage": "code-quality",
  "failed_step": {
    "name": "cleo-quality",
    "phase": "Failed",
    "exit_code": 1,
    "message": "clippy failed"
  },
  "logs": "error: clippy::uninlined_format_args...",
  "memory_suggestions": [
    {"content": "Use {var} instead of {}, var", "relevance_score": 0.89}
  ],
  "consecutive_failures": 1,
  "timestamp": "2024-01-15T10:40:00Z"
}
```

**On failure, you should:**

1. Parse `logs` to identify the root cause
2. Check `memory_suggestions` for known solutions
3. Make code fixes in the test repository
4. Commit and push via automated PR process
5. The workflow will auto-retry

### `completed` - Workflow finished successfully

```json
{
  "event_type": "completed",
  "play_id": "play-task-42-abc123",
  "duration_seconds": 1800,
  "timestamp": "2024-01-15T11:00:00Z"
}
```

### `stopped` - Max failures reached

```json
{
  "event_type": "stopped",
  "play_id": "play-task-42-abc123",
  "reason": "Max consecutive failures reached (5)",
  "timestamp": "2024-01-15T11:05:00Z"
}
```

**On stopped:**

1. Reset environment: `play-monitor reset --force`
2. Re-run with new task ID

## Automated PR Process

When `failure` event is received:

1. **Analyze** - Parse logs, check memory suggestions
2. **Fix** - Make code changes in test repo
3. **Branch** - Create `fix/<description>` branch
4. **PR** - `gh pr create --title "fix: ..." --body "..."`
5. **Bugbot** - Wait for Bugbot, fix any comments, repeat until clean
6. **CI** - Wait for CI to pass, fix any failures
7. **Merge** - `gh pr merge --squash --delete-branch`
8. **Continue** - Workflow auto-retries on main update

## Common Fixes by Stage

<!-- markdownlint-disable MD013 -->
| Stage | Error Pattern | Fix |
|-------|---------------|-----|
| Rex/Blaze | Compilation errors | Fix syntax, add imports |
| Cleo | `clippy::` errors | Apply clippy suggestions |
| Cleo | Formatting | Run `cargo fmt` |
| Cypher | Security warnings | Update deps |
| Tess | Test failures | Fix assertions |
<!-- markdownlint-enable MD013 -->

## Reset and Retry

If too many failures occur:

```bash
# Reset cluster and test repo
play-monitor reset --force

# Start fresh with new task ID
play-monitor full --task-id <NEW_TASK_ID>
```

## Example Session

```bash
# From test repo directory
cd /path/to/cto-parallel-test

# Set GitHub token for automation
export GITHUB_TOKEN="ghp_..."

# Run the full E2E loop
play-monitor full --task-id 42

# Output (JSON events):
{"event_type":"started","play_id":"play-task-42-abc123",...}
{"event_type":"status","workflow_phase":"Running","stage":"implementation",...}
{"event_type":"stage_complete","stage":"implementation","next_stage":"code-quality",...}
{"event_type":"failure","stage":"code-quality","logs":"error: clippy...",...}
# (you fix the code, push, workflow retries)
{"event_type":"status","workflow_phase":"Running","stage":"code-quality",...}
{"event_type":"stage_complete","stage":"code-quality","next_stage":"security",...}
# ... continues through all stages ...
{"event_type":"completed","duration_seconds":3600,...}
```

## Summary

1. **One command**: `play-monitor full --task-id <ID>`
2. **Reads config**: Uses `cto-config.json` for agent configuration
3. **Emits events**: React to JSON events automatically
4. **Fix failures**: Parse logs, apply fixes, push PRs
5. **Auto-retry**: Workflow retries on main branch updates
6. **Reset if stuck**: `play-monitor reset --force` then retry
