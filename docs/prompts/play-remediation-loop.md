# Play Remediation Loop - Automated Agent Instructions

You are running a fully automated E2E test loop. No human intervention required.
Your job is to monitor, detect failures, fix them, and continue until completion.

## Start the Loop

```bash
play-monitor loop --play-id <PLAY_ID>
```

This streams JSON events. Process each event and take action automatically.

## Event Handling

### On `started`

```json
{"event_type":"started","play_id":"42","interval_seconds":30}
```

Continue reading events. The loop is running.

---

### On `status`

```json
{"event_type":"status","play_id":"42","workflow_status":"running","stage":"implementation"}
```

No action. Workflow is progressing.

---

### On `failure` → FIX IT AUTOMATICALLY

```json
{
  "event_type": "failure",
  "play_id": "42",
  "stage": "code-quality",
  "failed_pods": [{"name": "cleo-task-42-xyz", "exit_code": 1}],
  "logs": "error: clippy::uninlined_format_args\n  --> src/main.rs:42:5",
  "memory_suggestions": [
    {"content": "Use {var} instead of {}, var", "relevance_score": 0.89}
  ],
  "consecutive_failures": 1
}
```

**Automated remediation process:**

1. **Analyze the failure**
   - Parse the `logs` field to identify the error
   - Check `memory_suggestions` for known solutions

2. **Create the fix**
   - Clone/pull the test repository
   - Create a new branch: `fix/<descriptive-name>`
   - Implement the code fix
   - Commit with descriptive message

3. **Create PR**

   ```bash
   gh pr create --title "fix: <description>" --body "<details>"
   ```

4. **Handle Bugbot comments**
   - Wait for Bugbot to run on the PR
   - Check for Bugbot comments: `gh pr view --comments`
   - If Bugbot comments exist, fix each issue
   - Commit and push the fixes
   - Wait for Bugbot to run again
   - Repeat until no new Bugbot comments appear

5. **Fix CI failures**
   - Check CI status: `gh pr checks`
   - If any checks fail, analyze and fix
   - Push fixes and wait for CI to pass
   - All checks must be green before merging

6. **Handle merge conflicts**
   - If conflicts exist: `git fetch origin main && git rebase origin/main`
   - Resolve any conflicts
   - Push the rebased branch: `git push --force-with-lease`

7. **Merge the PR**

   ```bash
   gh pr merge --squash --delete-branch
   ```

8. **Continue monitoring** - The workflow will auto-retry

---

### On `stage_complete`

```json
{"event_type":"stage_complete","play_id":"42","stage":"implementation","next_stage":"code-quality"}
```

Stage passed. Continue monitoring for the next stage.

---

### On `completed` → SUCCESS

```json
{"event_type":"completed","play_id":"42","duration_seconds":1847}
```

All stages passed. The play is done. Exit successfully.

---

### On `stopped` → RESET AND RETRY

```json
{
  "event_type": "stopped",
  "play_id": "42",
  "reason": "Max consecutive failures reached (5)"
}
```

Too many failures. Automatically reset and retry:

```bash
play-monitor reset --repo cto-parallel-test --org 5dlabs
play-monitor run --play-id <NEW_ID> --repository 5dlabs/cto-parallel-test
play-monitor loop --play-id <NEW_ID>
```

---

## Stage Sequence

```text
Rex/Blaze → Cleo → Cypher → Tess → Atlas
```

## Common Fixes by Stage

| Stage | Error Pattern | Automated Fix |
|-------|--------------|---------------|
| Cleo | `clippy::` errors | Apply clippy suggestions |
| Cleo | formatting | Run `cargo fmt` |
| Tess | test failures | Fix test assertions |
| Cypher | security warnings | Update deps |

## The Automated Flow

```text
┌─────────────────────────────────────────────────────┐
│  play-monitor loop --play-id X                      │
│                                                     │
│  ┌──────────────┐                                   │
│  │ Read Event   │◄─────────────────────────────┐    │
│  └──────┬───────┘                              │    │
│         │                                      │    │
│         ▼                                      │    │
│  ┌──────────────┐     ┌──────────────┐         │    │
│  │ failure?     │─YES─►│ Parse logs   │         │    │
│  └──────┬───────┘     │ Apply fix    │         │    │
│         │NO           │ Commit/Push  │─────────┘    │
│         ▼             └──────────────┘              │
│  ┌──────────────┐                                   │
│  │ completed?   │─YES─► EXIT SUCCESS                │
│  └──────┬───────┘                                   │
│         │NO                                         │
│         ▼                                           │
│  ┌──────────────┐                                   │
│  │ stopped?     │─YES─► Reset & Retry               │
│  └──────┬───────┘                                   │
│         │NO                                         │
│         └───────────────────────────────────────────┘
└─────────────────────────────────────────────────────┘
```

## Key Principle

**No waiting for humans.** When you see a failure:

1. Read the error
2. Fix it
3. Push it
4. Move on

The goal is unattended, autonomous progression from start to finish.
