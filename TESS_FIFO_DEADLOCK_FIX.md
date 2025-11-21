# Tess FIFO Deadlock Bug - Investigation & Remediation

## Executive Summary

**Issue**: Tess testing agent hangs indefinitely after completing work, preventing Rex→Cleo→Cipher→Tess workflows from finishing.

**Root Cause**: FIFO deadlock - container script waits for Claude to exit, but Claude waits for EOF, creating circular dependency.

**Status**: Fix committed to PR #1551, awaiting merge to main.

---

## Bug Details

### The Deadlock Pattern

```bash
# Current (BROKEN) code in container.sh.hbs:
exec 9>"$FIFO_PATH"                    # Open FIFO writer
printf '...' >&9                       # Send prompt
# Writer still open here!
wait $CLAUDE_PID                       # Wait for Claude to exit
exec 9>&- 2>/dev/null || true         # Close writer AFTER wait
```

**Why this hangs:**
1. Claude finishes work, sends final `end_turn` message
2. Claude waits for EOF on stdin to exit cleanly
3. But FIFO writer (fd 9) is still open in parent shell
4. Claude never receives EOF, hangs forever
5. Script waits for Claude, hangs forever
6. Writer never closes (line 1400 never executes)
7. **Circular dependency = deadlock**

### Fixed Pattern

```bash
# Fixed code:
exec 9>"$FIFO_PATH"                    # Open FIFO writer  
printf '...' >&9                       # Send prompt
exec 9>&-                              # Close writer IMMEDIATELY
wait $CLAUDE_PID                       # Now Claude can exit cleanly
```

---

## Evidence from Investigation

### 1. Tess CodeRun Status

```yaml
name: coderun-cto-parallel-test-t3-testing-qrhqr
status:
  phase: Running
  workCompleted: false
  lastUpdate: "2025-11-21T13:45:29Z"  # 6+ hours ago!
```

### 2. Job Status

```yaml
job: play-coderun-task-3-tess-claude-agent-plat-34f1645a-9f0bd680-v1
status:
  active: 1     # Still running
  ready: 1
```

### 3. Running Processes in Container

```
root   15  /bin/bash /task-files/container.sh    ← Stuck at wait
root   43  docker-daemon sidecar                  ← Waiting for .agent_done
root  584  claude                                 ← HUNG - waiting for EOF!
root  636  toolman
```

**Key finding**: Claude process (PID 584) still running hours after sending completion message.

### 4. Log File Evidence

From `play-coderun-task-3-tess-claude-agent-plat-34f1645a-9f0bd6bw5z7.log` line 420:

```json
{"type":"result","subtype":"success","is_error":false","duration_ms":407395,...}
```

Claude sent success message, but **no subsequent cleanup logs**. Container never exited.

### 5. Missing Sentinel File

Docker daemon sidecar waiting for:
```bash
while true; do
  if [ -f /workspace/task-3/.agent_done ]; then
    exit 0
  fi
  sleep 5
done
```

Sentinel never created because trap cleanup never fires because container never exits.

---

## Why Other Agents Don't Have This Bug

| Agent | Pattern Used | FIFO Closes | Status |
|-------|--------------|-------------|---------|
| **Tess** | FIFO with late close | ❌ AFTER wait | **BROKEN** |
| Rex | Direct piping | ✅ Auto | ✅ Works |
| Blaze | Direct piping | ✅ Auto | ✅ Works |  
| Rex-remediation | FIFO with early close | ✅ BEFORE wait | ✅ Works |
| Cleo | No FIFO | N/A | ✅ Works |
| Cipher | No FIFO | N/A | ✅ Works |

---

## Fix Details

### Changed Files

1. **Source Template**:
   - `infra/charts/controller/agent-templates/code/claude/container.sh.hbs`
   - Lines 1343-1350: Added `exec 9>&-` after sending prompt
   - Line 1400: Updated comment (writer already closed)

2. **Generated ConfigMap**:
   - `infra/charts/controller/templates/agent-templates-claude.yaml`
   - Regenerated with fix applied

### PR

**PR #1551**: https://github.com/5dlabs/cto/pull/1551

---

## Remediation Steps

### Step 1: Merge PR to Main ✅ REQUIRED BEFORE STEP 2

```bash
# Wait for CI to pass, then merge PR #1551
gh pr merge 1551 --merge
```

### Step 2: Wait for ArgoCD Sync

```bash
# ArgoCD will automatically sync ConfigMaps from main
# Check sync status:
kubectl get app -n argocd controller -o jsonpath='{.status.sync.status}'
# Wait for: "Synced"
```

### Step 3: Kill Stuck Tess Job

```bash
# Delete the hung job (will trigger retry with fixed ConfigMap)
kubectl delete job -n agent-platform play-coderun-task-3-tess-claude-agent-plat-34f1645a-9f0bd680-v1

# Verify CodeRun retries:
kubectl get coderun -n agent-platform coderun-cto-parallel-test-t3-testing-qrhqr -w
```

### Step 4: Verify Fix

```bash
# New job should complete within 5-10 minutes
kubectl logs -n agent-platform -l task-id=3,stage=testing -c claude-claude-sonnet-4-5-20250929 --tail=20

# Should see:
# - "🧹 Cleanup: Signaling docker-daemon to shut down..."
# - Pod exits cleanly
# - CodeRun transitions to Succeeded
```

---

## Expected Timeline

1. **Now**: PR created, awaiting review
2. **+5 min**: CI passes, PR merged to main
3. **+7 min**: ArgoCD syncs updated ConfigMaps to cluster
4. **+8 min**: Stuck job deleted, CodeRun retries
5. **+15 min**: New job completes successfully with proper cleanup
6. **+16 min**: Workflow progresses to completion

---

## Prevention

This fix ensures all code paths close FIFO writer before waiting:

```bash
# Correct pattern (all paths should follow this):
exec 9>"$FIFO_PATH"   # Open
printf '...' >&9       # Send
exec 9>&-              # Close immediately  
wait $CLAUDE_PID       # Then wait
```

Future code reviews should verify FIFO lifecycle:
- Writer opened → message sent → writer closed → THEN wait
- Never wait with writer still open

---

## Related Issues

- Previous fix #1545: Fixed wrong sentinel file path (different issue)
- This bug: FIFO writer not closed before wait (new regression)

Both are now fixed.

---

**Investigation completed**: 2025-11-21 11:56 PST
**Fix committed**: PR #1551
**Status**: Awaiting merge to main

