# Cross-CLI Resume and Workflow State Management - Comprehensive Analysis

## Executive Summary

The system uses a **multi-layered state persistence architecture** designed to support cross-CLI workflow resume functionality:

1. **Workflow-Level State** (Kubernetes Labels): Tracks overall workflow progression through stages
2. **Agent-Level State** (ConfigMaps + PVCs): Tracks per-agent session state
3. **Task-Level State** (ConfigMaps): Tracks multi-agent task completion and staging progress
4. **CLI-Specific State** (PVCs + Environment): Tracks individual CLI session continuity

This enables seamless resume when workflows are deleted and restarted, or when transitioning between CLI agents.

---

## 1. WORKFLOW-LEVEL STATE TRACKING (Kubernetes Labels)

### Stage Progression Model
**Location**: `stage-transitions-template.yaml`, `stage-aware-*-sensor.yaml`

Workflows progress through **named stages** stored as Kubernetes labels:

```yaml
current-stage: "implementation"        # Workflow is at Rex stage
current-stage: "waiting-pr-created"    # Waiting for event to progress
current-stage: "waiting-quality-complete"  # After Cleo approval
current-stage: "waiting-pr-merged"     # After Tess approval
```

### Stage Lifecycle

```
workflow created
    ↓
current-stage: "implementation"
    ↓ (Rex completes)
current-stage: "waiting-pr-created"
    ↓ (GitHub event: PR created)
current-stage: [continues in cleo-quality]
    ↓ (Cleo completes)
current-stage: "waiting-quality-complete"
    ↓ (GitHub event: Cleo approves)
current-stage: [continues in tess-testing]
    ↓ (Tess completes)
current-stage: "waiting-pr-merged"
    ↓ (GitHub event: PR merged)
workflow completes
```

### Workflow Label Structure
**File**: `stage-transitions-template.yaml` lines 38-60

```yaml
metadata:
  labels:
    task-id: "5"                    # Task being worked on
    repository: "5dlabs-cto"        # Repository (/ replaced with -)
    current-stage: "implementation" # Current execution stage
    updated-at: "2025-11-12T..."   # Last stage transition time
    workflow-type: "play-orchestration"
```

### Critical for Resume
When a workflow is deleted and recreated:
- **Sensors match workflows by labels**: `task-id`, `repository`, `workflow-type`
- **Stage label determines next action**: If `current-stage=waiting-pr-created`, only PR-created events resume it
- **Prevents incorrect stage progression**: Sensor explicitly checks stage before resuming

Example from `stage-aware-pr-created.yaml` lines 163-167:
```bash
CURRENT_STAGE=$(kubectl get workflow $WORKFLOW_NAME \
  -n agent-platform \
  -o jsonpath='{.metadata.labels.current-stage}')
if [ "$CURRENT_STAGE" = "waiting-pr-created" ]; then
  # Only resume if at correct stage
```

---

## 2. AGENT-LEVEL STATE STORAGE

### PVC Naming Strategy: CLI-Agnostic with Agent Awareness
**Location**: `controller/src/tasks/code/agent.rs`

#### Implementation Pattern (Rex → Blaze → Cleo → Cipher → Tess)

```
Agent Name Extraction: "5DLabs-Rex" → "rex"
                      "5DLabs-Cleo" → "cleo"
                      "5DLabs-Tess" → "tess"
```

#### PVC Allocation

```
Implementation Agents (Rex, Blaze):
  - Workspace: workspace-{service}        (SHARED)
  - Both agents work in same PVC
  - Access previous work from predecessor
  - Example: workspace-cto

Quality/Testing Agents (Cleo, Cipher, Tess):
  - Workspace: workspace-{service}-{agent}  (ISOLATED)
  - Each gets isolated workspace
  - Copy needed files from implementation workspace
  - Example: workspace-cto-cleo, workspace-cto-tess
```

**Code from `agent.rs` lines 82-97**:
```rust
pub fn get_pvc_name(&self, service: &str, github_app: &str) -> Result<String, String> {
    let agent_name = self.extract_agent_name(github_app)?;
    
    if self.is_implementation_agent(&agent_name) {
        // Implementation agents share workspace
        format!("workspace-{service}")
    } else {
        // Non-implementation agents get isolated workspaces
        format!("workspace-{service}-{agent_name}")
    }
}
```

### Workspace Persistence Across CLI Transitions

**PVC is PERSISTENT and NOT deleted** when:
- Workflow is deleted and recreated
- Agent transitions (Rex → Blaze → Cleo → etc.)
- CLI changes (Claude → Factory → Codex)

When a CodeRun CRD is created for a new workflow execution:
1. Controller retrieves or creates PVC (never deletes)
2. Agent mounts PVC at standard location
3. Previous work is immediately available

**Code from `resources.rs` lines 53-98**:
```rust
// Determine PVC name based on agent classification
let pvc_name = if let Some(github_app) = &code_run_ref.spec.github_app {
    classifier.get_pvc_name(service_name, github_app)?
} else {
    format!("workspace-{service_name}")
};

// Ensure PVC exists (idempotent - doesn't delete)
self.ensure_pvc_exists(&pvc_name, service_name, github_app).await?;
```

---

## 3. WORKFLOW ORCHESTRATION STATE (ConfigMaps)

### Play Project Progress Tracking
**Location**: `controller/src/tasks/play/progress.rs`

#### PlayProgress Structure
```rust
pub struct PlayProgress {
    pub repository: String,              // "5dlabs/cto"
    pub branch: String,                  // Current branch
    pub current_task_id: Option<u32>,    // Which task being worked
    pub workflow_name: Option<String>,   // Argo workflow name
    pub status: PlayStatus,              // in-progress|suspended|failed|completed
    pub stage: Option<String>,           // implementation|code-quality|qa|etc
    pub started_at: DateTime<Utc>,       // When workflow started
    pub last_updated: DateTime<Utc>,     // When last stage transition occurred
}
```

#### Storage Format
**ConfigMap Name**: `play-progress-{repo-with-slashes-replaced}`
- Example: `5dlabs/cto` → `play-progress-5dlabs-cto`
- **Namespace**: `agent-platform`
- **Label**: `play-tracking=true`

#### ConfigMap Data Keys
```yaml
repository: "5dlabs/cto"
branch: "main"
current-task-id: "5"
workflow-name: "play-task-5-workflow"
status: "in-progress"
stage: "implementation"
started-at: "2025-11-12T10:00:00Z"
last-updated: "2025-11-12T10:15:00Z"
```

#### Read/Write Operations
**Code from `progress.rs` lines 165-229**:

```rust
/// Read progress from ConfigMap
pub async fn read_progress(client: &Client, repo: &str) -> Result<Option<PlayProgress>>

/// Write or update progress to ConfigMap
pub async fn write_progress(client: &Client, progress: &PlayProgress) -> Result<()>

/// Clear progress for a repository
pub async fn clear_progress(client: &Client, repo: &str) -> Result<()>
```

---

## 4. CLI-SPECIFIC STATE PERSISTENCE

### Session State Management
**Location**: `controller/src/cli/session.rs`

#### SessionState Structure
```rust
pub struct SessionState {
    pub id: String,                              // Unique session ID
    pub cli_type: CLIType,                       // claude|codex|cursor|factory|etc
    pub universal_config: UniversalConfig,       // Shared config format
    pub created_at: chrono::DateTime<Utc>,       // Session start time
    pub last_active: chrono::DateTime<Utc>,      // Last activity
    pub cli_specific_state: serde_json::Value,   // CLI-specific data
    pub execution_history: Vec<ExecutionRecord>, // All operations in session
    pub status: SessionStatus,                   // active|executing|completed|etc
}

pub enum SessionStatus {
    Active,
    Executing,
    Completed,
    Failed,
    Terminated,
}
```

#### Cross-CLI Transitions
**Code from `session.rs` lines 285-318**:

```rust
pub async fn transition_cli(&self, session_id: &str, new_cli_type: CLIType) -> Result<()> {
    let mut session = self.get_session(session_id).await?
        .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;
    
    // Record the transition in history
    let transition_record = ExecutionRecord {
        timestamp: chrono::Utc::now(),
        task: format!("CLI transition: {:?} → {:?}", session.cli_type, new_cli_type),
        result: ExecutionResult { success: true, ... },
        duration_ms: 0,
    };
    
    session.cli_type = new_cli_type;
    session.execution_history.push(transition_record);
    session.last_active = chrono::Utc::now();
    
    self.persistence.save_session(&session).await?;
    Ok(())
}
```

### In-PVC Agent State Files
Stored on each agent's PVC:

**For Claude/Cleo/Tess**:
```
/workspace/
  CLAUDE.md          # Memory file (universal across all CLIs)
  .agent-state/      # CLI-specific state
    session-id.json
    execution-log.json
```

**Location**: Configured in CodeRun spec
```yaml
spec:
  continueSession: true         # Resume previous session
  overwriteMemory: false        # Don't erase CLAUDE.md
```

**Code from `templates.rs`**:
```rust
"CLAUDE.md".to_string(),           // Universal memory file
"continue_session": Self::get_continue_session(code_run),  // Resume flag
"overwrite_memory": code_run.spec.overwrite_memory,        // Memory reset flag
```

---

## 5. MULTI-AGENT WORKFLOW STATE PROGRESSION

### Stage Transitions Architecture
**File**: `stage-transitions-template.yaml`

#### DAG-Based Stage Progression

```yaml
multi-agent-workflow:
  dag:
    tasks:
    # STAGE 1: Implementation (Rex)
    - name: rex-implementation
      template: agent-coderun
      arguments:
        - github-app: "5DLabs-Rex"
    
    - name: update-to-waiting-pr
      dependencies: [rex-implementation]
      template: update-workflow-stage
      arguments:
        - new-stage: "waiting-pr-created"
    
    - name: wait-pr-created
      dependencies: [update-to-waiting-pr]
      template: suspend-for-event
      
    # STAGE 2: Quality (Cleo)
    - name: cleo-quality
      dependencies: [wait-pr-created]
      template: agent-coderun
      arguments:
        - github-app: "5DLabs-Cleo"
    
    - name: update-to-waiting-qa
      dependencies: [cleo-quality]
      template: update-workflow-stage
      arguments:
        - new-stage: "waiting-ready-for-qa"
    
    # ... continue for remaining stages
```

### Sensor-Based Event Correlation
**Files**: `stage-aware-*.yaml`

Sensors correlate GitHub events to workflow stage:

```yaml
# Only match workflows at correct stage
if [ "$CURRENT_STAGE" = "waiting-pr-created" ]; then
  kubectl patch workflow $WORKFLOW_NAME \
    --type='merge' \
    -p '{"spec":{"suspend":false}}'
fi
```

**Event → Stage Mapping**:
```
GitHub Event: pull_request.opened
    ↓ Extract task-id, repository labels
    ↓ Find workflow: task-id=$ID, repository=$REPO
    ↓ Check: current-stage == "waiting-pr-created"?
    ↓ YES: Resume workflow (suspend: false)
    ✓ Workflow continues to Cleo stage

GitHub Event: pull_request_review (Cleo[bot] approved)
    ↓ Extract task-id from PR labels
    ↓ Find workflow with matching labels
    ↓ Check: current-stage == "waiting-quality-complete"?
    ↓ YES: Resume using argo CLI
    ✓ Workflow continues to Tess stage
```

---

## 6. CROSS-CLI RESUME MECHANICS

### Scenario: Workflow Deleted & Recreated (Same Task)

**BEFORE (Original Workflow)**:
```
Workflow: play-task-5-abc123
  Labels:
    task-id: 5
    repository: 5dlabs-cto
    current-stage: waiting-pr-created
  Status: Suspended waiting for PR

PVCs:
  workspace-cto          (Rex/Blaze work here - PERSISTS)
  workspace-cto-cleo     (Cleo work here - PERSISTS)

ConfigMap:
  play-progress-5dlabs-cto
    current-task-id: 5
    stage: waiting-pr-created
    workflow-name: play-task-5-abc123
```

**DELETE WORKFLOW**:
```
kubectl delete workflow play-task-5-abc123
```

**RECREATE WORKFLOW** (same task):
```
kubectl create -f play-task-5-xyz789.yaml
  Labels:
    task-id: 5              ← SAME
    repository: 5dlabs-cto  ← SAME
    current-stage: waiting-pr-created  ← SAME (from ConfigMap)
```

**RESUME DETECTION** (in Sensor):
```bash
# Sensor processes GitHub event for PR #314
TASK_ID=5
REPO_LABEL=5dlabs-cto

# Find workflow by LABELS (not name)
WORKFLOW_NAME=$(kubectl get workflows -n agent-platform \
  -l task-id=$TASK_ID,repository=$REPO_LABEL,workflow-type=play-orchestration \
  -o jsonpath='{.items[0].metadata.name}')

# Get current stage from label
CURRENT_STAGE=$(kubectl get workflow $WORKFLOW_NAME \
  -o jsonpath='{.metadata.labels.current-stage}')

# Check if stage matches event expectation
if [ "$CURRENT_STAGE" = "waiting-pr-created" ]; then
  # Resume the NEW workflow
  kubectl patch workflow $WORKFLOW_NAME -p '{"spec":{"suspend":false}}'
fi
```

**RESULT**:
- New workflow resumes at exact stage where old one stopped
- PVCs unmounted but persistent → new workflow mounts them
- Previous work immediately available to agents
- **No data loss, no stage regression**

---

### Scenario: CLI Transition (Claude → Factory → Codex)

**FLOW**:

```
Claude (5DLabs-Rex) Creates CodeRun
  ↓ Controller creates Job
  ↓ Job mounts workspace-cto (PVC)
  ↓ Agent works, stores CLAUDE.md + session state
  ↓ CodeRun completes

Factory (5DLabs-Blaze) Creates CodeRun  (frontend task)
  ↓ Agent Name: "blaze"
  ↓ PVC Name: workspace-cto (SHARED - implementation)
  ↓ Mounts same PVC
  ↓ Can read Rex's work
  ↓ Can read CLAUDE.md from previous session
  ↓ continueSession: true → resumes from CLAUDE.md
  ↓ Builds on Rex's implementation

Codex (5DLabs-Cleo) Creates CodeRun  (code quality)
  ↓ Agent Name: "cleo"
  ↓ PVC Name: workspace-cto-cleo (ISOLATED)
  ↓ New, clean workspace
  ↓ Controller copies needed files from workspace-cto
  ↓ Runs quality checks
  ↓ Has own CLEO.md memory file

Claude (5DLabs-Tess) Creates CodeRun  (testing)
  ↓ Agent Name: "tess"
  ↓ PVC Name: workspace-cto-tess (ISOLATED)
  ↓ New, clean workspace
  ↓ Can reference implementation via ConfigMap
  ↓ Has own TESS.md memory file
```

**Key Architecture Decision**: 
- **Implementation agents (Rex/Blaze)** = Shared PVC = Handoff via filesystem
- **Quality/Testing agents (Cleo/Cipher/Tess)** = Isolated PVC = Handoff via annotations + event metadata

---

## 7. RESUME STATE ON WORKFLOW DELETION

### What PERSISTS (Does NOT get deleted)

1. **PVCs** (Persistent Volumes)
   - `workspace-{service}` (Implementation workspace)
   - `workspace-{service}-{agent}` (Agent-specific workspaces)
   - All agent work preserved

2. **ConfigMaps** (Workflow orchestration state)
   - `play-progress-{repo}` (Current task, stage, workflow name)
   - Agent-specific ConfigMaps (if any)

3. **Secrets** (Credentials, tokens)
   - GitHub App tokens
   - MCP server credentials
   - Preserved for new workflow

### What GETS DELETED (By Kubernetes TTL)

1. **Workflow CRD** (Deleted by user or TTL)
   - Label metadata preserved in ConfigMap
   - Stage information recoverable

2. **Associated Pods** (Deleted by workflow)
   - Only compute, not state

3. **Job objects** (Garbage collected)

### Recovery Process

When new Workflow starts for same task:

```
1. New Workflow created with same labels (from ConfigMap)
2. Controller reads ConfigMap for previous stage
3. Update workflow.metadata.labels.current-stage = previous stage
4. Create CodeRun CRD
5. Controller mounts EXISTING PVCs (no recreation)
6. Agent starts, loads previous session from PVC
7. continueSession: true → Resume from CLAUDE.md
8. GitHub event sensor matches workflow by labels + stage
9. Event triggers resume if stage matches expectation
10. Workflow continues from exact suspension point
```

---

## 8. TASK-LEVEL RESUME STATE

### Multi-Task Workflow Coordination
**File**: `play-project-workflow-template.yaml`

#### SharedVolume for Progress Tracking

```yaml
volumeClaimTemplates:
  - metadata:
      name: morgan-pm-workspace
    spec:
      accessModes: ["ReadWriteOnce"]
      storage: 1Gi
```

#### Current Task Marker

```yaml
current-task-marker.txt contains:
  task-id: 5
  stage: waiting-pr-created
  workflow-name: play-task-5-abc123
  previous-tasks: [1, 2, 3, 4]
```

#### Resume from Specific Task

```yaml
arguments:
  - name: start-from-task
    value: "5"         # Start/resume from task 5, not task 1
  - name: max-tasks
    value: "100"       # Process up to 100 tasks
```

---

## 9. CRITICAL RESUME LOGIC

### Condition for Successful Resume

**ALL must be true**:

1. ✅ **Same task-id**: New workflow has `task-id=5` label
2. ✅ **Same repository**: New workflow has `repository=5dlabs-cto` label
3. ✅ **Stage match**: `current-stage` label matches sensor expectation
4. ✅ **PVC exists**: Controller finds or creates PVC with same name
5. ✅ **ConfigMap exists**: `play-progress-*` ConfigMap available
6. ✅ **Event matches**: GitHub event (PR created/approved/merged) matches stage
7. ✅ **Sensor healthy**: Stage-aware sensor is running and processing events

### What Prevents Resume

❌ If any of these are true:

- Task ID changes → New workflow doesn't match old one
- Repository changes → Label mismatch prevents correlation
- Stage label missing → Sensor can't verify correct stage
- PVC deleted → Controller must recreate (data loss)
- ConfigMap deleted → No record of previous progress
- Wrong event → Sensor skips resume (prevents stage regression)
- Sensor error → Events not processed

**Example Sensor Guard** (`stage-aware-cleo-approval.yaml` lines 236-247):

```bash
if [ "$CURRENT_STAGE" = "waiting-quality-complete" ] && [ "$SUSPEND_NODE" = "Running" ]; then
  echo "Found workflow at correct stage and suspended, resuming..."
  if ! resume_workflow; then
    echo "⚠️ Resume attempt failed"
    exit 0
  fi
else
  echo "Workflow not ready for resume (stage: $CURRENT_STAGE, suspend: $SUSPEND_NODE)"
  echo "Skipping resume to prevent incorrect stage progression"
fi
```

---

## 10. STATE PERSISTENCE SUMMARY TABLE

| State Level | Storage | Deletion Behavior | Resume Function |
|---|---|---|---|
| **Workflow Stage** | Kubernetes Labels | NOT deleted with workflow | Sensor matches by label, reads stage, resumes if correct |
| **Agent Workspace** | PVC (Kubernetes) | PERSISTENT, reused | New agent mounts existing PVC, continues session |
| **Agent Session** | Files on PVC | PERSISTENT | `continueSession: true` loads CLAUDE.md, resumes execution |
| **Task Progress** | ConfigMap | PERSISTENT unless manual delete | Tracks current task, stage, workflow name for recovery |
| **CLI State** | SessionState (memory/disk) | Cleanup on timeout | Records CLI type, transitions, execution history |
| **PR Metadata** | Workflow annotations | Injected by sensor on resume | Sensor updates workflow parameters with PR details |

---

## 11. UNIVERSAL STATE ACROSS ALL 6 CLIs

### What's Shared (CLI-Agnostic)

```
✓ PVC mount path: /workspace
✓ Memory file: CLAUDE.md (universal, not CLI-specific)
✓ Workflow labels: task-id, repository, current-stage
✓ ConfigMap format: play-progress-{repo}
✓ CodeRun CRD structure: Identical across all CLIs
✓ Event model: GitHub webhooks independent of CLI
✓ Session state: Recorded in universal format
```

### What's CLI-Specific

```
✗ CLI Config (stored in CodeRun):
  - cli-type: claude|codex|factory|etc
  - cli-model: model for that CLI
  - temperature, max-tokens per CLI
  
✗ Agent credentials:
  - GitHub App per agent/CLI combination
  - Auth tokens specific to CLI provider

✗ Tool implementations:
  - Each CLI adapter implements tools differently
  - Parameter mapping in ToolImplementation struct

✗ Container image:
  - Different container per CLI
  - But all use same mount structure
```

### Resume Across CLI Change

When transitioning from Claude to Codex (for same task):

```
1. Old Workflow (Claude):
   - workspace-cto mounted
   - CLAUDE.md created/updated
   - Stage: waiting-pr-created
   - Label: github-app: "5DLabs-Rex"

2. Delete old workflow

3. New Workflow (Codex for Blaze):
   - New CodeRun CRD created
   - Agent: "blaze" → Same PVC: workspace-cto (SHARED)
   - Same CLAUDE.md available
   - New workflow gets same labels (from ConfigMap)
   - continueSession: true → CLAUDE.md loaded
   
4. Resume Logic:
   - Same PVC → previous work available
   - Same CLAUDE.md → context preserved
   - If stage is waiting-pr-created, PR event resumes it
   - CLI doesn't matter; state is universal
```

---

## 12. RECOMMENDED IMPROVEMENTS FOR ENHANCEMENT

### Current Gaps

1. **No explicit "last-agent" tracking**: Could add to ConfigMap
   ```yaml
   last-agent: "rex"
   last-agent-end-state: {...}
   ```

2. **No agent-handoff annotations**: Could add to Workflow
   ```yaml
   agents.platform/handoff-from-agent: "rex"
   agents.platform/handoff-at-time: "2025-11-12T10:00:00Z"
   ```

3. **No stage completion summary**: Could persist per-stage results
   ```yaml
   stage-results:
     implementation:
       completion-time: "2025-11-12T10:15:00Z"
       pr-created: true
       pr-number: 314
     code-quality:
       completion-time: "..."
       approval-given: true
   ```

4. **CLI transition tracking**: Could be more explicit
   ```yaml
   cli-history:
     - cli: "claude"
       agent: "rex"
       duration: "15m"
     - cli: "factory"
       agent: "blaze"
       duration: "20m"
   ```

### Implementation Options

**Option A: Extend ConfigMap** (Recommended for immediate use)
- Add `stages-completed.yaml` key
- Add `cli-transitions.yaml` key
- Add `agent-handoff-details.yaml` key

**Option B: Use Workflow Annotations** (More visible)
- Store stage results in workflow.metadata.annotations
- More visible in kubectl output
- Could exceed annotation size limits for long-running workflows

**Option C: Create StateSnapshot CRD** (Most structured)
- New CRD: `WorkflowSnapshot` or `AgentHandoff`
- Captured at each stage transition
- Full audit trail of workflow execution
- More storage cost

---

## CONCLUSION

The system implements a **sophisticated, label-based workflow state machine** that enables:

✅ **Transparent Cross-CLI Resume**: Any CLI can resume where another left off  
✅ **Stage-Safe Resumption**: Sensors verify stage before resuming  
✅ **Persistent Agent Context**: PVCs ensure work isn't lost  
✅ **Atomic Stage Transitions**: Labels updated atomically  
✅ **Fault-Tolerant Architecture**: Deleted workflows are recoverable  
✅ **Multi-Agent Coordination**: Clear handoff between implementation, quality, and testing agents  

**Key Insight**: Resume functionality doesn't depend on which CLI runs the workflow—it depends on:
1. **Workflow labels** (task-id, repository, current-stage)
2. **PVC persistence** (agent workspace)
3. **ConfigMap state** (progress tracking)
4. **GitHub event timing** (sensors detect and resume)

This is a **universal resume system**, not a CLI-specific one.

