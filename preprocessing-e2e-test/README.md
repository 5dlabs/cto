# Preprocessing Pipeline E2E Test

End-to-end test of the PRD preprocessing pipeline using a MiniMax swarm with Ralph loop.

## Overview

This test validates the new preprocessing pipeline that converts Markdown PRDs and supporting documentation into structured JSON before the main intake process. The test runs from Linear project creation through PR creation.

## Quick Start

```bash
# 1. Set up OAuth (required)
./setup-oauth.sh

# 2. Start tmux session (9 agent monitoring panes)
./tmux-session.sh --attach

# 3. In another terminal, run the Ralph loop
./loop.sh
```

## Prerequisites

### MiniMax API Key

The MiniMax API key is fetched from 1Password automatically:

```bash
export MINIMAX_API_KEY=$(op read 'op://Development/MiniMax API Key/credential')
```

Or set manually if 1Password is not available.

### Morgan OAuth Token

Linear integration requires valid OAuth tokens for Morgan:

```bash
./setup-oauth.sh
```

This script will guide you through:
1. Checking for existing tokens in 1Password
2. Adding tokens to `.env.local`
3. Regenerating launchd services

### Services

Ensure CTO services are running:

```bash
just launchd-status

# If not running:
just launchd-install
```

## Directory Structure

```
preprocessing-e2e-test/
├── README.md              # This file
├── PLAN.md                # Full preprocessing pipeline plan
├── agents/                # Subagent prompt files
│   ├── oauth.md           # OAuth validation
│   ├── environment.md     # Service health
│   ├── intake-mcp.md      # MCP intake execution
│   ├── tools-validation.md# Tools verification
│   ├── linear-sync.md     # Task → Linear sync
│   ├── linear-update.md   # Bidirectional sync test
│   ├── parity.md          # Feature parity check
│   ├── critic-observer.md # Multi-model validation
│   └── failback.md        # MiniMax failback to Claude Opus
├── issues/                # Issue logs for each agent
│   └── issues-{agent}.md
├── test-data/             # Test documents
│   ├── prd.md             # AlertHub PRD
│   ├── architecture.md    # AlertHub architecture
│   ├── research-effect-ts.md
│   ├── research-grpc-patterns.md
│   └── resources.md       # Research links
├── output/                # Iteration logs
├── agents.json            # Swarm configuration
├── ralph-coordination.json# Loop state
├── swarm-coordinator.md   # Main coordinator prompt
├── loop.sh                # Ralph loop script
├── tmux-session.sh        # TMux setup
└── setup-oauth.sh         # OAuth helper
```

## Subagents

| Agent | Priority | Responsibility |
|-------|----------|----------------|
| oauth-agent | 1 (blocking) | Validate Morgan OAuth tokens |
| environment-agent | 2 | Service health, restarts |
| intake-mcp-agent | 3 | Run MCP intake, create Linear |
| tools-validation-agent | 3 | Verify MCP tools config |
| linear-sync-agent | 4 | Task → Linear sync |
| critic-observer-agent | 4 | Multi-model feature test |
| linear-update-agent | 5 | Bidirectional sync test |
| parity-agent | 6 | Feature parity verification |
| failback-agent | 7 | Monitor MiniMax failures, trigger failback to Claude Opus |

## TMux Layout

The `tmux-session.sh` script creates:

- **Window 0 (swarm)**: 9 panes (oauth, environment, intake-mcp, tools-validation, linear-sync, linear-update, parity, critic-observer, failback)
- **Window 1 (logs)**: PM server, controller, milestone watch
- **Window 2 (state)**: Live coordination state (including failback)

Run `./loop.sh` in a separate terminal to start the Ralph loop.

Navigation:
- `Ctrl+b, 0/1/2` - Switch windows
- `Ctrl+b, arrow` - Switch panes
- `Ctrl+b, z` - Zoom pane
- `Ctrl+b, d` - Detach

## Milestones

| Milestone | Description |
|-----------|-------------|
| `oauth_valid` | Morgan tokens validated |
| `services_healthy` | All services responsive |
| `linear_project_created` | Linear project and PRD created |
| `tasks_generated` | Tasks JSON generated |
| `tasks_synced` | Tasks synced to Linear issues |
| `updates_tested` | Bidirectional sync works |
| `parity_verified` | Feature parity confirmed |
| `critic_tested` | Multi-model feature works |

## Issue Tracking

Each agent logs issues to `issues/issues-{agent}.md`:

```markdown
## ISSUE-{N}: {Brief title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what went wrong}
- **Root Cause**: {why it happened}
- **Resolution**: {how it was fixed}
```

## Failback

When MiniMax fails (timeouts, API errors, invalid output), the failback agent sets `failback.active` in `ralph-coordination.json`. The loop then uses **Claude Opus** instead of MiniMax for the next iteration. Ensure the `claude` CLI is installed when using failback.

## Completion

The test completes when:
1. All milestones in `ralph-coordination.json` are `true`
2. No open issues (issues_count.open = 0)
3. `.complete` file is created

## Troubleshooting

### OAuth Token Expired

```bash
./setup-oauth.sh
```

### Services Not Running

```bash
just launchd-uninstall
just launchd-install
just launchd-status
```

### MiniMax / Swarm Binary Not Found

The loop uses `claudesp-minimax` if available, otherwise `minimax`:

```bash
which claudesp-minimax minimax
# Expect at least one at ~/.local/bin/

# Install claudesp-minimax (sneakpeek):
npx @realmikekelly/claude-sneakpeek quick --provider minimax --api-key "$MINIMAX_API_KEY" --name claudesp-minimax
```

### Loop Stuck

Check the current iteration log:

```bash
cat output/iteration-$(jq -r '.iteration' ralph-coordination.json).log
```

Check for blocking issues:

```bash
grep -l "BLOCKING" issues/*.md
```
