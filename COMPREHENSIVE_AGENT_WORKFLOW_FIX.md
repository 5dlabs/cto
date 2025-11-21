# Comprehensive Agent Workflow Issues & Fixes

## Investigation Summary

After deep dive analysis, identified **THREE systemic issues** affecting the Rex→Cleo→Cipher→Tess workflow:

---

## Issue #1: Tess FIFO Deadlock (CRITICAL)

### Status: ✅ FIXED in PR #1551

### Symptoms
- **Tess testing agent hangs indefinitely** after completing all work
- Agent posts final success message but container never exits
- Workflow stuck in Running phase for 6+ hours
- Affects both Task 3 and Task 4 (confirmed)

### Root Cause
**FIFO Deadlock Pattern** in `container.sh.hbs`:

```bash
# BROKEN CODE:
exec 9>"$FIFO_PATH"      # Open FIFO writer
printf '...' >&9          # Send prompt
wait $CLAUDE_PID          # Wait for Claude ← DEADLOCK HERE
exec 9>&- 2>/dev/null     # Close writer (never reached)
```

**Why it deadlocks:**
1. Claude finishes work, sends `end_turn` message
2. Claude waits for EOF on stdin to exit cleanly
3. FIFO writer still open in parent shell (fd 9)
4. Claude never gets EOF → hangs waiting
5. Script waits for Claude → hangs waiting
6. **Circular dependency = infinite deadlock**

### Evidence
- Claude process (PID 584) still running 6+ hours after completion
- Container trap cleanup never fires
- Sentinel file `.agent_done` never created
- Docker daemon sidecar waiting forever

### Fix
Close FIFO writer **BEFORE** waiting for Claude:

```bash
# FIXED CODE:
exec 9>"$FIFO_PATH"
printf '...' >&9
exec 9>&-                # Close immediately!
wait $CLAUDE_PID         # Now Claude gets EOF and can exit
```

**Files Changed:**
- `infra/charts/controller/agent-templates/code/claude/container.sh.hbs`
- `infra/charts/controller/templates/agent-templates-claude.yaml`

**PR:** #1551

---

## Issue #2: Cipher Posting Duplicate Reviews

### Status: ✅ ALREADY FIXED in PR #1546 (deployed at 19:58 UTC)

### Symptoms  
- **20 reviews posted to PR #11 in 40 minutes**
- Reviews alternate between APPROVED and CHANGES_REQUESTED
- Spams PR with duplicate security validations

### Timeline
- Cipher ran: 12:54-13:36 UTC (42 minutes of loops)
- Fix deployed: 19:58 UTC
- **Issue resolved - old runs were before fix deployment**

### Root Cause (Was)
Cipher's Claude prompt allowed it to run `gh pr review` commands directly, causing multiple reviews per execution plus one from the container script.

### Fix (Already Applied)
Added explicit restrictions in Cipher's system prompt to prevent review commands inside agent loop. Container script posts single review after completion.

**No action needed** - this is already fixed.

---

## Issue #3: Missing Security Stage Support in Controller

### Status: ✅ FIXED (included in new PR)

### Symptoms
- Security stage falls into `Unknown` category
- Controller has no retry/completion logic for Security
- Potential for improper retry handling

### Root Cause
`WorkflowStage` enum missing Security variant:

```rust
// OLD:
enum WorkflowStage {
    Implementation,
    Quality,
    Testing,
    Unknown(String),  // ← Security falls here!
}
```

### Impact
- Security stage gets `Unknown("security")` classification
- `determine_retry_reason` has no Security case
- Falls through to Unknown→None (no retry logic)
- **Actually worked correctly by accident** since returning None means no retry
- But inconsistent and fragile - should be explicit

### Fix
Added Security to enum and retry logic:

```rust
// FIXED:
enum WorkflowStage {
    Implementation,
    Quality,
    Security,      // ← Explicit support
    Testing,
    Unknown(String),
}

// In determine_retry_reason:
WorkflowStage::Security => {
    // Security agent posts GitHub review - always final
    // No retry conditions
    None
}
```

**Files Changed:**
- `controller/src/tasks/code/controller.rs`

---

## Complete Fix Package

### PR #1551 Contents

**Branch:** `fix/tess-fifo-deadlock`  
**Changes:**
1. ✅ Tess FIFO deadlock fix (container.sh.hbs)
2. ✅ Security stage controller support (controller.rs)

**Why Combined:**
Both are systemic issues discovered during investigation that affect the Rex→Cleo→Cipher→Tess pipeline. Combining ensures complete workflow stability.

---

## Remediation Steps

### Step 1: Verify No Compilation Errors

```bash
cd controller && cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
cd controller && cargo test
```

### Step 2: Merge PR #1551

```bash
# Wait for CI, then:
gh pr merge 1551 --merge
```

### Step 3: Wait for ArgoCD Sync (~2 min)

```bash
kubectl get app -n argocd controller -o jsonpath='{.status.sync.status}'
# Wait for: "Synced"
```

### Step 4: Kill Stuck Tess Jobs

```bash
# Task 3:
kubectl delete job -n agent-platform play-coderun-task-3-tess-claude-agent-plat-34f1645a-9f0bd680-v1

# Task 4:
kubectl delete job -n agent-platform play-coderun-task-4-tess-claude-agent-plat-9e0e7e92-0ddd3cbe-v1
```

### Step 5: Monitor Workflow Completion

```bash
# Watch both workflows:
kubectl get workflow -n agent-platform play-task-3-zvzm5 play-task-4-8zn72 -w

# Should see:
# - Tess CodeRuns transition to Running
# - Jobs complete within 5-10 minutes  
# - CodeRuns transition to Succeeded with work_completed=true
# - Workflows complete successfully
```

---

## Why These Issues Occurred

### Tess FIFO Deadlock
- **Recent Change**: Refactored to use FIFO pattern for prompt injection
- **Missed Pattern**: Other agents (Rex, Blaze) use direct piping (auto-closes stdin)
- **Oversight**: Didn't close FIFO writer before wait in generic `container.sh.hbs`

### Cipher Duplicate Reviews
- **Fixed in #1546**: System prompt now prevents duplicate review commands
- **Why We Still See It**: Looking at old runs from before fix deployed

### Missing Security Stage
- **Historical Gap**: Controller originally designed for Rex→Cleo→Tess (3 stages)
- **Security Added Later**: Cipher security stage added but controller enum not updated
- **Worked By Luck**: Unknown→None meant no retries, which was correct behavior
- **Should Be Explicit**: Proper stage support prevents future bugs

---

## Testing Plan

### 1. Verify FIFO Fix Works

```bash
# After merge + ArgoCD sync, trigger new Tess run:
kubectl delete job -n agent-platform -l stage=testing,task-id=3

# Monitor logs for proper exit:
kubectl logs -f -n agent-platform -l stage=testing,task-id=3 -c claude-claude-sonnet-4-5-20250929

# Should see:
# - "🧹 Cleanup: Signaling docker-daemon to shut down..."
# - Container exits cleanly
# - No hanging processes
```

### 2. Verify Security Stage Recognition

```bash
# Check controller logs recognize Security stage properly:
kubectl logs -n agent-platform deployment/controller | grep "stage: Security"

# Should see explicit Security stage logging, not Unknown
```

### 3. End-to-End Workflow Test

```bash
# Launch fresh task to verify complete pipeline:
task-master play --task-id=5

# Should complete: Rex → Cleo → Cipher → Tess without hangs
```

---

## Prevention Measures

### For FIFO Patterns
**Rule:** Always close FIFO writer immediately after sending message, BEFORE any wait.

```bash
# Pattern to follow:
exec 9>"$FIFO"    # Open
printf '...' >&9   # Send
exec 9>&-          # Close immediately
wait $PROCESS_PID  # Then wait
```

### For New Workflow Stages
**Rule:** When adding new stages, update controller enum AND retry logic.

**Checklist:**
- [ ] Add variant to `WorkflowStage` enum
- [ ] Add case to `get_workflow_stage()` mapping
- [ ] Add case to `determine_retry_reason()` logic
- [ ] Add tests for new stage behavior

### For Agent Loop Prevention
**Rule:** Agents should post ONE artifact (PR, review, comment) then exit.

**Prevention:**
- System prompts forbid duplicate API calls
- Container scripts post final artifacts AFTER agent completes
- Controller recognizes completion signals properly

---

## Verification Checklist

After merging PR #1551:

- [ ] Tess jobs complete within 10 minutes (not hanging)
- [ ] Sentinel files `.agent_done` created properly
- [ ] Docker daemon sidecars shut down cleanly
- [ ] Workflows progress from testing→completion
- [ ] No duplicate Cipher reviews on new PRs
- [ ] Controller logs show explicit Security stage recognition
- [ ] All 4 stages (Implementation/Quality/Security/Testing) work end-to-end

---

**Investigation Completed:** 2025-11-21 12:10 PST  
**Primary Fix:** PR #1551 (T

ess FIFO + Security stage)  
**Secondary Fix:** PR #1546 (Cipher reviews - already deployed)

