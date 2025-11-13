# Workflow Resume Implementation Plan
## ConfigMap-Based Stage Tracking Across All CLIs

**Date:** 2025-11-13
**Goal:** Enable workflows to resume from their last completed stage (e.g., Cipher) instead of restarting from Rex/Blaze when workflows are deleted and recreated.

---

## Problem Statement

### Current Behavior
When a Play workflow is deleted (accidentally or intentionally), and a new workflow is created for the same task:
- ❌ Always starts from **beginning** (Rex/Blaze implementation)
- ❌ Re-runs completed stages (wasting time and tokens)
- ❌ Ignores existing progress stored in ConfigMap

### What EXISTS but Isn't Used
- ✅ ConfigMap `play-progress-{repo}` stores:
  - `current-task-id`: Which task (e.g., "5")
  - `stage`: Which agent/stage (e.g., "cipher-security")
  - `status`: in-progress, suspended, failed, completed
  - `workflow-name`: Argo workflow name
  - Timestamps: started-at, last-updated

- ✅ Controller updates ConfigMap as workflow progresses
- ✅ MCP server can query progress
- ✅ CLI-agnostic (works for Claude, Codex, Cursor, Factory, OpenCode, Blaze/Rex)

### The Gap
**Workflow template doesn't READ the ConfigMap to determine resume point!**

---

## Solution Architecture

### High-Level Flow

```
Workflow Start
    ↓
Read play-progress ConfigMap
    ↓
Stage found? → YES → Skip to that stage
             → NO  → Start from beginning (Rex/Blaze)
    ↓
Execute from resume point
    ↓
Update ConfigMap as stages complete
```

### Stage Mapping

| Stage Value in ConfigMap | Workflow Step to Resume At | Agents Skipped |
|--------------------------|----------------------------|----------------|
| `"implementation"` | implementation-cycle | None (start from beginning) |
| `"code-quality"` | update-to-quality-in-progress | Rex, Blaze |
| `"security"` | security-cycle (if configured) | Rex, Blaze, Cleo |
| `"qa"` | update-to-qa-in-progress | Rex, Blaze, Cleo, Cipher |
| `"waiting-merge"` | wait-merge-to-main | Rex, Blaze, Cleo, Cipher, Tess |
| `"completed"` | Don't create new workflow | All (task done) |

---

## Implementation Plan

### Phase 1: Add ConfigMap Read Template (30 min)

**File:** `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`

**Add new template:** `determine-resume-point`

```yaml
- name: determine-resume-point
  inputs:
    parameters:
      - name: task-id
      - name: repository
  script:
    image: alpine/k8s:1.31.0
    command: [bash]
    source: |
      #!/bin/bash
      set -e

      # Install jq if not available
      if ! command -v jq >/dev/null 2>&1; then
        apk add --no-cache jq
      fi

      REPO="{{inputs.parameters.repository}}"
      TASK_ID="{{inputs.parameters.task-id}}"
      CONFIGMAP_NAME="play-progress-$(echo $REPO | tr '/' '-')"

      echo "=== Checking for Existing Progress ==="
      echo "Repository: $REPO"
      echo "Task ID: $TASK_ID"
      echo "ConfigMap: $CONFIGMAP_NAME"

      # Try to get ConfigMap
      if ! PROGRESS=$(kubectl get configmap "$CONFIGMAP_NAME" -n agent-platform -o json 2>/dev/null); then
        echo "No existing progress found - starting from beginning"
        echo "implementation" > /tmp/resume-stage.txt
        exit 0
      fi

      # Extract progress data
      STORED_TASK_ID=$(echo "$PROGRESS" | jq -r '.data["current-task-id"] // ""')
      STORED_STAGE=$(echo "$PROGRESS" | jq -r '.data["stage"] // "implementation"')
      STORED_STATUS=$(echo "$PROGRESS" | jq -r '.data["status"] // "in-progress"')
      WORKFLOW_NAME=$(echo "$PROGRESS" | jq -r '.data["workflow-name"] // ""')

      echo "Found existing progress:"
      echo "  Task ID: $STORED_TASK_ID"
      echo "  Stage: $STORED_STAGE"
      echo "  Status: $STORED_STATUS"
      echo "  Workflow: $WORKFLOW_NAME"

      # Validate task ID matches
      if [ "$STORED_TASK_ID" != "$TASK_ID" ]; then
        echo "⚠️  ConfigMap has different task ($STORED_TASK_ID vs $TASK_ID)"
        echo "Starting from beginning for new task"
        echo "implementation" > /tmp/resume-stage.txt
        exit 0
      fi

      # Check if workflow still exists
      if [ -n "$WORKFLOW_NAME" ]; then
        if kubectl get workflow "$WORKFLOW_NAME" -n agent-platform >/dev/null 2>&1; then
          echo "⚠️  Workflow $WORKFLOW_NAME still exists!"
          echo "This indicates concurrent execution attempt"
          exit 1  # Prevent duplicate workflows
        fi
      fi

      # Determine resume stage based on stored stage
      case "$STORED_STAGE" in
        "implementation")
          RESUME_STAGE="implementation"
          echo "✅ Resuming at: Implementation (Rex/Blaze)"
          ;;
        "code-quality")
          RESUME_STAGE="code-quality"
          echo "✅ Resuming at: Code Quality (Cleo)"
          ;;
        "security")
          RESUME_STAGE="security"
          echo "✅ Resuming at: Security (Cipher)"
          ;;
        "qa")
          RESUME_STAGE="qa"
          echo "✅ Resuming at: QA (Tess)"
          ;;
        "waiting-merge")
          RESUME_STAGE="waiting-merge"
          echo "✅ Resuming at: Waiting for Merge"
          ;;
        *)
          echo "⚠️  Unknown stage: $STORED_STAGE, starting from beginning"
          RESUME_STAGE="implementation"
          ;;
      esac

      echo "$RESUME_STAGE" > /tmp/resume-stage.txt
  outputs:
    parameters:
      - name: resume-stage
        valueFrom:
          path: /tmp/resume-stage.txt
```

### Phase 2: Modify Main DAG to Use Resume Point (45 min)

**Current main DAG:**
```yaml
- name: main
  dag:
    tasks:
      - name: implementation-cycle
        template: implementation-cycle
      - name: update-to-quality-in-progress
        dependencies: [implementation-cycle]
      - name: quality-cycle
        dependencies: [update-to-quality-in-progress]
      # ... etc
```

**New main DAG with resume:**
```yaml
- name: main
  dag:
    tasks:
      # STEP 1: Determine resume point
      - name: check-resume-point
        template: determine-resume-point
        arguments:
          parameters:
            - name: task-id
              value: "{{workflow.parameters.task-id}}"
            - name: repository
              value: "{{workflow.parameters.repository}}"

      # STEP 2: Conditional execution based on resume point

      # Implementation stage (always check, might skip)
      - name: should-run-implementation
        dependencies: [check-resume-point]
        template: check-stage-needed
        arguments:
          parameters:
            - name: resume-stage
              value: "{{tasks.check-resume-point.outputs.parameters.resume-stage}}"
            - name: current-stage
              value: "implementation"

      - name: implementation-cycle
        dependencies: [should-run-implementation]
        template: implementation-cycle
        when: "{{tasks.should-run-implementation.outputs.parameters.should-run}} == true"

      # Code Quality stage
      - name: should-run-quality
        dependencies: [implementation-cycle]
        template: check-stage-needed
        arguments:
          parameters:
            - name: resume-stage
              value: "{{tasks.check-resume-point.outputs.parameters.resume-stage}}"
            - name: current-stage
              value: "code-quality"

      - name: update-to-quality-in-progress
        dependencies: [should-run-quality]
        when: "{{tasks.should-run-quality.outputs.parameters.should-run}} == true"
        template: update-workflow-stage

      # ... similar for security, qa, waiting-merge
```

### Phase 3: Add Stage Skip Logic Template (20 min)

```yaml
- name: check-stage-needed
  inputs:
    parameters:
      - name: resume-stage
        description: "Stage to resume from (from ConfigMap)"
      - name: current-stage
        description: "Current stage being evaluated"
  script:
    image: alpine/k8s:1.31.0
    command: [bash]
    source: |
      #!/bin/bash

      RESUME_STAGE="{{inputs.parameters.resume-stage}}"
      CURRENT_STAGE="{{inputs.parameters.current-stage}}"

      # Stage order
      STAGES=("implementation" "code-quality" "security" "qa" "waiting-merge")

      # Find indices
      RESUME_IDX=-1
      CURRENT_IDX=-1

      for i in "${!STAGES[@]}"; do
        if [ "${STAGES[$i]}" = "$RESUME_STAGE" ]; then
          RESUME_IDX=$i
        fi
        if [ "${STAGES[$i]}" = "$CURRENT_STAGE" ]; then
          CURRENT_IDX=$i
        fi
      done

      # If current stage is before resume stage, skip it
      if [ $CURRENT_IDX -lt $RESUME_IDX ]; then
        echo "Skipping $CURRENT_STAGE (already completed, resuming at $RESUME_STAGE)"
        echo "false" > /tmp/should-run.txt
      else
        echo "Running $CURRENT_STAGE"
        echo "true" > /tmp/should-run.txt
      fi
  outputs:
    parameters:
      - name: should-run
        valueFrom:
          path: /tmp/should-run.txt
```

### Phase 4: Update ConfigMap After Each Stage (15 min)

Modify existing `update-workflow-stage` template to also update ConfigMap:

```yaml
# Add to update-workflow-stage template
# After updating workflow labels, also update ConfigMap

kubectl patch configmap "play-progress-$(echo $REPO | tr '/' '-')" \
  -n agent-platform \
  --type merge \
  -p "{\"data\":{\"stage\":\"{{inputs.parameters.new-stage}}\",\"last-updated\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}"
```

### Phase 5: Handle Edge Cases (30 min)

**Edge Case 1: Workflow exists for task**
- Check if active workflow exists before creating new one
- Prevent duplicate workflows

**Edge Case 2: ConfigMap out of sync**
- Validate ConfigMap workflow-name matches reality
- Detect orphaned states (workflow deleted but ConfigMap says in-progress)
- Clean up stale ConfigMaps

**Edge Case 3: Stage transitions**
- What if Cleo failed but ConfigMap says "code-quality"?
- Should we retry the failed stage or move to next?
- Decision: Retry the stage it failed on

**Edge Case 4: Manual workflow creation**
- User explicitly provides `start-from-stage` parameter
- Override ConfigMap resume point

---

## Testing Plan

### Test Case 1: Fresh Start (No ConfigMap)
```bash
# Precondition: No ConfigMap exists
# Action: Start play workflow for task 5
# Expected: Creates ConfigMap, starts at implementation
# Verify: ConfigMap created with stage="implementation"
```

### Test Case 2: Resume from Cipher
```bash
# Precondition: ConfigMap exists with stage="security", task-id="5"
# Action: Workflow deleted, restart play workflow for task 5
# Expected: Skips Rex/Blaze/Cleo, starts at Cipher
# Verify: Workflow logs show "Skipping implementation (already completed)"
```

### Test Case 3: Different Task
```bash
# Precondition: ConfigMap exists with task-id="5", stage="security"
# Action: Start play workflow for task 6
# Expected: Ignores ConfigMap (different task), starts at implementation
# Verify: New ConfigMap entry or task-id updated to 6
```

### Test Case 4: Completed Task
```bash
# Precondition: ConfigMap exists with status="completed", task-id="5"
# Action: Attempt to start play workflow for task 5
# Expected: Workflow exits early (task already done) OR clears and restarts
# Verify: No duplicate work performed
```

### Test Case 5: Orphaned State
```bash
# Precondition: ConfigMap says workflow="play-task-5-abc" but workflow doesn't exist
# Action: Start play workflow for task 5
# Expected: Detects orphan, resumes from ConfigMap stage
# Verify: Workflow name updated in ConfigMap
```

### Test Case 6: Cross-CLI Resume
```bash
# Precondition: Rex (Claude) completed implementation
# Action: Delete workflow, restart with Factory CLI
# Expected: Factory starts at Cleo (skips implementation)
# Verify: Stage progression works regardless of CLI
```

---

## Implementation Steps

### Step 1: Add Determine Resume Point Template ✅
**File:** `play-workflow-template.yaml`
**Lines:** Add new template ~line 400
**Test:** Template renders correctly, kubectl apply succeeds

### Step 2: Add Check Stage Needed Template ✅
**File:** `play-workflow-template.yaml`
**Lines:** Add new template ~line 500
**Test:** Stage comparison logic works correctly

### Step 3: Modify Main DAG ✅
**File:** `play-workflow-template.yaml`
**Lines:** Modify main DAG (lines 222-270)
**Changes:**
- Add `check-resume-point` as first task
- Add `should-run-{stage}` checks before each major stage
- Add `when:` conditions to skip stages
**Test:** DAG renders correctly, dependencies are valid

### Step 4: Update ConfigMap on Stage Transitions ✅
**File:** `play-workflow-template.yaml`
**Lines:** Modify `update-workflow-stage` template (~line 1200)
**Changes:** Add kubectl patch to update ConfigMap stage
**Test:** ConfigMap updates after each stage transition

### Step 5: Add Concurrent Workflow Prevention ✅
**File:** `play-workflow-template.yaml`
**Lines:** In `determine-resume-point` template
**Logic:** Check if workflow-name from ConfigMap still exists
**Test:** Second workflow creation fails if first still running

### Step 6: Update Controller to Write ConfigMap ✅
**File:** `controller/src/tasks/play/progress.rs`
**Verify:** Code already exists (PR #1322)
**Check:** Controller calls `write_progress()` at appropriate times

### Step 7: Add MCP Tool for Manual Override ✅
**File:** `mcp/src/tools.rs`
**New parameter:** `start_from_stage` (optional)
**Logic:** If provided, override ConfigMap resume point
**Test:** Can manually force restart from specific stage

### Step 8: Update Documentation ✅
**Files:**
- `docs/.taskmaster/docs/architecture.md` - Document resume functionality
- `README.md` - Add resume examples
- Agent system prompts - Note that stages may be skipped

---

## Code Locations

### Files to Modify

1. **Workflow Template** (PRIMARY)
   - `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
   - Add: determine-resume-point template
   - Add: check-stage-needed template
   - Modify: main DAG with conditional execution
   - Modify: update-workflow-stage to update ConfigMap

2. **Controller Progress Module** (VERIFY ONLY)
   - `controller/src/tasks/play/progress.rs`
   - Already exists, verify it's called correctly

3. **MCP Play Tool** (ENHANCEMENT)
   - `mcp/src/tools.rs`
   - Add optional `start_from_stage` parameter

4. **Project Workflow Template** (SECONDARY)
   - `infra/charts/controller/templates/workflowtemplates/play-project-workflow-template.yaml`
   - Apply same changes for multi-task projects

### Key Functions to Use

**From controller:**
- `read_progress(client, repo)` → Option<PlayProgress>
- `write_progress(client, progress)` → Result<()>
- `clear_progress(client, repo)` → Result<()>

**From workflow:**
- `kubectl get configmap play-progress-{repo}`
- `kubectl patch configmap play-progress-{repo}`
- Stage comparison logic (bash arrays)

---

## Detailed Implementation: Main DAG Redesign

### Current DAG Structure
```
main
├── implementation-cycle
├── update-to-quality-in-progress (depends: implementation-cycle)
├── quality-cycle (depends: update-to-quality-in-progress)
├── update-to-security-in-progress (depends: quality-cycle)
├── security-cycle (depends: update-to-security-in-progress, when: cipher configured)
├── update-to-qa-in-progress (depends: security-cycle OR quality-cycle)
├── qa-cycle (depends: update-to-qa-in-progress)
├── update-to-waiting-merge (depends: qa-cycle)
└── wait-merge-to-main (depends: update-to-waiting-merge)
```

### New DAG Structure with Resume
```
main
├── determine-resume-point (ALWAYS FIRST)
│   ↓ outputs: resume-stage
│
├── implementation-cycle (when: stage >= implementation)
│   ├── should-run-implementation (depends: determine-resume-point)
│   └── run-if-needed (when: should-run == true)
│
├── quality-cycle (when: stage >= code-quality)
│   ├── should-run-quality (depends: implementation-cycle or determine-resume-point)
│   ├── update-to-quality (when: should-run-quality == true)
│   └── quality-cycle (depends: update-to-quality)
│
├── security-cycle (when: stage >= security AND cipher configured)
│   ├── should-run-security
│   ├── update-to-security (when: should-run-security == true)
│   └── security-cycle (depends: update-to-security)
│
├── qa-cycle (when: stage >= qa)
│   ├── should-run-qa
│   ├── update-to-qa (when: should-run-qa == true)
│   └── qa-cycle (depends: update-to-qa)
│
└── wait-merge (when: stage >= waiting-merge)
    ├── should-run-merge-wait
    ├── update-to-waiting-merge (when: should-run-merge-wait == true)
    └── wait-merge-to-main (depends: update-to-waiting-merge)
```

### Conditional Execution Logic

Each stage now has THREE tasks:
1. **Check if needed** - Compare resume-stage vs current-stage
2. **Update stage label** - Only if stage should run
3. **Execute stage** - Only if stage should run

**Benefits:**
- ✅ Stages that are already completed are skipped entirely
- ✅ ConfigMap provides single source of truth
- ✅ Works across all CLIs (no Claude-specific logic)
- ✅ Workflow can be deleted and recreated safely

---

## ConfigMap Schema

### Example ConfigMap
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: play-progress-5dlabs-cto
  namespace: agent-platform
  labels:
    play-tracking: "true"
    repository: "5dlabs-cto"
data:
  repository: "5dlabs/cto"
  branch: "main"
  current-task-id: "5"
  workflow-name: "play-task-5-abc123"
  status: "suspended"
  stage: "cipher-security"        # ← RESUME POINT!
  started-at: "2025-11-13T01:00:00Z"
  last-updated: "2025-11-13T03:15:00Z"
```

### Data Fields

| Field | Type | Purpose | Example |
|-------|------|---------|---------|
| `repository` | string | GitHub repo | "5dlabs/cto" |
| `branch` | string | Target branch | "main" |
| `current-task-id` | string | Task number | "5" |
| `workflow-name` | string | Argo workflow | "play-task-5-abc123" |
| `status` | enum | Workflow status | "in-progress", "suspended", "failed", "completed" |
| **`stage`** | **string** | **Current agent** | **"cipher-security"** ← KEY! |
| `started-at` | RFC3339 | Start time | "2025-11-13T01:00:00Z" |
| `last-updated` | RFC3339 | Last update | "2025-11-13T03:15:00Z" |

---

## Stage Values Reference

### Valid Stage Values
- `"implementation"` - Rex (Claude), Blaze (Cursor/Factory), or other implementation agents
- `"code-quality"` - Cleo ensures linting, formatting, unit tests pass
- `"security"` - Cipher runs security scans (optional, configured per task)
- `"qa"` - Tess performs end-to-end testing in live Kubernetes
- `"waiting-merge"` - Waiting for Atlas to merge PR to main
- `"completed"` - Task fully complete

### Stage Transitions
```
implementation → code-quality → security → qa → waiting-merge → completed
     (Rex)          (Cleo)      (Cipher)   (Tess)   (Atlas)
```

---

## Error Handling

### Scenario: ConfigMap exists but workflow-name is stale
**Detection:** Workflow name in ConfigMap doesn't exist in cluster
**Action:** Log warning, resume from ConfigMap stage, update workflow-name
**Rationale:** Previous workflow was deleted, resume from last known good stage

### Scenario: Two workflows try to start simultaneously
**Detection:** ConfigMap workflow-name exists AND workflow still running
**Action:** Fail second workflow creation with clear error
**Rationale:** Prevent concurrent execution of same task

### Scenario: Invalid stage in ConfigMap
**Detection:** Stage value doesn't match known stages
**Action:** Log warning, default to "implementation", update ConfigMap
**Rationale:** Corrupted ConfigMap, start from safe point

### Scenario: Task ID mismatch
**Detection:** ConfigMap task-id=5 but new workflow wants task-id=6
**Action:** Ignore ConfigMap, start from beginning, update ConfigMap to task 6
**Rationale:** Different task, different workflow

---

## Rollback Plan

If resume functionality causes issues:

### Quick Disable
Add workflow parameter to disable resume:
```yaml
- name: disable-resume
  value: "false"  # Set to "true" to always start from beginning
```

### Full Rollback
1. Revert to previous workflow template version
2. ConfigMaps remain but are ignored
3. System reverts to "always start from beginning" behavior

---

## Performance Impact

### Before Resume
- Task at Cipher stage
- Workflow deleted
- New workflow: Rex (30min) → Blaze (20min) → Cleo (15min) → Cipher
- **Total: 65+ minutes to get back to Cipher**

### After Resume
- Task at Cipher stage
- Workflow deleted
- New workflow: Check ConfigMap (5s) → Skip to Cipher
- **Total: 5 seconds to resume at Cipher**

**Improvement: ~99% faster resume!**

---

## Security Considerations

### ConfigMap Access
- ConfigMaps readable by workflow ServiceAccount
- No sensitive data stored (only workflow state)
- Task IDs and stages are not secrets

### Concurrent Execution Prevention
- Prevent two workflows for same task running simultaneously
- Check workflow-name existence before starting
- Atomic ConfigMap updates to prevent race conditions

### State Validation
- Always validate ConfigMap data before using
- Handle missing/corrupted fields gracefully
- Never trust ConfigMap blindly (verify against cluster state)

---

## Future Enhancements

### Phase 2 Features (After Initial Implementation)

1. **Per-Agent Progress Tracking**
   - Store which agents completed within a stage
   - If Cleo failed 3 times, maybe skip to Tess with warning

2. **Multi-Repository Support**
   - Track progress for multiple repos simultaneously
   - Separate ConfigMaps: `play-progress-{owner}-{repo}`

3. **Progress API Endpoint**
   - HTTP endpoint to query progress
   - Dashboard showing all tasks and their stages

4. **Stage Rollback**
   - MCP tool to manually set stage back (e.g., "rerun Cleo")
   - Useful for debugging or retrying failed stages

5. **Progress History**
   - Store historical progress in annotations
   - Track how long each stage took

---

## Success Criteria

### Must Have (MVP)
- ✅ ConfigMap read on workflow start
- ✅ Skip to correct resume stage based on ConfigMap
- ✅ Update ConfigMap as stages complete
- ✅ Works across all 6 CLIs
- ✅ Prevents duplicate workflow execution
- ✅ Clean error messages for edge cases

### Should Have (Nice to Have)
- ✅ Manual override via MCP tool
- ✅ Orphaned state detection and cleanup
- ✅ Progress status queryable via MCP
- ✅ Clear logging of skip decisions

### Could Have (Future)
- Per-agent completion tracking
- Progress history/analytics
- Stage rollback capability
- Multi-repo support

---

## Implementation Checklist

### Pre-Implementation
- [x] Review PR #1322 to understand existing code
- [x] Verify controller progress.rs module exists
- [x] Check ConfigMap structure and fields
- [ ] Review current workflow template structure
- [ ] Identify all stage transition points

### Phase 1: Templates
- [ ] Add `determine-resume-point` template
- [ ] Add `check-stage-needed` template
- [ ] Test templates render correctly
- [ ] Test kubectl apply succeeds

### Phase 2: DAG Modification
- [ ] Add resume point check as first task
- [ ] Add conditional execution for implementation stage
- [ ] Add conditional execution for code-quality stage
- [ ] Add conditional execution for security stage
- [ ] Add conditional execution for qa stage
- [ ] Add conditional execution for waiting-merge stage
- [ ] Test DAG with `when:` conditions
- [ ] Verify dependency graph is valid

### Phase 3: ConfigMap Updates
- [ ] Modify `update-workflow-stage` to patch ConfigMap
- [ ] Test ConfigMap updates on stage transitions
- [ ] Verify stage value is correct in ConfigMap

### Phase 4: Edge Cases
- [ ] Add concurrent workflow detection
- [ ] Add orphaned state detection
- [ ] Add invalid stage handling
- [ ] Add task ID mismatch handling
- [ ] Test all edge case scenarios

### Phase 5: Testing
- [ ] Test fresh start (no ConfigMap)
- [ ] Test resume from each stage
- [ ] Test cross-CLI resume
- [ ] Test concurrent workflow prevention
- [ ] Test orphaned state recovery
- [ ] Test task ID mismatch
- [ ] Test completed task handling

### Phase 6: Documentation
- [ ] Update architecture.md with resume flow
- [ ] Add resume examples to README
- [ ] Document ConfigMap schema
- [ ] Add troubleshooting guide

### Phase 7: PR & Deployment
- [ ] Create feature branch
- [ ] Commit all changes
- [ ] Create PR with comprehensive description
- [ ] Wait for CI to pass
- [ ] Merge PR
- [ ] Verify controller redeploys
- [ ] Test in production

---

## Example Usage

### Starting a Play Workflow (with Auto-Resume)
```bash
# Via MCP tool (auto-detects resume point)
/play

# Explicitly specify task (still auto-resumes from ConfigMap)
/play task_id=5

# Force restart from beginning (override ConfigMap)
/play task_id=5 start_from_stage=implementation

# Force start from specific stage
/play task_id=5 start_from_stage=qa
```

### Checking Progress
```bash
# Via MCP tool
/play_status

# Manually
kubectl get configmap play-progress-5dlabs-cto -n agent-platform -o yaml
```

### Clearing Progress (Force Fresh Start)
```bash
# Via controller
# (MCP tool to be added)

# Manually
kubectl delete configmap play-progress-5dlabs-cto -n agent-platform
```

---

## Risk Assessment

### Low Risk
- ✅ ConfigMap read/write operations
- ✅ Adding new templates (doesn't affect existing)
- ✅ Conditional execution with `when:` (well-tested Argo feature)

### Medium Risk
- ⚠️ DAG dependency modifications (could break workflow)
- ⚠️ Concurrent workflow detection (race conditions possible)
- ⚠️ Stage value validation (need comprehensive testing)

### Mitigation
- Extensive testing before production deployment
- Feature flag to disable resume if issues occur
- Rollback plan ready
- Monitor first few workflow executions closely

---

## Timeline Estimate

### Development: ~3 hours
- Phase 1 (Templates): 30 min
- Phase 2 (DAG): 45 min
- Phase 3 (ConfigMap updates): 15 min
- Phase 4 (Edge cases): 30 min
- Phase 5 (Testing): 45 min
- Phase 6 (Documentation): 15 min

### Deployment: ~1 hour
- PR creation and review: 30 min
- CI pipeline: 15 min
- Controller redeploy: 10 min
- Production validation: 5 min

**Total: ~4 hours end-to-end**

---

## Idempotent Operations on Resume

### Problem: Duplicate Resource Creation

When resuming, agents must NOT recreate resources that already exist:

**GitHub Resources:**
- ❌ Don't create duplicate GitHub Project
- ❌ Don't create duplicate Issues for sub-tasks
- ❌ Don't recreate PR if it already exists
- ❌ Don't add duplicate labels to PR

**Kubernetes Resources:**
- ❌ Don't recreate namespaces that already exist
- ❌ Don't recreate ArgoCD applications
- ❌ Don't recreate ConfigMaps/Secrets

### Solution: Check Before Create Pattern

Each agent must implement **idempotent operations**:

#### Implementation Stage (Rex/Blaze)
```bash
# Before creating PR
if gh pr list --head "$FEATURE_BRANCH" --state open | grep -q .; then
  echo "✅ PR already exists, using existing PR"
  PR_NUMBER=$(gh pr list --head "$FEATURE_BRANCH" --json number -q '.[0].number')
else
  echo "Creating new PR..."
  gh pr create ...
fi
```

#### Code Quality Stage (Cleo)
```bash
# Before adding review
if gh pr reviews $PR_NUMBER | grep -q "5dlabs-cleo"; then
  echo "✅ Cleo review already exists, updating if needed"
else
  echo "Adding Cleo review..."
  gh pr review ...
fi
```

#### QA Stage (Tess)
```bash
# Before creating test namespace
if kubectl get namespace "test-task-$TASK_ID" >/dev/null 2>&1; then
  echo "✅ Test namespace already exists, reusing"
else
  echo "Creating test namespace..."
  kubectl create namespace ...
fi
```

#### Morgan (PM Agent) - GitHub Project Setup
```bash
# Before creating GitHub Project
PROJECT_ID=$(gh project list --owner 5dlabs --format json | jq -r '.[] | select(.title=="Task Master") | .number')
if [ -n "$PROJECT_ID" ]; then
  echo "✅ Project already exists: #$PROJECT_ID"
else
  echo "Creating GitHub Project..."
  gh project create --owner 5dlabs --title "Task Master"
fi

# Before creating issues
for task in tasks/*.md; do
  TASK_ID=$(basename "$task" .md | sed 's/task-//')
  if gh issue list --label "task-$TASK_ID" --state all | grep -q .; then
    echo "✅ Issue for task $TASK_ID already exists"
  else
    echo "Creating issue for task $TASK_ID..."
    gh issue create ...
  fi
done

# Before linking to project
ITEM_ID=$(gh project item-list $PROJECT_ID --owner 5dlabs --format json | jq -r ".[] | select(.content.number==$ISSUE_NUMBER) | .id")
if [ -n "$ITEM_ID" ]; then
  echo "✅ Issue already linked to project"
else
  echo "Linking issue to project..."
  gh project item-add $PROJECT_ID --owner 5dlabs --url "$ISSUE_URL"
fi
```

### Template Updates Required

**All agent templates need idempotency checks:**

1. **`container.sh.hbs`** (all CLIs)
   - Check for existing PR before `gh pr create`
   - Check for existing labels before `gh pr edit --add-label`
   - Check for existing project link before `gh project item-add`

2. **`morgan-pm.sh.hbs`**
   - Check for existing GitHub Project
   - Check for existing issues
   - Check for existing project items (issue links)
   - Update field values instead of creating duplicates

3. **`container-cleo.sh.hbs`**
   - Check for existing PR reviews from Cleo
   - Update review if changes needed vs create new

4. **`container-tess.sh.hbs`**
   - Check for existing test namespaces
   - Check for existing test deployments
   - Reuse if healthy, recreate if failed

### Implementation Pattern

**Standard pattern for all agents:**

```bash
# 1. Check if resource exists
if resource_exists "$RESOURCE_ID"; then
  echo "✅ Resource already exists: $RESOURCE_ID"

  # 2. Validate it's in correct state
  if resource_is_valid "$RESOURCE_ID"; then
    echo "   Using existing resource"
    RESOURCE=$RESOURCE_ID
  else
    echo "   ⚠️  Resource exists but invalid, recreating..."
    delete_resource "$RESOURCE_ID"
    RESOURCE=$(create_resource)
  fi
else
  # 3. Create new resource
  echo "Creating new resource..."
  RESOURCE=$(create_resource)
fi

# 4. Proceed with resource
use_resource "$RESOURCE"
```

### Testing Idempotency

**Test each agent's resume behavior:**

```bash
# Test 1: Rex already created PR
# - Delete workflow
# - ConfigMap says stage="implementation", status="suspended"
# - Restart workflow
# - Expected: Rex finds existing PR, doesn't create duplicate

# Test 2: Cleo already reviewed
# - Delete workflow mid-Cleo
# - ConfigMap says stage="code-quality"
# - Restart workflow
# - Expected: Cleo updates review, doesn't create duplicate

# Test 3: Tess test namespace exists
# - Delete workflow mid-Tess
# - ConfigMap says stage="qa"
# - Test namespace still exists
# - Restart workflow
# - Expected: Tess reuses namespace, runs tests
```

---

## Open Questions

1. **What if a stage partially completes?**
   - E.g., Cleo runs but PR isn't created yet
   - Should we resume at Cleo or mark as incomplete?
   - **Decision:** Resume at the stage that was running (let it retry)

2. **How to handle manual stage skips?**
   - User wants to skip Cipher and go straight to Tess
   - **Decision:** Add MCP parameter `skip_stages=["security"]`

3. **Should we track sub-stages?**
   - E.g., "implementation-rex-complete", "implementation-blaze-in-progress"
   - **Decision:** Start simple with major stages only, add sub-stages in Phase 2

4. **ConfigMap cleanup strategy?**
   - When to delete old ConfigMaps?
   - **Decision:** Keep for 7 days after completion, add TTL cleanup

---

## References

- **PR #1322**: Original ConfigMap progress tracking implementation
- **Architecture Doc**: `docs/.taskmaster/docs/architecture.md` - Multi-agent workflow design
- **Controller Code**: `controller/src/tasks/play/progress.rs` - ConfigMap operations
- **Workflow Template**: `play-workflow-template.yaml` - Current stage tracking via labels

---

## Next Steps

1. ✅ Review this plan with team
2. ✅ Get approval for DAG restructuring approach
3. ✅ Create feature branch: `feat/workflow-stage-resume-from-configmap`
4. ✅ Begin Phase 1 implementation
5. ✅ Iterative testing after each phase
6. ✅ Create PR when all phases complete
7. ✅ Deploy and validate in production

---

**Ready to implement!** This plan provides:
- ✅ Clear problem statement
- ✅ Detailed solution architecture
- ✅ Step-by-step implementation guide
- ✅ Comprehensive testing plan
- ✅ Risk mitigation strategies
- ✅ Timeline and success criteria
