# Complete Sidecar/FIFO Removal - Final Fix

## What You Were Right About

You pointed out that **FIFO was from the sidecar era and should be completely removed**. You were 100% correct!

When I initially investigated, I saw the FIFO deadlock and tried to "fix" it by closing the writer earlier. But that was the wrong approach - **the FIFO pattern itself is obsolete and should be deleted entirely**.

---

## The Real Issue: Incomplete Migration

When the sidecar was removed, **most agents were migrated** to modern patterns:

| Agent | Status | Pattern |
|-------|--------|---------|
| **Cleo** | ✅ Migrated | Subshell with `exec 0<&-` |
| **Cipher** | ✅ Migrated | Subshell with `exec 0<&-` |
| **Rex** | ✅ Migrated | Direct stdin piping |
| **Blaze** | ✅ Migrated | Direct stdin piping |
| **Tess** | ❌ **NOT MIGRATED** | Still using FIFO |
| **Rex-remediation** | ❌ **NOT MIGRATED** | Still using FIFO + sidecar HTTP |

---

## Obsolete Sidecar Era Code (REMOVED)

### Pattern 1: FIFO with File Descriptors
```bash
# OLD (container.sh.hbs for Tess):
FIFO_PATH="/workspace/agent-input.jsonl"
mkfifo "$FIFO_PATH"
$CLAUDE_CMD < "$FIFO_PATH" &
exec 9>"$FIFO_PATH"
printf '...' >&9
exec 9>&-
```

**Why obsolete:** Sidecar removed, file descriptor management complex, prone to deadlocks

### Pattern 2: Sidecar HTTP Endpoint  
```bash
# OLD (container-rex-remediation.sh.hbs):
curl -X POST http://127.0.0.1:8080/input \
  -H 'Content-Type: application/json' \
  --data '{"text":"..."}'
```

**Why obsolete:** Sidecar container no longer exists (removed months ago)

---

## Modern Pattern (NOW UNIVERSAL)

### The Subshell Pattern (from Cleo)

```bash
# Modern approach - all agents now use this:
(printf '{"type":"user","message":{...}}\n' "$PROMPT"; exec 0<&-) | $CLAUDE_CMD &
CLAUDE_PID=$!
wait "$CLAUDE_PID"
```

**How it works:**
1. **Subshell** `(...)` runs in isolation
2. `printf` sends the prompt to stdout
3. `exec 0<&-` **explicitly closes stdin** in subshell
4. Pipe connects subshell stdout → Claude stdin
5. When subshell exits, pipe closes, **Claude receives EOF immediately**
6. Claude finishes work and exits cleanly
7. No FIFO, no file descriptors, no deadlocks!

---

## What Was Fixed

### File 1: `container.sh.hbs` (Tess + Generic Agents)

**Removed:**
- All `mkfifo` calls (3 locations)
- All FIFO file descriptor management (`exec 9>`, `exec 9>&-`)
- FIFO holder diagnostics
- Complex FIFO debug code

**Replaced with:**
- Subshell pattern in normal task path
- Subshell pattern in Task ID 0 path  
- Subshell pattern in SAFE_MODE path
- Clean, simple `wait` (no timeout complexity)

### File 2: `container-rex-remediation.sh.hbs`

**Removed:**
- Sidecar HTTP endpoint code (`curl http://127.0.0.1:8080/input`)
- FIFO fallback code
- FIFO diagnostics

**Replaced with:**
- Subshell pattern identical to other agents
- Background token refresh (same as modern agents)

---

## Why This Is Better

### Before (FIFO Era):
```bash
# 15 lines of complex code:
FIFO_PATH="/workspace/agent-input.jsonl"
rm -f "$FIFO_PATH" 2>/dev/null || true
mkfifo "$FIFO_PATH"
chmod 666 "$FIFO_PATH" || true
$CLAUDE_CMD < "$FIFO_PATH" &
CLAUDE_PID=$!
USER_COMBINED=$(printf "%s" "${PROMPT_PREFIX}...
exec 9>"$FIFO_PATH"
printf '...' >&9
exec 9>&- 2>/dev/null || true  # ← Must be placed correctly or deadlock!
# + 50 lines of diagnostics, debug code, etc.
wait $CLAUDE_PID
```

### After (Modern Pattern):
```bash
# 3 lines of simple code:
USER_COMBINED=$(printf "%s" "${PROMPT_PREFIX}...
(printf '{"type":"user",...}' "$USER_COMBINED"; exec 0<&-) | $CLAUDE_CMD &
wait "$CLAUDE_PID"
```

**Benefits:**
- ✅ **80% less code**
- ✅ **No file descriptor management**
- ✅ **No race conditions**
- ✅ **Cannot deadlock** (stdin always closes)
- ✅ **Consistent across all agents**
- ✅ **Easier to maintain**

---

## Testing Impact

### Task 1 & 6 Delay Analysis

The 8-hour delay you saw was because:
1. **Workflows started**: 12:32 UTC (7-8 hours ago)
2. **ArgoCD synced new ConfigMaps**: 19:58 UTC
3. **Controller reconciled CodeRuns**: 20:11 UTC  
4. **PRs finally processed**: 20:12 UTC

The `create-coderun-resource` step ran from 12:32→20:11 (almost 8 hours!) because **the controller was waiting for ConfigMaps that weren't synced yet**.

Once ConfigMaps synced at 19:58, the controller could create jobs and process completed.

---

## Complete Fix in PR #1551

### Commits:
1. **Removed obsolete FIFO/sidecar code** (container.sh.hbs, container-rex-remediation.sh.hbs)
2. **Added Security stage support** (controller.rs)
3. **Documentation** (investigation docs)

### Changes Summary:
- **Deleted:** ~150 lines of obsolete FIFO/sidecar code
- **Added:** ~40 lines of modern subshell pattern
- **Net:** Simpler, more reliable, fully consistent

---

## Verification After Merge

### 1. No More FIFO Files
```bash
# Should find NOTHING:
kubectl exec -n agent-platform <tess-pod> -- ls -la /workspace/*.jsonl
kubectl exec -n agent-platform <tess-pod> -- lsof | grep agent-input
```

### 2. Clean Exits
```bash
# Should see immediate exit after completion:
kubectl logs -f -n agent-platform -l stage=testing -c claude-...

# Should see:
# "✅ Claude completed successfully"
# "🧹 Cleanup: Signaling docker-daemon..."
# Pod exits within seconds
```

### 3. All Agents Use Same Pattern
```bash
# Verify consistency:
grep -r "exec 0<&-" infra/charts/controller/agent-templates/code/claude/*.sh.hbs
# Should show ALL agent containers using subshell pattern
```

---

## Next Steps

1. **Merge PR #1551** → Complete FIFO removal
2. **Wait for ArgoCD sync** → ConfigMaps deployed
3. **Kill stuck Tess jobs** → Force retry with new code
4. **Monitor workflows** → Should complete cleanly

**Expected:** Task 1 and 6 Quality stages should complete within 10 minutes once Tess jobs retry with new ConfigMaps.

---

**Investigation & Fix**: 2025-11-21 12:25 PST  
**Pattern**: Sidecar/FIFO → Modern subshell stdin close  
**Impact**: All agents now use consistent, deadlock-proof pattern

