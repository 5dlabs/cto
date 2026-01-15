# Implementation Plan: Current → Desired State

## Executive Summary

We have template code for Fresh Start and Worker Isolation, but the config values aren't parsed or passed through. Linear Task Sync is documented but not implemented at all.

Additionally, we need to:
1. **Remove `local=true` option from intake** - causes confusion in E2E tests
2. **Auto-append final deployment task** - Bolt's final task isn't guaranteed
3. **Clarify 6 implementation agents** - documentation needs updating

## Gap Analysis

| Feature | Current State | Work Required |
|---------|---------------|---------------|
| **Fresh Start** | Template code exists, never triggers | Wire up config → template |
| **Worker Isolation** | Template code exists, never activates | Wire up config → template |
| **Role Models** | Not implemented | New config struct + routing |
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

## Phase 1: Wire Up Existing Code (Easiest)

**Goal:** Make Fresh Start and Worker Isolation actually work.

### 1.1 Add Config Fields

**File:** `crates/config/src/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlayDefaults {
    // ... existing fields ...

    /// Retry count before triggering fresh start (default: 3)
    #[serde(rename = "freshStartThreshold", default = "default_fresh_start_threshold")]
    pub fresh_start_threshold: u32,

    /// Enable worker isolation mode (default: true)
    #[serde(rename = "workerIsolation", default = "default_worker_isolation")]
    pub worker_isolation: bool,
}

fn default_fresh_start_threshold() -> u32 { 3 }
fn default_worker_isolation() -> bool { true }
```

### 1.2 Pass to Templates

**File:** `crates/healer/src/prompt/context.rs` (or wherever PromptContext is defined)

Add fields to the context struct that gets passed to Handlebars:

```rust
pub struct PromptContext {
    // ... existing fields ...
    pub fresh_start_threshold: u32,
    pub worker_isolation: bool,
}
```

### 1.3 Test

```bash
# Verify config parsing
cargo test -p cto-config -- fresh_start

# Verify template rendering
cargo test -p cto-healer -- render_prompt
```

**Estimated effort:** 2-4 hours

---

## Phase 2: Role-Specific Models (Medium)

**Goal:** Different models for planner/worker/reviewer roles.

### 2.1 Add RoleModels Struct

**File:** `crates/config/src/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RoleModels {
    /// Model for Morgan (planning)
    #[serde(default)]
    pub planner: Option<String>,
    
    /// Model for implementation agents (Rex, Blaze, etc.)
    #[serde(default)]
    pub worker: Option<String>,
    
    /// Model for Cleo (quality review)
    #[serde(default)]
    pub reviewer: Option<String>,
}

// Add to PlayDefaults:
#[serde(rename = "roleModels", default)]
pub role_models: RoleModels,
```

### 2.2 Add Role Detection

**File:** `crates/healer/src/workflow/agent.rs`

```rust
pub enum AgentRole {
    Planner,   // Morgan
    Worker,    // Rex, Blaze, Grizz, Nova, Tap, Spark, Bolt
    Reviewer,  // Cleo, Cipher, Tess
    Integrator, // Atlas
}

impl AgentRole {
    pub fn from_agent_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            n if n.contains("morgan") => Self::Planner,
            n if n.contains("cleo") || n.contains("cipher") || n.contains("tess") => Self::Reviewer,
            n if n.contains("atlas") => Self::Integrator,
            _ => Self::Worker,
        }
    }
}
```

### 2.3 Override Model Selection

When starting an agent, check role and override model:

```rust
fn get_model_for_agent(agent: &str, config: &PlayDefaults) -> String {
    let role = AgentRole::from_agent_name(agent);
    match role {
        AgentRole::Planner => config.role_models.planner.clone(),
        AgentRole::Worker => config.role_models.worker.clone(),
        AgentRole::Reviewer => config.role_models.reviewer.clone(),
        AgentRole::Integrator => None, // Use default
    }.unwrap_or_else(|| config.get_default_model())
}
```

**Estimated effort:** 4-6 hours

---

## Phase 3: Intake Update Functionality (Most Important)

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

### 3.1 Command: `intake update`

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

### 3.2 Command: `intake sync-task`

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

### 3.3 Linear Issue Parsing

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

### 3.4 MCP Tool for Update

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

### 3.5 Flow Diagram

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
| 2 | **Phase 1**: Fresh Start + Worker Isolation | 2-4 hours | High |
| 3 | **Phase 3**: Intake Update functionality | 3-5 days | **Highest** |
| 4 | **Phase 2**: Role Models | 4-6 hours | Medium |

Note: Phase 3 is reordered before Phase 2 because mid-flight updates is the most valuable feature.

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
cargo test -p cto-config -- fresh_start worker_isolation

# Integration: Template rendering
cargo test -p cto-healer -- render_with_fresh_start
```

### Phase 2 Tests
```bash
# Unit: Role detection
cargo test -p cto-healer -- agent_role

# Unit: Model routing
cargo test -p cto-healer -- role_model_routing
```

### Phase 3 Tests
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
| 1 | `crates/config/src/types.rs` | Add fresh_start_threshold, worker_isolation |
| 1 | `crates/healer/src/prompt/context.rs` | Pass values to templates |
| 2 | `crates/config/src/types.rs` | Add RoleModels struct |
| 2 | `crates/healer/src/workflow/agent.rs` | Add role detection + routing |
| 3 | `crates/intake/src/commands/update.rs` | NEW: update command |
| 3 | `crates/intake/src/commands/sync.rs` | NEW: sync-task command |
| 3 | `crates/intake/src/domain/linear_parser.rs` | NEW: Linear issue parsing |
| 3 | `crates/intake/src/domain/delta.rs` | NEW: task diff logic |
| 3 | `crates/notify/mcp/src/tools.rs` | Add intake_update, intake_sync_task |

## Definition of Done

- [ ] Phase 0: `local=true` removed from MCP intake
- [ ] Phase 0: Deploy task auto-appended to all projects
- [ ] Phase 1: Fresh Start triggers after configured threshold
- [ ] Phase 1: Worker Isolation shows focused prompts
- [ ] Phase 2: Morgan uses planner model, Rex uses worker model  
- [ ] Phase 3: `intake update` re-parses PRD and creates delta PR
- [ ] Phase 3: `intake sync-task` pulls Linear edits and creates PR
- [ ] Phase 3: MCP tools available for both operations
- [ ] All tests pass
- [ ] Documentation updated
