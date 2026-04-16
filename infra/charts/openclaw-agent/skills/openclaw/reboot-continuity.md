---
name: reboot_continuity
description: Resume work after pod restart by reading handoff state and mem0 memories
metadata: {"openclaw": {"always": true}}
---

# Reboot Continuity

## On FIRST boot or after restart

When you detect you have NO prior conversation context (fresh session, empty history), you have just rebooted. Do this IMMEDIATELY:

### Step 1: Read handoff state
```bash
cat /workspace/HANDOFF.md
```
This file contains your prior session context — what you were working on, progress, and next steps.

### Step 2: Check mem0 for recent context
Your mem0 memories persist across reboots. Recall your recent work:
- Active tasks and their status
- Files you were editing
- PRs you created or were reviewing
- Debugging findings and decisions made

### Step 3: Check workspace state
```bash
# Check for uncommitted work
cd /workspace/repos/cto && git status && git stash list

# Check for in-progress branches
git branch --sort=-committerdate | head -5

# Check for running processes or builds
ps aux | grep -v grep | grep -E 'cargo|node|python'
```

### Step 4: Resume or report
- If you had active work: **resume it immediately** without asking
- If you had a PR in review: check CI status and address any failures
- If you were idle: check NATS messages and open issues labeled 'coder'
- Post a brief status update to your Discord channel:
  ```
  message({ action: "send", text: "🔄 Rebooted. Resuming: [brief description of what you're picking up]" })
  ```

## Before shutdown / compaction

When you detect compaction is coming (memory flush prompt) or you're about to lose context:

### Write handoff state
```bash
cat > /workspace/HANDOFF.md << 'EOF'
# Handoff State
Updated: [timestamp]

## Active Task
[What you're working on right now]

## Progress
[What's done, what's remaining]

## Next Steps
[Exact next action to take on resume]

## Key Context
- Branch: [current branch]
- PR: [PR number if any]
- Files: [key files being edited]
- Blockers: [anything blocking progress]
EOF
```

### Flush to mem0
The compaction memory flush prompt handles this — make sure to include ALL active task context.

## Rules
- NEVER ask "what should I work on?" after a reboot — read HANDOFF.md and resume
- ALWAYS write HANDOFF.md before long-running ACP sessions (they may outlive you)
- Keep HANDOFF.md under 2000 chars — concise, actionable, no prose
- If HANDOFF.md is missing or empty, check mem0 and NATS for recent task assignments
