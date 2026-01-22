# PR Merge Loop - Ralph System

A dual-agent system for continuously merging pending pull requests, remediating bug-bot comments, and ensuring CI passes. Runs infinitely in the background.

## Overview

This system uses two AI agents with distinct roles:

| Agent | CLI | Role |
|-------|-----|------|
| **Merger** | Claude | Works on PRs, fixes issues, merges when ready |
| **Monitor** | Droid | Watches patterns, identifies automation opportunities, implements code fixes |

**The Key Insight**: Every time Claude has to manually fix something, Droid asks: "What code change would prevent this next time?" Then Droid implements that fix.

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
```

### Unattended Mode (Default)

Both agents run in fully unattended mode by default:

- **Claude** uses `--dangerously-skip-permissions` to auto-approve all operations
- **Droid** uses `droid exec --skip-permissions-unsafe --auto high` for non-interactive execution

### Interactive Mode

For debugging or manual oversight:

```bash
# Interactive mode (prompts for approval)
./run-merger.sh --interactive
```

### Coordination

The monitor script automatically waits for the merger to start before beginning checks:

```bash
# Skip the wait (useful for debugging)
./run-monitor.sh --no-wait
```

## Files

| File | Description |
|------|-------------|
| `merger-prompt.md` | Instructions for Claude merger agent |
| `monitor-prompt.md` | Instructions for Droid monitor agent |
| `ralph-coordination.json` | Shared state between agents |
| `progress.txt` | Human-readable progress log |
| `run-merger.sh` | Launch script for Claude |
| `run-monitor.sh` | Launch script for Droid |
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

3. **Fix Bug-Bot Comments**
   - Parse comments for actionable items
   - Fix code issues (lint, format, tests)
   - Push fixes
   - Wait for CI

4. **Ensure CI Passes**
   - Check status: `gh pr checks <number>`
   - Fix failing checks
   - Re-run checks if needed

5. **Merge When Ready**
   ```bash
   gh pr merge <number> --squash --delete-branch
   ```

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