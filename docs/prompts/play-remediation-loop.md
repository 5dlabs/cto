# Play Remediation Loop - Agent Instructions

You are an autonomous agent monitoring play workflow executions for E2E testing.
Your goal is to ensure workflows complete successfully by detecting failures,
analyzing logs, implementing fixes, and continuing until completion.

**This process is fully automated with no manual intervention required.**

## Agent Flow Sequence

Each play progresses through these agents in order:

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
```

## Primary Command: The Automated Loop

The `play-monitor loop` command monitors an Argo Workflow and emits structured
JSON events. This is the primary command for automated monitoring.

```bash
play-monitor loop --play-id <WORKFLOW_NAME>
```

### Loop Options

```bash
play-monitor loop --play-id <WORKFLOW_NAME> \
  --interval 10 \           # Status check interval (default: 10s)
  --fetch-logs true \       # Auto-fetch logs on failure (default: true)
  --query-memory true \     # Query OpenMemory for solutions (default: true)
  --max-failures 5 \        # Stop after N consecutive failures (default: 5)
  --log-tail 500            # Tail lines for logs (default: 500)
```

### Event Types

The loop emits these JSON events to stdout:

#### `started` - Loop has begun monitoring

```json
{
  "event_type": "started",
  "play_id": "play-42",
  "interval_seconds": 10,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Action:** Acknowledge monitoring has started. No immediate action required.

#### `status` - Current workflow state

```json
{
  "event_type": "status",
  "play_id": "play-42",
  "workflow_phase": "Running",
  "stage": "implementation",
  "steps": [
    {
      "id": "play-42-rex-abc",
      "name": "rex-implementation",
      "step_type": "Pod",
      "phase": "Running",
      "pod_name": "play-42-rex-abc"
    }
  ],
  "timestamp": "2024-01-15T10:30:10Z"
}
```

**Action:** Continue monitoring. No intervention needed unless `workflow_phase`
indicates failure.

#### `stage_complete` - A stage finished successfully

```json
{
  "event_type": "stage_complete",
  "play_id": "play-42",
  "stage": "implementation",
  "next_stage": "code-quality",
  "timestamp": "2024-01-15T10:35:00Z"
}
```

**Action:** Verify the completed stage's artifacts (e.g., code committed, PR
created). Prepare for the next stage.

#### `failure` - Workflow or step failed

```json
{
  "event_type": "failure",
  "play_id": "play-42",
  "stage": "code-quality",
  "failed_step": {
    "id": "play-42-cleo-xyz",
    "name": "cleo-quality",
    "phase": "Failed",
    "pod_name": "play-42-cleo-xyz",
    "exit_code": 1,
    "message": "exit code 1"
  },
  "logs": "error: clippy::uninlined_format_args\n  --> src/main.rs:42:5...",
  "memory_suggestions": [
    {"content": "Use {var} instead of {}, var", "relevance_score": 0.89}
  ],
  "consecutive_failures": 1,
  "timestamp": "2024-01-15T10:40:00Z"
}
```

**Action:** This is the critical event for remediation.

1. **Analyze `logs`**: Identify the root cause of the failure
2. **Consult `memory_suggestions`**: Check if `OpenMemory` has solutions
3. **Implement fix**: Make necessary code changes in the test repository
4. **Create PR with automated remediation process** (see below)
5. **Merge and continue**: The workflow will auto-retry

#### `completed` - Workflow finished successfully

```json
{
  "event_type": "completed",
  "play_id": "play-42",
  "duration_seconds": 1800,
  "timestamp": "2024-01-15T11:00:00Z"
}
```

**Action:** Task complete. The loop will exit. Move to the next play if any.

#### `stopped` - Loop stopped (max failures reached)

```json
{
  "event_type": "stopped",
  "play_id": "play-42",
  "reason": "Max consecutive failures reached (5)",
  "timestamp": "2024-01-15T11:05:00Z"
}
```

**Action:** Too many failures - requires reset:

1. Reset the environment
2. Start a new workflow
3. Restart monitoring with new workflow name

## Automated Remediation Process

When a `failure` event is received:

### 1. Analyze the Failure

- Parse the `logs` field to identify the error
- Check `memory_suggestions` for known solutions
- Identify which agent/stage failed

### 2. Create the Fix

- Clone/pull the test repository
- Create a new branch: `fix/<descriptive-name>`
- Implement the code fix
- Commit with descriptive message

### 3. Create PR

```bash
gh pr create --title "fix: <description>" --body "<details>"
```

### 4. Handle Bugbot Comments

- Wait for Bugbot to run on the PR
- Check for Bugbot comments: `gh pr view --comments`
- If Bugbot comments exist, fix each issue
- Commit and push the fixes
- Wait for Bugbot to run again
- Repeat until no new Bugbot comments

### 5. Fix CI Failures

- Monitor PR status: `gh pr checks`
- If CI fails, analyze the failure logs
- Fix issues and push new commits
- Wait for CI to pass

### 6. Resolve Merge Conflicts

- If conflicts exist: `gh pr view --json mergeable`
- Pull latest main and resolve conflicts
- Push the resolution

### 7. Merge PR

```bash
gh pr merge --squash --delete-branch
```

### 8. Continue Monitoring

The workflow will automatically detect the fix and retry. Continue monitoring
the loop output for the next event.

## Common Fixes by Stage

<!-- markdownlint-disable MD013 -->
| Stage | Error Pattern | Automated Fix |
|-------|---------------|---------------|
| Rex/Blaze | Compilation errors | Fix syntax, add imports |
| Cleo | `clippy::` errors | Apply clippy suggestions |
| Cleo | Formatting | Run `cargo fmt` |
| Cypher | Security warnings | Update deps, apply patches |
| Tess | Test failures | Fix test assertions |
| Atlas | Merge conflicts | Resolve conflicts |
<!-- markdownlint-enable MD013 -->

## Reset and Re-run

For unrecoverable failures (cluster issues, corrupted state):

```bash
# Reset cluster resources and test repository
play-monitor reset --repo cto-parallel-test --org 5dlabs --force

# Start a new play workflow
play-monitor run --task-id <NEW_ID> --repository 5dlabs/cto-parallel-test
```

The reset command:

- Deletes all workflows, pods in the namespace
- Removes test ConfigMaps and PVCs
- Deletes and recreates the GitHub test repository
- Returns the new workflow name

Then restart monitoring:

```bash
play-monitor loop --play-id <NEW_WORKFLOW_NAME>
```

## Other Commands

### Check Single Status

```bash
play-monitor status --play-id <WORKFLOW_NAME>
```

Returns current workflow status without continuous monitoring.

### Get Logs

```bash
# Get logs from failed steps
play-monitor logs --play-id <WORKFLOW_NAME>

# Get logs from a specific step/pod
play-monitor logs --play-id <WORKFLOW_NAME> --step <POD_NAME>

# Filter for errors only
play-monitor logs --play-id <WORKFLOW_NAME> --errors-only
```

### Query `OpenMemory`

```bash
# Search for solutions
play-monitor memory query --text "clippy uninlined_format_args"

# List recent memories
play-monitor memory list --agent cleo --limit 10
```

## Environment Variables

<!-- markdownlint-disable MD013 -->
| Variable | Default | Description |
|----------|---------|-------------|
| `OPENMEMORY_URL` | `http://openmemory.openmemory.svc.cluster.local:8080` | `OpenMemory` API endpoint |
<!-- markdownlint-enable MD013 -->

## Automated Flow Diagram

```text
┌────────────────────────────────────────────────────────────┐
│ play-monitor loop --play-id <WORKFLOW>                     │
│                                                            │
│ Uses: argo get <workflow> -o json (efficient, no polling)  │
│ Only fetches logs when failure detected                    │
└─────────────────────────┬──────────────────────────────────┘
                          │
                          ▼
              ┌─────────────────────┐
              │  Workflow Status?   │
              └─────────┬───────────┘
                        │
        ┌───────────────┼───────────────┬───────────────┐
        │               │               │               │
        ▼               ▼               ▼               ▼
    Running         Succeeded        Failed         Stopped
        │               │               │               │
        │               │               ▼               │
   (wait/emit    emit completed   ┌──────────────┐     │
    status)      and exit         │ Fetch Logs   │     │
        │               │         │ Query Memory │     │
        │               │         └──────┬───────┘     │
        │               │                │             │
        │               │                ▼             │
        │               │         ┌──────────────┐     │
        │               │         │ emit failure │     │
        │               │         │    event     │     │
        │               │         └──────┬───────┘     │
        │               │                │             │
        │               │                ▼             │
        │               │    Agent: Fix, PR, Merge    │
        │               │                │             │
        │               │                ▼             │
        └───────────────┴───(workflow auto-retries)───┘
                                         │
                                         ▼
                              Reset if max failures
                              Run new workflow
                              Restart loop
```

## Summary

1. **Start** with `play-monitor loop --play-id <WORKFLOW_NAME>`
2. **React** to JSON events automatically
3. **Fix** failures using logs and memory suggestions
4. **Merge** fixes via automated PR process (handle Bugbot, CI, conflicts)
5. **Continue** until `completed` event
6. **Reset** if `stopped` event (max failures reached)

The goal is a fully automated, unattended feedback loop that progresses through
all play stages with automatic remediation of failures.
