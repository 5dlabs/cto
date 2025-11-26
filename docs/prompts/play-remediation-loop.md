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
3. **Monitors** everything via `kubectl --watch` streams (real-time)
4. **Emits** JSON events for workflows, CRDs, pods, and sensors
5. **Fetches logs** automatically on failure
6. **Polls GitHub** for PR status, reviews, and checks
7. **Continues** until completion or max failures

### Options

```bash
play-monitor full --task-id <TASK_ID> \
  --config cto-config.json \    # Config file path (default: cto-config.json)
  --interval 30 \               # GitHub poll interval in seconds (default: 30)
  --max-failures 5 \            # Stop after N consecutive failures (default: 5)
  --template play-workflow-template \  # Argo template name
  --repository 5dlabs/cto-parallel-test \  # GitHub repo for PR polling
  --output-file /tmp/events.jsonl  # Optional: write events to file
```

### Alternative: Watch Only (no workflow submission)

```bash
play-monitor watch --task-id <TASK_ID> \
  --repository 5dlabs/cto-parallel-test \
  --output-file /tmp/events.jsonl
```

## Environment Variables

<!-- markdownlint-disable MD013 -->
| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | GitHub token for `gh` CLI and git push (automation) |
| `OPENMEMORY_URL` | No | OpenMemory API endpoint (for verification commands) |
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

The command emits these JSON events to stdout (and optionally to a file):

### `started` - Monitoring begun

```json
{
  "event_type": "started",
  "task_id": "42",
  "watching": [
    "workflows.argoproj.io (ns: argo)",
    "coderuns.agents.platform (ns: agent-platform)",
    "docsruns.agents.platform (ns: agent-platform)",
    "sensors.argoproj.io (ns: argo)",
    "pods (ns: agent-platform)"
  ],
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### `resource` - CRD/Pod/Workflow change (real-time from kubectl --watch)

```json
{
  "event_type": "resource",
  "task_id": "42",
  "resource_type": "workflow",
  "action": "modified",
  "name": "play-task-42-abc123",
  "namespace": "argo",
  "phase": "Running",
  "labels": {"task-id": "42"},
  "timestamp": "2024-01-15T10:30:10Z"
}
```

Resource types: `workflow`, `coderun`, `docsrun`, `sensor`, `pod`

### `status` - Current workflow state (legacy compatibility)

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
  "consecutive_failures": 1,
  "timestamp": "2024-01-15T10:40:00Z"
}
```

**On failure, you should:**

1. Parse `logs` to identify the root cause
2. Make code fixes in the test repository
3. Commit and push via automated PR process
4. The workflow will auto-retry

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

### `github` - PR state update (from polling)

```json
{
  "event_type": "github",
  "task_id": "42",
  "repository": "5dlabs/cto-parallel-test",
  "pull_request": {
    "number": 123,
    "state": "open",
    "title": "feat: implement task 42",
    "mergeable": true,
    "draft": false,
    "labels": ["task-42", "implementation"],
    "reviews": [{"author": "5DLabs-Cleo", "state": "APPROVED"}],
    "checks": [{"name": "CI", "status": "completed", "conclusion": "success"}]
  },
  "timestamp": "2024-01-15T10:35:00Z"
}
```

## Automated PR Process

When `failure` event is received:

1. **Analyze** - Parse logs to identify the issue
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

## OpenMemory Verification

After each stage completes, verify agents are using OpenMemory correctly:

```bash
# List memories created by a specific agent
play-monitor memory list --agent rex --limit 10

# Query memories for a specific task
play-monitor memory list --task-id 42
```

### What to Verify

1. **Each agent persists memories** - Check that memories are created
2. **Memory quality** - Memories should be meaningful, not noise
3. **Proper tagging** - Memories should have agent and task context

### OpenMemory Documentation

Reference the integration guide for best practices:
`docs/openmemory-integration-guide.md`

Agents should:

- Store learnings from successful operations
- Record error patterns and their solutions
- Tag memories with task context for retrieval

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

# Run the full E2E loop with file output
play-monitor full --task-id 42 --output-file /tmp/play-events.jsonl

# Output (JSON events - real-time from kubectl --watch):
{"event_type":"started","task_id":"42","watching":["workflows.argoproj.io",...]}
{"event_type":"resource","resource_type":"workflow","action":"modified","phase":"Running",...}
{"event_type":"resource","resource_type":"coderun","action":"modified","phase":"Running",...}
{"event_type":"resource","resource_type":"pod","action":"modified","phase":"Running",...}
{"event_type":"github","repository":"5dlabs/cto-parallel-test","pull_request":{...},...}
{"event_type":"failure","stage":"code-quality","logs":"error: clippy...",...}
# (you fix the code, push, workflow retries)
{"event_type":"resource","resource_type":"workflow","phase":"Running",...}
{"event_type":"completed","duration_seconds":3600,...}

# Tail the event file while working
tail -f /tmp/play-events.jsonl | jq .

# After completion, verify OpenMemory usage
play-monitor memory list --task-id 42
```

## Summary

1. **One command**: `play-monitor full --task-id <ID>`
2. **Reads config**: Uses `cto-config.json` for agent configuration
3. **Watches everything**: Workflows, CRDs, pods, sensors via `kubectl --watch`
4. **Polls GitHub**: PR status, reviews, checks on interval
5. **Emits events**: React to JSON `resource`, `failure`, `github` events
6. **File output**: Use `--output-file` for persistent event log
7. **Fix failures**: Parse logs, apply fixes, push PRs
8. **Auto-retry**: Workflow retries on main branch updates
9. **Verify memory**: Use `play-monitor memory list` to check agent usage
10. **Reset if stuck**: `play-monitor reset --force` then retry
