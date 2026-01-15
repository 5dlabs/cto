# Linear → Task Sync: Mid-Flight Task Updates

## Problem Statement

Currently, task definitions are **static** once a Play workflow starts:
1. Intake generates task files in docs repo
2. Play reads these files once at workflow start
3. If requirements change mid-flight, there's no way to update without restarting

**Reality:** Plans rarely survive first contact. We need the ability to adjust task definitions while a workflow is running.

## Desired Behavior

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LINEAR AS SOURCE OF TRUTH                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Linear Issue                    Task Files                         │
│   ┌──────────────┐               ┌──────────────┐                   │
│   │ Title        │──────────────▶│ prompt.md    │                   │
│   │ Description  │    sync       │ prompt.xml   │                   │
│   │ Acceptance   │──────────────▶│ acceptance   │                   │
│   │ Labels       │               │ .md          │                   │
│   └──────────────┘               └──────────────┘                   │
│         │                              │                             │
│         │  ✏️ User edits               │  📖 Agent reads             │
│         │  in Linear                   │  from files                 │
│         ▼                              ▼                             │
│   ┌──────────────┐               ┌──────────────┐                   │
│   │ Updated      │    re-sync    │ Updated      │                   │
│   │ requirements │──────────────▶│ task files   │                   │
│   └──────────────┘               └──────────────┘                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## When Sync Should Happen

| Trigger | Description |
|---------|-------------|
| **Before each agent stage** | Check Linear for updates before Implementation, Quality, Security, Testing, Integration |
| **On explicit request** | Morgan or operator can trigger a sync |
| **On webhook** | Linear issue update webhook could trigger sync |

## What Gets Synced

| Linear Field | Task File | Notes |
|--------------|-----------|-------|
| Issue Title | `prompt.md` title | Task name |
| Issue Description | `prompt.md` body, `prompt.xml` | Main task definition |
| Acceptance Criteria (checklist) | `acceptance.md` | Checkboxes in description |
| Labels | Metadata | Agent routing, priority |
| Assignee | Metadata | Which agent to use |
| Comments | Context? | Optional: include as additional context |

## Implementation Components

### 1. Linear Task Fetcher (New)

**Location:** `crates/pm/src/linear/task_sync.rs`

```rust
/// Fetch current task state from Linear
pub async fn fetch_task_from_linear(
    issue_id: &str,
    linear_client: &LinearClient,
) -> Result<LinearTaskState> {
    // Query Linear GraphQL API for:
    // - Issue title
    // - Issue description (markdown)
    // - Checklist items (acceptance criteria)
    // - Labels
    // - Current state
}

/// Compare Linear state with existing task files
pub fn detect_changes(
    linear_state: &LinearTaskState,
    task_files: &TaskFiles,
) -> TaskDiff {
    // Return what changed:
    // - description_changed: bool
    // - acceptance_changed: bool  
    // - title_changed: bool
}

/// Regenerate task files from Linear state
pub async fn sync_task_files(
    issue_id: &str,
    task_dir: &Path,
    linear_state: &LinearTaskState,
) -> Result<()> {
    // Write updated prompt.md
    // Write updated prompt.xml
    // Write updated acceptance.md
}
```

### 2. Pre-Stage Sync Step (Workflow Template)

**Location:** `infra/gitops/manifests/argo-workflows/play-workflow-template.yaml`

Add a sync step before each agent stage:

```yaml
- name: sync-task-from-linear
  inputs:
    parameters:
      - name: task-id
      - name: linear-issue-id
  script:
    image: ghcr.io/5dlabs/pm:latest
    command: [pm]
    args:
      - task
      - sync
      - --issue-id
      - "{{inputs.parameters.linear-issue-id}}"
      - --task-dir
      - "/workspace/task"
      - --if-changed-only
```

### 3. PM CLI Command (New)

**Location:** `crates/pm/src/commands/task.rs`

```bash
# Sync task files from Linear
pm task sync --issue-id LIN-123 --task-dir ./task

# Check if task has changes (for conditional execution)
pm task check-changes --issue-id LIN-123 --task-dir ./task
# Exit 0 if changes, exit 1 if no changes

# Force regenerate task files
pm task regenerate --issue-id LIN-123 --task-dir ./task --force
```

### 4. Linear Webhook Handler (Optional Enhancement)

**Location:** `crates/pm/src/webhooks/linear.rs`

```rust
/// Handle Linear issue update webhook
pub async fn handle_issue_update(
    payload: LinearWebhookPayload,
) -> Result<()> {
    // If issue is part of active Play workflow:
    // 1. Trigger task sync
    // 2. Optionally notify running agent
}
```

### 5. Agent Notification (Optional)

If an agent is mid-execution and the task changes:
- Option A: Let current execution finish, sync before retry
- Option B: Send interrupt signal to agent with updated context
- **Recommended:** Option A (simpler, less disruptive)

## Data Flow

```
1. User edits Linear issue description
   │
   ▼
2. Play workflow reaches next stage
   │
   ▼
3. Pre-stage: pm task sync --issue-id LIN-123
   │
   ├─▶ Query Linear API for current state
   ├─▶ Compare with existing task files
   ├─▶ If changed: regenerate task files
   │
   ▼
4. Agent stage runs with updated task files
   │
   ▼
5. Agent reads fresh prompt.md, acceptance.md
```

## Edge Cases

| Scenario | Handling |
|----------|----------|
| Linear offline | Use cached task files, log warning |
| Sync fails mid-workflow | Continue with existing files, flag for retry |
| Conflicting changes | Linear wins (source of truth) |
| Task completed but changed | Log warning, don't re-run completed stages |
| Acceptance criteria removed | Update files, agent sees new criteria |

## Configuration

Add to `cto-config.json`:

```json
{
  "defaults": {
    "play": {
      "enableTaskSync": true,
      "syncBeforeStages": ["implementation", "quality", "security", "testing"],
      "syncOnWebhook": false
    }
  }
}
```

## Migration Path

1. **Phase 1:** Manual sync command (`pm task sync`)
2. **Phase 2:** Automatic pre-stage sync in workflow template
3. **Phase 3:** (Optional) Webhook-triggered sync for real-time updates

## Acceptance Criteria for This Feature

- [ ] `pm task sync` command implemented
- [ ] `pm task check-changes` command implemented
- [ ] Linear GraphQL query for issue details works
- [ ] Task files regenerated correctly from Linear state
- [ ] Workflow template includes sync step before agent stages
- [ ] Sync is skipped if no changes detected (efficiency)
- [ ] Config option to enable/disable sync
- [ ] Logging shows when sync happens and what changed
- [ ] Existing workflows continue to work (backward compatible)

## Related Files

- `crates/pm/src/linear/` - Linear API client
- `crates/pm/src/handlers/play.rs` - Play workflow handler
- `infra/gitops/manifests/argo-workflows/play-workflow-template.yaml` - Workflow template
- `templates/agents/*/` - Agent prompt templates

## Out of Scope (Future)

- **Dynamic task creation:** Morgan creating new tasks mid-workflow (continuous planning)
- **Task splitting:** Breaking a task into subtasks during execution
- **Rollback:** Reverting to previous task definition
