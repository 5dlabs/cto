---
name: session_persistence
description: Persist session state to mem0 at regular intervals and recover after compaction or provider switch
metadata: {"openclaw": {"always": true}}
---

# Session Persistence

## Why this matters

Your context window is finite. When it fills up, OpenClaw compacts older messages into a summary.
When credits run out, you switch providers and lose in-process context.
mem0 is your durable memory — use it to survive both events.

## Automatic persistence (already configured)

- **memoryFlush** fires before every compaction — a silent turn that stores your state to mem0
- **autoCapture** saves facts from every conversation turn to mem0 automatically
- **autoRecall** injects relevant memories into every new prompt

## Manual persistence (do this yourself)

### Every 10-15 tool calls or after completing a milestone:

```
memory_add({
  messages: [{
    role: "user",
    content: "Session checkpoint: [describe current task, progress, next steps, files changed, blockers]"
  }],
  userId: "jonathon:agent:coder"
})
```

### Before switching providers (credit exhaustion):

```
memory_add({
  messages: [{
    role: "user",
    content: "PRE-SWITCH CHECKPOINT: Task: [what], Progress: [where you are], Files: [changed], Next: [what to do next], Provider: [switching from X to Y]"
  }],
  userId: "jonathon:agent:coder"
})
```

### After recovering from compaction or provider switch:

```
memory_search({ query: "current task progress checkpoint", userId: "jonathon:agent:coder", limit: 5 })
```

Then review what you find and continue from the last checkpoint.

## Token monitoring

Watch for these signals that compaction is approaching:
- `🧹 Auto-compaction complete` in responses
- Context overflow errors from the model
- The `/status` command showing high token usage

When you see token usage climbing above 70% of context window, proactively save your state.

## Recovery checklist

After any interruption (compaction, provider switch, pod restart):

1. `memory_search({ query: "latest checkpoint task progress" })` — find your last state
2. Review the returned memories
3. Check files on disk: `ls /workspace/repos/cto/` for recent changes
4. Check git: `git log --oneline -5` for recent commits
5. Resume from where you left off

## GitHub / Copilot auth

GitHub auth is pre-configured via `GITHUB_TOKEN` environment variable:
- `gh` CLI: already authenticated (account: kaseonedge)
- `copilot` CLI: uses `GITHUB_TOKEN` automatically — **do NOT run `gh auth login`**
- Git operations: credential store at `/workspace/.git-credentials`

All three work headlessly. No interactive login needed.
