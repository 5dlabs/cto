# Play Remediation Loop - Agent Instructions

You are monitoring a play workflow execution for E2E testing. Your goal is
to ensure the workflow completes successfully by detecting failures, analyzing
logs, implementing fixes, and continuing until completion.

## Agent Flow Sequence

Each task progresses through these agents in order:

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
    ↓
[Future: Bolt (Deployment)]
```

## Prerequisites

Ensure the `play-monitor` CLI is available:

```bash
# Build from the repository root
cargo build -p play-monitor --release

# The binary will be at target/release/play-monitor
# Add to PATH or use full path
```

## Monitoring Loop

### Step 1: Check Status

Run the status command to check current workflow state:

```bash
play-monitor status --task-id <TASK_ID>
```

This returns JSON with:

- `status`: Current state (`running`, `failed`, `completed`, `pending`)
- `stage`: Current agent stage (implementation, code-quality, security, qa)
- `pods`: List of pods with phase, exit codes, restart counts
- `failed_pods`: List of pods that have failed

### Step 2: Handle Status

Based on the status:

**If `status` is "running":**

- Wait 30-60 seconds
- Check status again
- Continue monitoring

**If `status` is "failed":**

- Proceed to Step 3 (Get Logs)
- Analyze and remediate

**If `status` is "completed":**

- Verify stage confirmations
- Proceed to next task or finish

**If `status` is "pending":**

- Wait for pods to start
- Check status again after 30 seconds

### Step 3: Get Failure Logs

When a failure is detected:

```bash
# Get logs for the task
play-monitor logs --task-id <TASK_ID>

# Or get logs with error filtering
play-monitor logs --task-id <TASK_ID> --errors-only

# For a specific pod
play-monitor logs --pod <POD_NAME>
```

### Step 4: Analyze Failure

Common failure patterns and remediation:

<!-- markdownlint-disable MD013 -->
| Failure Type | Indicators | Remediation |
|--------------|------------|-------------|
| OOMKilled | `reason: "OOMKilled"` | Increase memory limits or optimize code |
| Exit Code 1 | `exit_code: 1` | Check logs for specific error messages |
| CrashLoopBackOff | `reason: "CrashLoopBackOff"` | Config or dependency issue |
| Image Pull Error | `reason: "ImagePullBackOff"` | Check image name/tag/registry |
| Lint/Format Fail | Cleo stage errors | Run `cargo fmt` and `cargo clippy` |
| Test Failure | Tess stage errors | Fix failing tests |
| Security Issue | Cypher stage errors | Address security findings |
<!-- markdownlint-enable MD013 -->

### Step 5: Implement Fix

1. Identify the root cause from logs
2. Make necessary code changes
3. Test locally if possible
4. Commit changes with descriptive message

### Step 6: Create PR and Merge

1. Create a new branch from latest main
2. Push changes
3. Create pull request
4. Wait for CI checks
5. Merge to main

### Step 7: Workflow Retry

After merging:

- The workflow will automatically detect the fix and retry
- Monitor status to confirm the stage passes
- Continue to next stage

### Step 8: Reset and Re-run (For Unrecoverable Failures)

If a failure cannot be fixed in code (e.g., cluster state issues):

```bash
# Reset cluster resources and test repository
play-monitor reset --repo cto-parallel-test --org 5dlabs

# Re-run the play workflow
play-monitor run --task-id <TASK_ID> --repository 5dlabs/cto-parallel-test
```

The reset command:

- Deletes all workflows, pods in `agent-platform` namespace
- Removes test ConfigMaps (play-*, test-*, coderun-*, docsrun-*)
- Removes test PVCs (workspace-play-*, workspace-test-*)
- Deletes and recreates the GitHub test repository
- Initializes with minimal structure (README, .gitignore)

The run command:

- Submits a new play workflow via Argo CLI
- Uses the play-workflow-template
- Returns the workflow name for monitoring

## Stage Confirmations

Verify each stage completed successfully before moving on:

### Rex/Blaze (Implementation)

- [ ] Code changes committed
- [ ] Pull request created
- [ ] No compilation errors

### Cleo (Code Quality)

- [ ] `cargo fmt` passing
- [ ] `cargo clippy` passing (pedantic)
- [ ] No lint errors

### Cypher (Security)

- [ ] No secrets in code
- [ ] Dependencies checked
- [ ] Security scan passing

### Tess (QA)

- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] QA criteria met

### Atlas (Integration)

- [ ] PR merged to main
- [ ] Build successful
- [ ] Integration verified

## Task Progression

Once all stages complete for a task:

1. Verify final status is "completed"
2. Check all stage confirmations
3. Move to next task ID in sequence
4. Repeat the monitoring loop

## Watch Mode (Optional)

For continuous monitoring with visual updates:

```bash
play-monitor watch --task-id <TASK_ID> --interval 30
```

This provides a real-time terminal display of pod status.

## Example Session

```bash
# Start monitoring task 42
$ play-monitor status --task-id 42
{
  "task_id": "42",
  "status": "running",
  "stage": "implementation",
  "pods": [
    {"name": "rex-task-42-abc123", "phase": "Running", "restarts": 0}
  ],
  "failed_pods": []
}

# ... wait and check again ...

$ play-monitor status --task-id 42
{
  "task_id": "42",
  "status": "failed",
  "stage": "code-quality",
  "pods": [
    {"name": "cleo-task-42-xyz789", "phase": "Failed", "exit_code": 1}
  ],
  "failed_pods": ["cleo-task-42-xyz789"]
}

# Get logs to understand the failure
$ play-monitor logs --task-id 42 --errors-only
error: clippy::uninlined_format_args
  --> src/main.rs:42:5
...

# Fix the issue, commit, push, merge
# Monitor until completion
```

## OpenMemory Integration

Query agent memories to understand what patterns and solutions have been stored:

### List Recent Memories

```bash
# List memories for a specific task
play-monitor memory list --task-id 42 --limit 20

# List memories by agent
play-monitor memory list --agent rex --limit 10
```

### Semantic Query

```bash
# Find memories related to specific errors or patterns
play-monitor memory query --text "Docker build failures npm" --limit 10

# Include waypoint connections for related memories
play-monitor memory query --text "authentication flow" --include-waypoints
```

### Memory Statistics

```bash
# Check memory health and usage statistics
play-monitor memory stats

# Stats filtered by agent
play-monitor memory stats --agent cleo
```

### Example Memory Response

```json
{
  "success": true,
  "query": "Docker build failures",
  "results": [
    {
      "id": "mem_abc123",
      "content": "Pattern: npm-clean-install\nSolution: rm node_modules",
      "metadata": {
        "agent": "rex",
        "task_id": "task-38",
        "pattern_type": "implementation",
        "success": true
      },
      "salience": 0.85,
      "score": 0.92
    }
  ],
  "total": 1
}
```

### Using Memory for Remediation

Before implementing a fix, check if similar issues have been solved:

1. Query memory for the error pattern
2. If solutions exist, apply the most relevant one
3. If no solutions, implement fix and store successful pattern

```bash
# Check for existing solutions
play-monitor memory query --text "clippy::uninlined_format_args error"

# After successful fix, the agent should store the pattern
# (done automatically by agent container scripts)
```

## Troubleshooting

### No pods found

- Check task ID is correct
- Verify namespace (default: `agent-platform`)
- Workflow may not have started yet

### Victoria Logs not available

- Logs fall back to kubectl
- Set `VICTORIA_LOGS_URL` env var if using custom endpoint

### Workflow stuck

- Check controller logs
- Verify Argo workflow status
- May need manual intervention

## Environment Variables

<!-- markdownlint-disable MD013 -->
| Variable | Default | Description |
|----------|---------|-------------|
| `VICTORIA_LOGS_URL` | See below | Victoria Logs API endpoint |
| `OPENMEMORY_URL` | `http://openmemory:3000` | OpenMemory API endpoint |
<!-- markdownlint-enable MD013 -->

Default Victoria Logs URL:
`http://victoria-logs-victoria-logs-single-server.telemetry.svc.cluster.local:9428`

## Summary

1. **Monitor** with `play-monitor status --task-id X`
2. **Detect** failures from status and failed_pods
3. **Analyze** with `play-monitor logs --task-id X`
4. **Fix** the root cause in code
5. **Merge** changes via PR
6. **Repeat** until all tasks complete

The goal is an unattended feedback loop that progresses through all tasks
with automatic remediation of recoverable failures.
