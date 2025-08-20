# Claude Process Hanging Analysis - Review
**Reviewer: Claude (AI Assistant)**  
**Date: January 2025**

## Root Cause Analysis

After examining the actual implementation files, I can confirm the root cause is **a FIFO reader/writer deadlock** in the fallback mechanism when the sidecar `/input` endpoint fails.

## The Bug in Detail

Looking at `container-rex.sh.hbs` lines 1075-1120:

```bash
# Line 1076: Sidecar fails
echo "⚠️ Sidecar /input failed, falling back to direct FIFO write"

# Line 1078: Opens FIFO writer and KEEPS IT OPEN
exec 9>"$FIFO_PATH"

# Line 1079-1080: Tracks and writes
FIFO_OPENED=true
printf '{"type":"user","message":...}' >&9

# Line 1120: Waits for Claude to exit (DEADLOCK HERE!)
wait "$CLAUDE_PID"

# Lines 1136-1144: Would close fd 9, but never reaches here
if [ "$FIFO_OPENED" = "true" ]; then
  exec 9>&-  # This never executes because we're stuck at wait
fi
```

## Why This Causes a Deadlock

1. **Claude's behavior**: After completing the task and reporting success, Claude waits for EOF on its input stream (the FIFO) to know there's no more input coming.

2. **FIFO semantics**: A reader on a FIFO only gets EOF when ALL writers have closed their file descriptors to that FIFO.

3. **The deadlock**:
   - Script keeps fd 9 open while waiting for Claude to exit
   - Claude won't exit until it sees EOF (needs fd 9 closed)
   - Script won't close fd 9 until Claude exits
   - **Both processes wait forever**

## Evidence From the Code

Interestingly, the **SAFE_MODE section** (lines 999-1006) does it correctly:

```bash
exec 9>"$FIFO_PATH"
$CLAUDE_CMD < "$FIFO_PATH" &
CLAUDE_PID=$!
printf '{"type":"user"...}' >&9
exec 9>&-  # ← CLOSES IMMEDIATELY AFTER WRITING
wait $CLAUDE_PID  # ← THEN waits
```

This proves the developers understood the correct pattern but didn't apply it to the main execution path's fallback mechanism.

## The Same Bug Exists in Other Templates

Looking at `container-cleo.sh.hbs` (lines 415-423), it has the same bug:

```bash
exec 9>"$FIFO_PATH"
printf '{"type":"user"...}' >&9
# Missing: exec 9>&- here!
wait "$CLAUDE_PID"  # Deadlock!
exec 9>&- 2>/dev/null  # Never reached
```

## Recommended Fix

### Immediate Solution

In `container-rex.sh.hbs` (and all other container templates), change lines 1076-1081 from:

```bash
else
  echo "⚠️ Sidecar /input failed, falling back to direct FIFO write"
  exec 9>"$FIFO_PATH"
  FIFO_OPENED=true
  printf '{"type":"user"...}' >&9
fi
```

To:

```bash
else
  echo "⚠️ Sidecar /input failed, falling back to direct FIFO write"
  exec 9>"$FIFO_PATH"
  printf '{"type":"user"...}' >&9
  exec 9>&-  # Close immediately to send EOF
  FIFO_OPENED=false  # Mark as already closed
fi
```

### Why Rex Shows the Problem But Others Don't

The sidecar `/input` endpoint is failing specifically in Rex's environment. Possible reasons:
1. **Timing issue**: Rex container might start faster than sidecar initialization
2. **Network configuration**: Rex might have different network setup
3. **Resource constraints**: Rex pod might have different resource limits affecting sidecar startup

## Priority Actions

1. **Fix the deadlock** in all container templates (rex, cleo, tess, generic)
2. **Investigate why sidecar fails** - Add retry logic or health check:
   ```bash
   # Wait for sidecar to be ready
   for i in {1..10}; do
     if curl -fsS http://127.0.0.1:8080/health >/dev/null 2>&1; then
       echo "✓ Sidecar is ready"
       break
     fi
     echo "Waiting for sidecar... attempt $i/10"
     sleep 1
   done
   ```

3. **Add defensive timeout** as a safety net:
   ```bash
   if timeout 300 wait "$CLAUDE_PID"; then
     echo "✓ Claude completed"
   else
     echo "⚠️ Claude timeout, sending SIGTERM"
     kill -TERM "$CLAUDE_PID"
   fi
   ```

## Testing the Fix

To verify the fix works:
1. Apply the change to close fd 9 immediately after writing
2. Force the sidecar to fail (e.g., don't start it)
3. Confirm Claude completes and exits normally
4. Check no processes remain in `do_epoll_wait` state

## Summary

This is a classic FIFO deadlock bug. The script keeps the write end of the FIFO open while waiting for the reader (Claude) to exit, but the reader won't exit until it sees EOF (which requires closing the write end). The fix is simple: close the FIFO writer immediately after sending the input, just like the SAFE_MODE code already does correctly.