# Implementation Plan: Current → Desired State

## Executive Summary

We have template code for Fresh Start and Subagent coordination, but the config values aren't parsed or passed through to templates. Linear Task Sync is documented but not implemented at all.

**Simplified Scope:** After review, we removed `workerIsolation` and `roleModels` as dead code:
- `workerIsolation` - Template existed but was never wired up; we don't have peer coordination anyway
- `roleModels` - Redundant with per-agent model config in `agents.*` section

**Remaining Items:**
1. **Wire up Fresh Start** - Pass `freshStartThreshold` from config to retry-loop template
2. **Wire up Subagents** - Pass `subagents` config and `subtasks` array to templates
3. **Remove `local=true` from intake** - Causes confusion in E2E tests
4. **Auto-append final deployment task** - Bolt's final task isn't guaranteed
5. **Clarify 6 implementation agents** - Documentation needs updating
6. **Linear Task Sync** - `intake update` and `intake sync-task` commands

## Gap Analysis

| Feature | Current State | Work Required |
|---------|---------------|---------------|
| **Fresh Start** | Template code exists, never triggers | Wire up config → template |
| **Subagents** | Config & templates exist, not wired | Pass to template context |
| **Linear Task Sync** | Design doc only | Full implementation |
| **Intake local option** | Exists, causes confusion | Remove from MCP tool |
| **Final Deploy Task** | PRD-dependent | Auto-append in intake |

---

## Phase 0: Intake Clarifications (Critical)

**Goal:** Remove confusion about intake and ensure deployment task is always created.

### 0.1 Remove `local=true` from MCP Tool

**File:** `crates/notify/mcp/src/main.rs`

```rust
// DELETE this function entirely:
fn handle_intake_local(arguments: &HashMap<String, Value>) -> Result<Value> {
    // ... all of this should be removed
}

// DELETE the local handling in the main intake handler:
// if local {
//     return handle_intake_local(arguments);
// }
```

**File:** `crates/notify/mcp/src/tools.rs`

```rust
// REMOVE from tool schema:
// "local": {
//     "description": "Run intake locally...",
//     "type": "boolean"
// }
```

### 0.2 Auto-Append Final Deployment Task

**File:** `crates/intake/src/bin/cli.rs`

After the existing Task 1 enforcement (around line 1145):

```rust
// Task 1 MUST always be bolt (infrastructure) - EXISTING CODE
if let Some(task1) = tasks.iter_mut().find(|t| t.id == "1") {
    if task1.agent_hint.as_deref() != Some("bolt") {
        task1.agent_hint = Some("bolt".to_string());
    }
}

// NEW: Final task MUST always be bolt (deployment)
let max_id: u32 = tasks.iter()
    .filter_map(|t| t.id.parse::<u32>().ok())
    .max()
    .unwrap_or(0);

let has_deploy_task = tasks.iter().any(|t| {
    t.title.to_lowercase().contains("deploy") && 
    t.agent_hint.as_deref() == Some("bolt")
});

if !has_deploy_task {
    let deploy_task_id = (max_id + 1).to_string();
    let all_task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
    
    let deploy_task = Task {
        id: deploy_task_id,
        title: "Deploy to Production (Bolt - Deployment)".to_string(),
        description: "Deploy application to production and verify public accessibility".to_string(),
        status: TaskStatus::Pending,
        dependencies: all_task_ids,
        priority: "high".to_string(),
        details: Some("Configure DNS, health probes, telemetry. Verify public access.".to_string()),
        test_strategy: Some("Verify endpoints are accessible and healthy".to_string()),
        agent_hint: Some("bolt".to_string()),
        ..Default::default()
    };
    
    tracing::info!("Auto-appended final deployment task: {}", deploy_task.id);
    tasks.push(deploy_task);
}
```

**Estimated effort:** 2-4 hours

## Phase 1: Wire Up Fresh Start (Easiest)

**Goal:** Make Fresh Start actually trigger from config.

### 1.1 Add Config Field

**File:** `crates/config/src/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlayDefaults {
    // ... existing fields ...

    /// Retry count before triggering fresh start (default: 3)
    #[serde(rename = "freshStartThreshold", default = "default_fresh_start_threshold")]
    pub fresh_start_threshold: u32,
}

fn default_fresh_start_threshold() -> u32 { 3 }
```

### 1.2 Pass to Templates

**File:** `crates/healer/src/prompt/context.rs` (or wherever PromptContext is defined)

Add field to the context struct that gets passed to Handlebars:

```rust
pub struct PromptContext {
    // ... existing fields ...
    pub fresh_start_threshold: u32,
}
```

This will make the `{{fresh_start_threshold}}` variable in `retry-loop.sh.hbs` actually get populated.

### 1.3 Test

```bash
# Verify config parsing
cargo test -p cto-config -- fresh_start

# Verify template rendering
cargo test -p cto-healer -- render_prompt
```

**Estimated effort:** 1-2 hours

---

## Phase 1.5: Wire Up Subagents

**Goal:** Enable parallel subtask execution via subagents when configured.

### Background

Subagents allow a single task to be broken into parallelizable subtasks that run concurrently:
- **Intake** generates subtasks with `subagent_type`, `execution_level`, `parallelizable`
- **Templates** exist (`coordinator.md.hbs`, `subagent-dispatch.md.hbs`) but never receive the data
- **Config** exists (`agents.*.subagents.enabled`, `agents.*.subagents.maxConcurrent`)

When `subagents.enabled = false` (default), agent works on task sequentially.
When `subagents.enabled = true`, agent acts as coordinator, dispatching subtasks to parallel workers.

### CLI Support Status

| CLI | Subagents? | How |
|-----|------------|-----|
| **Claude Code** | ✅ Yes | Native `@agent_name` mentions |
| **OpenCode** | ✅ Yes | Native `mode: subagent` config |
| **Cursor** | ❓ TBD | Needs verification |
| **Factory** | ❓ TBD | Needs verification |
| **Codex** | ❓ TBD | Needs verification |
| **Gemini** | ❓ TBD | Needs verification |

### 1.5.1 Update `should_use()` for OpenCode

**File:** `crates/config/src/types.rs`

```rust
impl SubagentConfig {
    /// Check if subagents should be used (enabled and valid CLI).
    #[must_use]
    pub fn should_use(&self, cli: &str) -> bool {
        // Currently only Claude and OpenCode support subagents
        self.enabled && matches!(cli, "claude" | "opencode")
    }
}
```

### 1.5.2 Pass Subagent Config to Template Context

**File:** `crates/controller/src/tasks/code/templates.rs`

In `enrich_cli_config_from_agent()` around line 1744:

```rust
if let Some(agent_config) = config.agents.get(&agent_name) {
    // Existing model_rotation and frontend_stack handling...
    
    // NEW: Inject subagent config if present
    if let Some(subagents) = &agent_config.subagents {
        enriched["subagents"] = json!({
            "enabled": subagents.enabled,
            "maxConcurrent": subagents.max_concurrent
        });
    }
}
```

### 1.5.3 Pass Subtasks to Template Context

The subtasks array needs to come from the task file. In each `generate_*_memory()` function:

```rust
let context = json!({
    // ... existing fields ...
    "subagents": cli_config.get("subagents").cloned().unwrap_or(json!({"enabled": false})),
    "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
});
```

**Note:** `CodeRunSpec` needs a `subtasks` field added:

```rust
// crates/controller/src/crds/coderun.rs
pub struct CodeRunSpec {
    // ... existing fields ...
    
    /// Subtasks for parallel execution (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtasks: Option<Vec<SubtaskSpec>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubtaskSpec {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "subagentType")]
    pub subagent_type: Option<String>,
    #[serde(rename = "executionLevel")]
    pub execution_level: Option<u32>,
    pub parallelizable: bool,
    pub dependencies: Vec<String>,
}
```

### 1.5.4 Register `group_by` Handlebars Helper

The `subagent-dispatch.md.hbs` template uses `{{#each (group_by subtasks "execution_level")}}`.

**File:** `crates/controller/src/tasks/code/templates.rs`

In `register_template_helpers()`:

```rust
// Helper for grouping arrays by a field: {{#each (group_by array "field")}}
handlebars.register_helper(
    "group_by",
    Box::new(|h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
        // Implementation: group array items by specified field
        // Returns object keyed by field value
    })
);
```

### 1.5.5 Test

```bash
# Verify config parsing
cargo test -p cto-config -- subagent

# Verify template rendering with subtasks
cargo test -p cto-controller -- render_with_subtasks
```

**Estimated effort:** 4-6 hours

---

## ~~Phase 2: Role-Specific Models~~ REMOVED

**Reason:** Redundant with per-agent model config in `agents.*` section of `cto-config.json`.

Each agent already has its own `model` field:
```json
"agents": {
  "morgan": { "model": "claude-opus-4-5-20251101" },
  "rex": { "model": "claude-opus-4-5-20251101" },
  ...
}
```

No need for an additional `roleModels` abstraction.

---

## ~~Worker Isolation~~ REMOVED

**Reason:** Dead code. We don't have peer coordination - each task runs in its own CodeRun pod independently. The template code existed but was never wired up, and would have had no effect anyway.

---

## Phase 2: Intake Update Functionality (Most Important)

**Goal:** Allow mid-flight task updates from PRD/Architecture changes OR Linear edits.

### Key Insight

Agents clone the docs repo fresh on every run. So we don't need a "sync step" in the workflow.
We just need to UPDATE the task files, and the next agent run automatically picks them up.

### Two Update Sources

| Source | When | Command |
|--------|------|---------|
| PRD/Architecture updated | User updates design docs | `intake update --project xyz` |
| Linear task manually edited | User changes task in Linear UI | `intake sync-task --issue LIN-123` |

Both produce the same output: **PR with updated task files**.

### 2.1 Command: `intake update`

Re-parse PRD/architecture and generate delta.

**File:** `crates/intake/src/commands/update.rs` (NEW)

```rust
#[derive(Args)]
pub struct UpdateArgs {
    /// Project name (folder in docs repo)
    #[arg(long)]
    project: String,
    
    /// Path to updated PRD (optional, defaults to project/prd.md)
    #[arg(long)]
    prd: Option<PathBuf>,
    
    /// Path to updated architecture (optional)
    #[arg(long)]
    architecture: Option<PathBuf>,
    
    /// GitHub repo for the docs
    #[arg(long)]
    docs_repo: String,
}

pub async fn update_project(args: UpdateArgs) -> Result<()> {
    // 1. Clone docs repo
    let repo = clone_docs_repo(&args.docs_repo)?;
    
    // 2. Load existing tasks
    let existing_tasks = load_tasks(&repo.join(&args.project))?;
    
    // 3. Parse updated PRD/architecture
    let updated_prd = read_prd(&args.prd.unwrap_or(repo.join(&args.project).join("prd.md")))?;
    let new_tasks = parse_prd(&updated_prd)?;
    
    // 4. Generate delta (what changed)
    let delta = diff_tasks(&existing_tasks, &new_tasks)?;
    
    // 5. Update task files for changed tasks only
    for task in delta.changed {
        generate_task_files(&repo, &args.project, &task)?;
    }
    
    // 6. Create PR with changes
    create_update_pr(&repo, &delta)?;
    
    Ok(())
}
```

### 2.2 Command: `intake sync-task`

Pull task changes from Linear manual edits.

**File:** `crates/intake/src/commands/sync.rs` (NEW)

```rust
#[derive(Args)]
pub struct SyncTaskArgs {
    /// Linear issue ID
    #[arg(long)]
    issue_id: String,
    
    /// Project name in docs repo
    #[arg(long)]
    project: String,
    
    /// Task ID (e.g., "5" for task-5/)
    #[arg(long)]
    task_id: String,
    
    /// GitHub docs repo
    #[arg(long)]
    docs_repo: String,
}

pub async fn sync_task(args: SyncTaskArgs) -> Result<()> {
    // 1. Query Linear for current issue state
    let issue = linear_client.get_issue(&args.issue_id).await?;
    
    // 2. Parse issue description into task components
    let task = parse_linear_issue(&issue)?;
    
    // 3. Clone docs repo
    let repo = clone_docs_repo(&args.docs_repo)?;
    
    // 4. Compare with existing task files
    let task_dir = repo.join(&args.project).join(format!("task-{}", args.task_id));
    let existing = load_task_files(&task_dir)?;
    
    if task == existing {
        tracing::info!("No changes detected in Linear issue");
        return Ok(());
    }
    
    // 5. Regenerate task files
    write_task_files(&task_dir, &task)?;
    
    // 6. Create PR
    create_sync_pr(&repo, &args.task_id, &issue.title)?;
    
    Ok(())
}
```

### 2.3 Linear Issue Parsing

**File:** `crates/intake/src/domain/linear_parser.rs` (NEW)

```rust
pub struct ParsedLinearTask {
    pub title: String,
    pub description: String,  // Main content → prompt.md
    pub acceptance_criteria: Vec<String>,  // Checkboxes → acceptance.md
    pub labels: Vec<String>,
}

pub fn parse_linear_issue(issue: &LinearIssue) -> Result<ParsedLinearTask> {
    // 1. Extract title
    let title = issue.title.clone();
    
    // 2. Parse description - split into main content and acceptance criteria
    let (description, criteria) = split_description(&issue.description)?;
    
    // 3. Extract acceptance criteria from checkboxes
    // Format: - [ ] Criterion text
    let acceptance_criteria = extract_checkboxes(&criteria);
    
    // 4. Get labels
    let labels = issue.labels.iter().map(|l| l.name.clone()).collect();
    
    Ok(ParsedLinearTask {
        title,
        description,
        acceptance_criteria,
        labels,
    })
}

fn extract_checkboxes(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.trim().starts_with("- [ ]") || line.trim().starts_with("- [x]"))
        .map(|line| {
            line.trim()
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim()
                .to_string()
        })
        .collect()
}
```

### 2.4 MCP Tool for Update

**File:** `crates/notify/mcp/src/tools.rs`

```rust
// Add new tool: intake_update
{
    "name": "intake_update",
    "description": "Update tasks from modified PRD/architecture. Creates PR with delta.",
    "parameters": {
        "project_name": { "type": "string", "required": true },
        "prd_content": { "type": "string", "description": "Optional: new PRD content" },
        "architecture_content": { "type": "string", "description": "Optional: new architecture" }
    }
}

// Add new tool: intake_sync_task
{
    "name": "intake_sync_task", 
    "description": "Sync task files from Linear issue edits. Creates PR with updated task.",
    "parameters": {
        "issue_id": { "type": "string", "required": true },
        "project_name": { "type": "string", "required": true },
        "task_id": { "type": "string", "required": true }
    }
}
```

### 2.5 Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    INTAKE UPDATE FLOW                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   SOURCE A: PRD/Architecture         SOURCE B: Linear Edits         │
│   ┌──────────────────────┐          ┌──────────────────────┐       │
│   │ User updates         │          │ User edits task      │       │
│   │ prd.md or           │          │ description in       │       │
│   │ architecture.md     │          │ Linear UI            │       │
│   └──────────┬───────────┘          └──────────┬───────────┘       │
│              │                                  │                   │
│              ▼                                  ▼                   │
│   ┌──────────────────────┐          ┌──────────────────────┐       │
│   │ intake update        │          │ intake sync-task     │       │
│   │ --project xyz        │          │ --issue LIN-123      │       │
│   │                      │          │ --task-id 5          │       │
│   └──────────┬───────────┘          └──────────┬───────────┘       │
│              │                                  │                   │
│              ▼                                  ▼                   │
│   ┌──────────────────────┐          ┌──────────────────────┐       │
│   │ Re-parse PRD         │          │ Fetch issue from     │       │
│   │ Compare to existing  │          │ Linear GraphQL API   │       │
│   │ Generate DELTA       │          │ Parse description    │       │
│   └──────────┬───────────┘          └──────────┬───────────┘       │
│              │                                  │                   │
│              └────────────────┬─────────────────┘                   │
│                               │                                     │
│                               ▼                                     │
│                    ┌──────────────────────┐                        │
│                    │ Create PR with       │                        │
│                    │ updated task files   │                        │
│                    │ in docs repo         │                        │
│                    └──────────┬───────────┘                        │
│                               │                                     │
│                               ▼                                     │
│                    ┌──────────────────────┐                        │
│                    │ Morgan reviews PR    │                        │
│                    │ (same as initial)    │                        │
│                    └──────────┬───────────┘                        │
│                               │                                     │
│                               ▼                                     │
│                    ┌──────────────────────┐                        │
│                    │ Next agent run       │                        │
│                    │ clones fresh →       │                        │
│                    │ sees updated files   │                        │
│                    └──────────────────────┘                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Estimated effort:** 3-5 days

---

## Implementation Order

| Priority | Phase | Effort | Value |
|----------|-------|--------|-------|
| 1 | **Phase 0**: Intake fixes (local removal, auto-deploy task) | 2-4 hours | Critical |
| 2 | **Phase 1**: Fresh Start wiring | 1-2 hours | Medium |
| 3 | **Phase 1.5**: Subagent wiring | 4-6 hours | High |
| 4 | **Phase 2**: Intake Update functionality | 3-5 days | **Highest** |

## Testing Strategy

### Phase 0 Tests
```bash
# Verify local option removed
cargo test -p cto-notify-mcp -- intake_no_local

# Verify deploy task auto-appended
cargo test -p cto-intake -- auto_deploy_task
```

### Phase 1 Tests
```bash
# Unit: Config parsing
cargo test -p cto-config -- fresh_start

# Integration: Template rendering
cargo test -p cto-healer -- render_with_fresh_start
```

### Phase 1.5 Tests
```bash
# Unit: Config parsing for subagents
cargo test -p cto-config -- subagent

# Unit: should_use() for claude and opencode
cargo test -p cto-config -- subagent_should_use

# Integration: Template rendering with subtasks
cargo test -p cto-controller -- render_with_subtasks

# Integration: group_by helper
cargo test -p cto-controller -- group_by_helper
```

### Phase 2 Tests
```bash
# Unit: Linear issue parsing
cargo test -p cto-intake -- parse_linear_issue

# Unit: Task delta generation
cargo test -p cto-intake -- diff_tasks

# Integration: Full update flow
cargo test -p cto-intake -- update_integration

# Integration: Sync from Linear
cargo test -p cto-intake -- sync_task_integration
```

## Files to Modify

| Phase | File | Changes |
|-------|------|---------|
| 0 | `crates/notify/mcp/src/main.rs` | Remove `handle_intake_local()` |
| 0 | `crates/notify/mcp/src/tools.rs` | Remove `local` param from intake |
| 0 | `crates/intake/src/bin/cli.rs` | Auto-append deploy task |
| 1 | `crates/config/src/types.rs` | Add fresh_start_threshold |
| 1 | `crates/healer/src/prompt/context.rs` | Pass value to templates |
| 1.5 | `crates/config/src/types.rs` | Update `should_use()` for opencode |
| 1.5 | `crates/controller/src/tasks/code/templates.rs` | Pass subagents + subtasks to context |
| 1.5 | `crates/controller/src/crds/coderun.rs` | Add `subtasks` field to CodeRunSpec |
| 1.5 | `crates/controller/src/tasks/code/templates.rs` | Register `group_by` helper |
| 2 | `crates/intake/src/commands/update.rs` | NEW: update command |
| 2 | `crates/intake/src/commands/sync.rs` | NEW: sync-task command |
| 2 | `crates/intake/src/domain/linear_parser.rs` | NEW: Linear issue parsing |
| 2 | `crates/intake/src/domain/delta.rs` | NEW: task diff logic |
| 2 | `crates/notify/mcp/src/tools.rs` | Add intake_update, intake_sync_task |

## Definition of Done

- [ ] Phase 0: `local=true` removed from MCP intake
- [ ] Phase 0: Deploy task auto-appended to all projects
- [ ] Phase 1: Fresh Start triggers after configured threshold
- [ ] Phase 1.5: Subagent config passed to templates when enabled
- [ ] Phase 1.5: Subtasks array rendered in coordinator prompts
- [ ] Phase 1.5: `group_by` helper works for execution level grouping
- [ ] Phase 2: `intake update` re-parses PRD and creates delta PR
- [ ] Phase 2: `intake sync-task` pulls Linear edits and creates PR
- [ ] Phase 2: MCP tools available for both operations
- [ ] All tests pass
- [ ] Documentation updated
