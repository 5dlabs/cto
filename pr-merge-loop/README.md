# PR Merge Loop - Ralph System

A dual-agent system for continuously merging pending pull requests, remediating bug-bot comments, and ensuring CI passes. Runs infinitely in the background.

## Overview

This system uses three AI agents with distinct roles:

| Agent | CLI | Role |
|-------|-----|------|
| **Merger** | Claude | Works on PRs, fixes issues, merges when ready |
| **Monitor** | Droid | Watches patterns, identifies automation opportunities, implements code fixes |
| **Remediation** | Claude | Fixes failures that block PR merging (merge conflicts, CI failures, etc.) |

**The Key Insight**: Every time Claude has to manually fix something, Droid asks: "What code change would prevent this next time?" Then Droid implements that fix. When the Merger gets stuck, Remediation unblocks it.

### Progressive Hardening Flow

```
Run N:   Claude fixes bug-bot comment → merges PR
         ↓
         Droid observes: "Claude fixed same lint issue 5 times"
         ↓
         Droid implements: Pre-commit hook to catch this
         ↓
Run N+1: Issue is caught before PR → Claude has less work
```

**Goal**: Each cycle makes the codebase MORE reliable and LESS dependent on manual fixes.

## Quick Start

```bash
# Terminal 1: Start the merger agent (unattended mode)
./run-merger.sh

# Terminal 2: Start the monitor agent (waits for merger to be running)
./run-monitor.sh

# Terminal 3: Start the remediation agent (waits for merger to be running)
./run-remediation.sh
```

### Unattended Mode (Default)

All agents run in fully unattended mode by default:

- **Merger (Claude)** uses `--dangerously-skip-permissions` to auto-approve all operations
- **Remediation (Claude)** uses `--dangerously-skip-permissions` to auto-approve all operations
- **Monitor (Droid)** uses `droid exec --skip-permissions-unsafe --auto high` for non-interactive execution

### Interactive Mode

For debugging or manual oversight:

```bash
# Interactive mode (prompts for approval)
./run-merger.sh --interactive
```

### Coordination

Both monitor and remediation scripts automatically wait for the merger to start before beginning:

```bash
# Skip the wait (useful for debugging)
./run-monitor.sh --no-wait
./run-remediation.sh --no-wait
```

## Files

| File | Description |
|------|-------------|
| `merger-prompt.md` | Instructions for Claude merger agent |
| `monitor-prompt.md` | Instructions for Droid monitor agent |
| `remediation-prompt.md` | Instructions for Claude remediation agent |
| `ralph-coordination.json` | Shared state between agents |
| `progress.txt` | Human-readable progress log |
| `run-merger.sh` | Launch script for Claude merger |
| `run-monitor.sh` | Launch script for Droid monitor |
| `run-remediation.sh` | Launch script for Claude remediation |
| `tmux-session.sh` | Tmux session for monitoring |

## Workflow

### Phase 1: Discover Pending PRs

1. List all open PRs: `gh pr list --state open --json number,title,headRefName,baseRefName,mergeable,mergeStateStatus,statusCheckRollup`
2. Filter for PRs that need work:
   - `mergeable: CONFLICTING` - Has merge conflicts
   - `mergeStateStatus: BLOCKED` - CI failing or review required
   - `mergeStateStatus: BEHIND` - Needs rebase
   - Has bug-bot comments

### Phase 2: Process Each PR

For each PR:

1. **Check Status**
   ```bash
   gh pr view <number> --json mergeable,mergeStateStatus,statusCheckRollup,comments
   ```

2. **Handle Merge Conflicts**
   - Fetch latest base branch
   - Rebase onto base
   - Resolve conflicts
   - Push with `--force-with-lease`
   - **If conflicts persist after 3 attempts**, add to issue queue for Remediation

3. **Fix Bug-Bot Comments**
   - Parse comments for actionable items
   - Fix code issues (lint, format, tests)
   - Push fixes
   - Wait for CI
   - **If issues persist**, add to issue queue for Remediation

4. **Ensure CI Passes**
   - Check status: `gh pr checks <number>`
   - Fix failing checks
   - Re-run checks if needed
   - **If CI keeps failing**, add to issue queue for Remediation

5. **Merge When Ready**
   ```bash
   gh pr merge <number> --squash --delete-branch
   ```

### Phase 2.5: Remediation (When Merger Gets Stuck)

When the Merger Agent adds an issue to the queue:
1. **Remediation Agent** polls the queue every 10 seconds
2. **Claims the issue** (marks as "claimed")
3. **Investigates** the root cause
4. **Fixes** the issue (merge conflicts, CI failures, etc.)
5. **Resolves** the issue (marks as "resolved")
6. **Merger Agent** retries the PR automatically

### Phase 3: Continuous Loop

After processing all PRs, wait 5 minutes and repeat.

## Coordination System

The agents coordinate via `ralph-coordination.json`:

```json
{
  "merger": {
    "status": "running|waiting|failed|idle",
    "currentPr": 123,
    "lastUpdate": "2026-01-20T12:00:00Z",
    "prsProcessed": 5,
    "prsMerged": 3,
    "prsFailed": 1
  },
  "monitor": {
    "status": "running|idle",
    "lastCheck": "2026-01-20T12:00:00Z",
    "fixesImplemented": 0
  },
  "remediation": {
    "status": "running|idle",
    "currentIssue": "issue-1234567890-12345",
    "lastCheck": "2026-01-20T12:00:00Z",
    "issuesResolved": 2,
    "issuesFailed": 0
  },
  "issueQueue": [
    {
      "id": "issue-1234567890-12345",
      "timestamp": "2026-01-20T12:00:00Z",
      "prNumber": 123,
      "type": "merge_conflict",
      "description": "PR #123 has merge conflicts",
      "error": "Merge conflict in src/main.rs",
      "status": "pending|claimed|resolved|failed",
      "retryCount": 0
    }
  ],
  "hardeningActions": [
    {
      "timestamp": "2026-01-20T12:00:00Z",
      "observation": "Claude fixed same clippy warning 5 times",
      "rootCause": "Pre-commit hook doesn't run clippy",
      "fix": "Added clippy check to .pre-commit-config.yaml",
      "files": [".pre-commit-config.yaml"]
    }
  ],
  "circuitBreaker": {
    "state": "closed|open",
    "failureCount": 0,
    "threshold": 3
  }
}
```

### Issue Queue

When the Merger Agent encounters a failure it can't fix, it adds an issue to the queue:
- **prNumber**: The PR that's blocked
- **type**: Type of issue (merge_conflict, ci_failure, bug_bot, etc.)
- **description**: Human-readable description
- **error**: Specific error message
- **status**: Current status (pending, claimed, resolved, failed)

The Remediation Agent polls this queue and fixes issues.

### Hardening Actions

When Droid implements a code fix, it logs:
- **observation**: What Claude had to do manually
- **rootCause**: Why the code didn't handle it
- **fix**: What Droid changed
- **files**: Which files were modified

This creates a trail of improvements for each cycle.

## Prerequisites

### Required Tools

```bash
# GitHub CLI
which gh

# Git
which git

# Claude CLI
which claude

# Droid CLI (for monitor)
which droid
```

### GitHub Authentication

```bash
# Verify authentication
gh auth status

# If not authenticated:
gh auth login
```

### Repository Setup

Ensure you're in the CTO repository root:

```bash
cd /Users/jonathonfritz/code/work-projects/5dlabs/cto
```

## Monitoring

### Check Status

```bash
# View coordination state
cat pr-merge-loop/ralph-coordination.json | jq .

# View progress log
tail -f pr-merge-loop/progress.txt
```

### Tmux Session

```bash
# Create tmux session with all panes
./tmux-session.sh

# Or attach to existing
tmux attach -t pr-merge-ralph
```

## Troubleshooting

### No PRs Found

If no PRs are being processed:
1. Check `gh pr list --state open` manually
2. Verify GitHub authentication: `gh auth status`
3. Check if PRs are filtered out (check coordination state)

### Merge Conflicts Stuck

If conflicts keep recurring:
1. Check if base branch is correct (should be `develop`)
2. Verify no force pushes to base branch
3. Consider rebasing all PRs at once

### CI Always Failing

If CI checks consistently fail:
1. Check monitor agent logs for patterns
2. Look for hardening actions that might help
3. Review `progress.txt` for common issues

### Circuit Breaker Opens

If the circuit breaker opens (too many failures):
1. Check `ralph-coordination.json` for failure count
2. Review `progress.txt` for error patterns
3. Reset: Update coordination file to reset circuit breaker

## Related Documentation

- [Latitude Install](../latitude-install/) - Similar Ralph loop pattern
- [Lifecycle Test](../lifecycle-test/) - Another Ralph loop example
- [Git Integration Skills](../templates/skills/workflow/git-integration/) - Git/GitHub patterns