### Claude Hanging: Root Cause and Recommendations

Author: GPT-5 (Cursor AI Assistant)

---

### Most likely root cause

The fallback FIFO writer (`exec 9>"$FIFO_PATH"`) remains open until after `wait "$CLAUDE_PID"`. This prevents EOF from ever reaching Claude’s stdin, so Claude (in `--input-format stream-json`) stays in its event loop (`do_epoll_wait`) waiting for more input. The container then blocks on `wait`, producing a classic deadlock:

- Claude waits for EOF (but writer fd 9 is still open)
- The script waits for Claude to exit (but Claude won’t exit until EOF)

This only occurs when the sidecar `/input` path fails and the script falls back to direct FIFO use, because the current fallback logic intentionally keeps fd 9 open “until Claude exits.”

### Why Claude reports success but does not exit

In stream/interactive mode, the CLI can emit a success result before the process terminates. Termination typically happens when:

- stdin reaches EOF, or
- an explicit “end/session” signal is handled, or
- a single-turn/auto-exit flag is used (if supported by the CLI).

With the writer end held open, EOF never arrives. Hence the final JSON “success” is printed, but the process continues idling in `epoll_wait` for more input.

### Recommended fixes (prefer in this order)

1) Close the fallback FIFO writer immediately after sending the initial message

```bash
exec 9>"$FIFO_PATH"
printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}\n' "$USER_COMBINED" >&9
exec 9>&-  # close right away to deliver EOF for single-turn runs
```

- This converts the fallback into a single-turn input, allowing Claude to receive EOF and exit cleanly.
- If future multi-turn is needed without the sidecar, use a different mechanism (e.g., reopen per message) rather than keeping one writer fd open.

2) Prefer the sidecar path; add a short readiness/retry before falling back

```bash
for i in 1 2 3 4 5; do
  curl -fsS http://127.0.0.1:8080/healthz >/dev/null 2>&1 && break
  sleep 0.2
done
```

- Only fall back if the sidecar remains unavailable. The sidecar should open/write/close correctly for multi-turn flows.

3) If supported, enable a single-turn/auto-exit CLI option

- Use a flag like “single-turn” or “exit-on-result” (exact name depends on the CLI) so the process exits directly after success even if stdin remains open.

4) Cleanup ordering and safety

- If you must keep fd 9 for any reason, ensure it is closed before the `wait` call, or set a trap:

```bash
trap 'exec 9>&- 2>/dev/null || true' EXIT
```

- This guarantees the writer is not left open during the `wait` phase.

### Additional diagnostics to confirm

- Verify open FDs and who holds the FIFO open:
  - `ls -l /proc/$CLAUDE_PID/fd`
  - `lsof | grep agent-input.jsonl`
  - `fuser -v "$FIFO_PATH"`

- Observe wait state live:
  - `cat /proc/$CLAUDE_PID/wchan` (expect `do_epoll_wait`)
  - `strace -f -p $CLAUDE_PID -e trace=read,write,epoll_wait -s 0` to see blocking reads

- Sidecar readiness root cause:
  - Check sidecar logs and add a `/healthz` endpoint if missing; confirm timing/race at container startup.

### Executive takeaway

Close the fallback FIFO writer immediately after the initial write. That delivers EOF to Claude, allowing it to exit. Add a brief sidecar readiness wait to reduce fallback occurrences. Optionally enable a single-turn/auto-exit flag if the CLI supports it. These changes directly resolve the deadlock between Claude waiting for EOF and the container waiting for Claude.


